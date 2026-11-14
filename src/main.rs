mod history;
mod prefs;
mod session;

use base64::Engine as _;
use gtk::prelude::*;
use history::History;
use image::GenericImageView as _;
use prefs::{Placement, Prefs};
use serde_json::{Value, json};
use session::{Session, SessionTab};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::platform::unix::{EventLoopBuilderExtUnix, WindowExtUnix};
use tao::window::{Fullscreen, Icon, Window, WindowBuilder};
use webkit2gtk::{WebContextExt as _, WebViewExt as _};
use wry::dpi::{PhysicalPosition, PhysicalSize};
use wry::{
    NewWindowResponse, PageLoadEvent, Rect, WebView, WebViewBuilder, WebViewBuilderExtUnix,
    WebViewExtUnix,
};

const CHROME_H: f64 = 86.0;

const START_HTML: &str = include_str!("../ui/start.html");
const SETTINGS_HTML: &str = include_str!("../ui/settings.html");
const LOGO_DATA_URI: &str = concat!("data:image/png;base64,", include_str!("../assets/logo.b64"));
const I18N_JS: &str = include_str!("../ui/i18n.js");
const THEME_JS: &str = include_str!("../ui/theme.js");

const SHORTCUT_SCRIPT: &str = r#"
(function () {
  if (window.__mestyShortcuts) return;
  window.__mestyShortcuts = true;
  window.addEventListener('keydown', function (event) {
    if (!(event.ctrlKey || event.metaKey) || event.altKey) return;
    var key = (event.key || '').toLowerCase();
    var cmd = null;
    if (key === 't' && event.shiftKey) cmd = 'reopen_tab';
    else if (key === 't') cmd = 'new_tab';
    else if (key === 'n') cmd = event.shiftKey ? 'new_incognito' : 'home_editor';
    else if (key === 'w') cmd = 'close_active';
    else if (key === 'l') cmd = 'focus_address';
    else if (key === 'f') cmd = 'open_find';
    else if (key === '=' || key === '+') cmd = 'zoom_in';
    else if (key === '-') cmd = 'zoom_out';
    else if (key === '0') cmd = 'zoom_reset';
    else if (key === 'tab') cmd = event.shiftKey ? 'prev_tab' : 'next_tab';
    if (!cmd) return;
    event.preventDefault();
    event.stopPropagation();
    if (window.ipc) window.ipc.postMessage(JSON.stringify({ cmd: cmd }));
  }, true);
  window.addEventListener('keydown', function (event) {
    if (event.key === 'F11') {
      event.preventDefault();
      event.stopPropagation();
      if (window.ipc) window.ipc.postMessage(JSON.stringify({ cmd: 'fullscreen' }));
    }
  }, true);
})();
"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabKind {
    Start,
    Settings,
    Web,
}

struct Tab {
    id: u64,
    kind: TabKind,
    webview: WebView,
    title: String,
    url: String,
    favicon: String,
    zoom: f64,
    external: bool,
    loading: bool,
    incognito: bool,
}

impl Tab {
    fn display_url(&self) -> &str {
        if self.external || self.kind == TabKind::Web {
            &self.url
        } else if self.kind == TabKind::Settings {
            "mesty://settings"
        } else {
            ""
        }
    }
}

enum UserEvent {
    Chrome(String),
    Content(u64, String),
    Title(u64, String),
    Load(u64, PageLoadEvent, String),
    OpenTab(u64, String),
    Favicon(u64, String),
}

struct App {
    window: Window,
    fixed: gtk::Fixed,
    chrome: WebView,
    chrome_ready: bool,
    tabs: Vec<Tab>,
    active: u64,
    next_id: u64,
    prefs: Prefs,
    chrome_height: f64,
    closed: Vec<(TabKind, Option<String>)>,
    history: History,
    session: Session,
    pending_editor: Option<u64>,
    restoring: bool,
    proxy: EventLoopProxy<UserEvent>,
}

impl App {
    fn new(
        window: Window,
        fixed: gtk::Fixed,
        chrome: WebView,
        proxy: EventLoopProxy<UserEvent>,
    ) -> Self {
        Self {
            window,
            fixed,
            chrome,
            chrome_ready: false,
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
            prefs: Prefs::load(),
            chrome_height: CHROME_H,
            closed: Vec::new(),
            history: History::load(),
            session: Session::load(),
            pending_editor: None,
            restoring: true,
            proxy,
        }
    }

    fn save_session(&self) {
        if self.restoring {
            return;
        }

        let mut tabs = Vec::new();
        let mut active = 0;

        for tab in &self.tabs {
            if tab.incognito {
                continue;
            }

            if tab.id == self.active {
                active = tabs.len();
            }

            let (kind, url) = if tab.kind == TabKind::Web || (tab.external && !tab.url.is_empty()) {
                ("web".to_string(), tab.url.clone())
            } else if tab.kind == TabKind::Settings {
                ("settings".to_string(), String::new())
            } else {
                ("start".to_string(), String::new())
            };

            tabs.push(SessionTab {
                kind,
                url,
                zoom: tab.zoom,
            });
        }

        Session { tabs, active }.save();
    }

    fn restore_session(&mut self) {
        let saved = self.session.clone();
        if saved.tabs.is_empty() {
            return;
        }

        let mut active_id = None;
        for (index, saved_tab) in saved.tabs.iter().enumerate() {
            let kind = match saved_tab.kind.as_str() {
                "settings" => TabKind::Settings,
                "web" => TabKind::Web,
                _ => TabKind::Start,
            };
            let url = if kind == TabKind::Web {
                Some(saved_tab.url.clone())
            } else {
                None
            };

            let id = self.new_tab(kind, url);
            if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                tab.zoom = saved_tab.zoom;
                tab.webview.webview().set_zoom_level(saved_tab.zoom);
            }
            if index == saved.active {
                active_id = Some(id);
            }
        }

        if let Some(id) = active_id.or_else(|| self.tabs.first().map(|tab| tab.id)) {
            self.activate(id);
        }
    }

    fn finish_restore(&mut self) {
        self.restoring = false;
        self.save_session();
    }

    fn content_bounds(&self) -> (i32, i32, i32, i32) {
        let size = self.window.inner_size();
        let scale = self.window.scale_factor();
        let chrome_h = (self.chrome_height * scale).round().max(1.0) as i32;
        let width = size.width.max(1) as i32;
        let height = (size.height as i32 - chrome_h).max(1);
        (0, chrome_h, width, height)
    }

    fn layout_all(&self) {
        let scale = self.window.scale_factor();
        let size = self.window.inner_size().to_logical::<i32>(scale);
        let chrome_h = self.chrome_height.round().max(1.0) as i32;
        let width = size.width.max(1);
        let height = (size.height - chrome_h).max(1);

        let chrome_view = self.chrome.webview();
        self.fixed.move_(&chrome_view, 0, 0);
        chrome_view.set_size_request(width, chrome_h);

        for tab in &self.tabs {
            let view = tab.webview.webview();
            self.fixed.move_(&view, 0, chrome_h);
            view.set_size_request(width, height);
        }
    }

    fn new_tab(&mut self, kind: TabKind, url: Option<String>) -> u64 {
        self.new_tab_with(kind, url, false)
    }

    fn new_tab_with(&mut self, kind: TabKind, url: Option<String>, incognito: bool) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let (x, y, width, height) = self.content_bounds();
        let bounds = Rect {
            position: PhysicalPosition::new(x, y).into(),
            size: PhysicalSize::new(width, height).into(),
        };

        let ipc_proxy = self.proxy.clone();
        let title_proxy = self.proxy.clone();
        let load_proxy = self.proxy.clone();
        let window_proxy = self.proxy.clone();

        let mut builder = WebViewBuilder::new()
            .with_bounds(bounds)
            .with_visible(false)
            .with_focused(false)
            .with_initialization_script(SHORTCUT_SCRIPT)
            .with_ipc_handler(move |request| {
                let _ = ipc_proxy.send_event(UserEvent::Content(id, request.body().clone()));
            })
            .with_document_title_changed_handler(move |title| {
                let _ = title_proxy.send_event(UserEvent::Title(id, title));
            })
            .with_on_page_load_handler(move |event, url| {
                let _ = load_proxy.send_event(UserEvent::Load(id, event, url));
            })
            .with_new_window_req_handler(move |url, _features| {
                let _ = window_proxy.send_event(UserEvent::OpenTab(id, url));
                NewWindowResponse::Deny
            })
            .with_incognito(incognito);

        let (title, display_url, external) = match kind {
            TabKind::Start => (
                if incognito {
                    "New Incognito Tab".to_string()
                } else {
                    "New Tab".to_string()
                },
                String::new(),
                false,
            ),
            TabKind::Settings => ("Customize Mesty".to_string(), String::new(), false),
            TabKind::Web => {
                let target = url.clone().unwrap_or_default();
                let title = target
                    .split("://")
                    .nth(1)
                    .and_then(|rest| rest.split('/').next())
                    .filter(|host| !host.is_empty())
                    .unwrap_or("New Tab")
                    .to_string();
                (title, target, true)
            }
        };

        builder = match kind {
            TabKind::Start => builder.with_html(page_html(START_HTML)),
            TabKind::Settings => builder.with_html(page_html(SETTINGS_HTML)),
            TabKind::Web => {
                if let Some(target) = url.as_deref() {
                    builder.with_url(target)
                } else {
                    builder.with_html(page_html(START_HTML))
                }
            }
        };

        let webview = builder
            .build_gtk(&self.fixed)
            .expect("failed to create webview");

        if let Some(context) = webview.webview().context() {
            if !incognito {
                let dir = dirs::cache_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
                    .join("mestybrovser")
                    .join("favicons");
                if std::fs::create_dir_all(&dir).is_ok()
                    && let Some(path) = dir.to_str()
                {
                    context.set_favicon_database_directory(Some(path));
                }
            }
            context.set_preferred_languages(&preferred_languages(&self.prefs.language));
        }

        let favicon_proxy = self.proxy.clone();
        webview
            .webview()
            .connect_favicon_notify(move |view: &webkit2gtk::WebView| {
                let Some(surface) = view.favicon() else {
                    return;
                };
                let mut png = Vec::new();
                if surface.write_to_png(&mut png).is_err() || png.len() > 512 * 1024 {
                    return;
                }
                let encoded = base64::engine::general_purpose::STANDARD.encode(&png);
                let _ = favicon_proxy.send_event(UserEvent::Favicon(
                    id,
                    format!("data:image/png;base64,{encoded}"),
                ));
            });

        self.tabs.push(Tab {
            id,
            kind,
            webview,
            title,
            url: display_url,
            favicon: String::new(),
            zoom: 1.0,
            external,
            loading: true,
            incognito,
        });

        self.layout_all();
        self.save_session();
        id
    }

    fn send_top_sites(&self) {
        let sites = self.history.top(&self.prefs.top_sites, 8);
        let Ok(sites_json) = serde_json::to_string(&sites) else {
            return;
        };
        for tab in &self.tabs {
            if tab.kind == TabKind::Start && !tab.external {
                let _ = tab.webview.evaluate_script(&format!(
                    "window.mestySetTopSites && window.mestySetTopSites({sites_json})"
                ));
            }
        }
    }

    fn apply_languages(&self) {
        let languages = preferred_languages(&self.prefs.language);
        for tab in &self.tabs {
            if let Some(context) = tab.webview.webview().context() {
                context.set_preferred_languages(&languages);
            }
        }
    }

    fn reorder_tabs(&mut self, ids: &[u64]) {
        let mut ordered = Vec::with_capacity(self.tabs.len());
        for id in ids {
            if let Some(index) = self.tabs.iter().position(|tab| tab.id == *id) {
                ordered.push(self.tabs.remove(index));
            }
        }
        ordered.append(&mut self.tabs);
        self.tabs = ordered;
        self.layout_all();
        self.send_state();
        self.save_session();
    }

    fn save_top_sites(&mut self, urls: Vec<String>) {
        self.prefs.top_sites = urls;
        self.prefs.sanitize();
        self.prefs.save();
        self.send_prefs();
        self.send_top_sites();
    }

    fn save_layout(&mut self, value: Option<&Value>) {
        let Some(value) = value else {
            return;
        };
        let Ok(layout) =
            serde_json::from_value::<std::collections::BTreeMap<String, Placement>>(value.clone())
        else {
            return;
        };
        self.prefs.layout = layout;
        self.prefs.sanitize();
        self.prefs.save();
        self.send_prefs();
    }

    fn choose_background(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Choose a background image"),
            Some(self.window.gtk_window()),
            gtk::FileChooserAction::Open,
        );
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Use image", gtk::ResponseType::Accept);
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Images"));
        filter.add_pixbuf_formats();
        dialog.add_filter(filter);
        dialog.set_modal(true);

        if dialog.run() == gtk::ResponseType::Accept
            && let Some(path) = dialog.filename()
            && let Ok(data) = encode_background(&path)
        {
            self.prefs.background = data;
            self.prefs.background_video.clear();
            self.prefs.save();
            self.send_prefs_full();
        }
        dialog.close();
    }

    fn choose_video(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Choose a background video"),
            Some(self.window.gtk_window()),
            gtk::FileChooserAction::Open,
        );
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Use video", gtk::ResponseType::Accept);
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Videos"));
        filter.add_mime_type("video/mp4");
        filter.add_mime_type("video/webm");
        filter.add_mime_type("video/ogg");
        dialog.add_filter(filter);
        dialog.set_modal(true);

        if dialog.run() == gtk::ResponseType::Accept
            && let Some(path) = dialog.filename()
            && let Ok(data) = encode_video(&path)
        {
            self.prefs.background_video = data;
            self.prefs.background.clear();
            self.prefs.save();
            self.send_prefs_full();
        }
        dialog.close();
    }

    fn clear_background(&mut self) {
        if self.prefs.background.is_empty() && self.prefs.background_video.is_empty() {
            return;
        }
        self.prefs.background.clear();
        self.prefs.background_video.clear();
        self.prefs.save();
        self.send_prefs_full();
    }

    fn choose_audio(&mut self) {
        let dialog = gtk::FileChooserDialog::new(
            Some("Choose background music"),
            Some(self.window.gtk_window()),
            gtk::FileChooserAction::Open,
        );
        dialog.add_button("Cancel", gtk::ResponseType::Cancel);
        dialog.add_button("Use audio", gtk::ResponseType::Accept);
        let filter = gtk::FileFilter::new();
        filter.set_name(Some("Audio"));
        filter.add_mime_type("audio/mpeg");
        filter.add_mime_type("audio/ogg");
        filter.add_mime_type("audio/wav");
        filter.add_mime_type("audio/mp4");
        filter.add_mime_type("audio/flac");
        dialog.add_filter(filter);
        dialog.set_modal(true);

        if dialog.run() == gtk::ResponseType::Accept
            && let Some(path) = dialog.filename()
            && let Ok(data) = encode_audio(&path)
        {
            self.prefs.audio = data;
            self.prefs.save();
            self.send_prefs_full();
        }
        dialog.close();
    }

    fn clear_audio(&mut self) {
        if self.prefs.audio.is_empty() {
            return;
        }
        self.prefs.audio.clear();
        self.prefs.save();
        self.send_prefs_full();
    }

    fn active_tab(&self) -> Option<&Tab> {
        self.tabs.iter().find(|tab| tab.id == self.active)
    }

    fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|tab| tab.id == self.active)
    }

    fn activate(&mut self, id: u64) {
        if !self.tabs.iter().any(|tab| tab.id == id) {
            return;
        }

        let previous = self.active;

        for tab in &self.tabs {
            let _ = tab.webview.set_visible(tab.id == id);
        }

        self.active = id;

        if previous != id {
            self.set_tab_active(previous, false);
            self.set_tab_active(id, true);
        }

        if let Some(tab) = self.active_tab() {
            let _ = tab.webview.focus();
        }

        self.send_state();
        self.update_window_title();
        self.save_session();
    }

    fn set_tab_active(&self, id: u64, active: bool) {
        if let Some(tab) = self.tabs.iter().find(|tab| tab.id == id)
            && !tab.external
            && tab.kind != TabKind::Web
        {
            let _ = tab.webview.evaluate_script(&format!(
                "window.mestySetTabActive && window.mestySetTabActive({active})"
            ));
        }
    }

    fn close_tab(&mut self, id: u64) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == id) else {
            return;
        };

        let was_active = self.active == id;
        let closing = &self.tabs[index];
        if !closing.incognito && closing.kind == TabKind::Web && !closing.url.is_empty() {
            self.closed.push((TabKind::Web, Some(closing.url.clone())));
            if self.closed.len() > 25 {
                self.closed.remove(0);
            }
        }
        self.tabs.remove(index);

        if self.tabs.is_empty() {
            let new_id = self.new_tab(TabKind::Start, None);
            self.activate(new_id);
            return;
        }

        if was_active {
            let next = self
                .tabs
                .get(index)
                .or_else(|| self.tabs.last())
                .map(|tab| tab.id);
            if let Some(next) = next {
                self.activate(next);
            }
        } else {
            self.send_state();
        }
        self.save_session();
    }

    fn new_start_tab(&mut self) {
        let id = self.new_tab(TabKind::Start, None);
        self.activate(id);
    }

    fn new_incognito_tab(&mut self) {
        let id = self.new_tab_with(TabKind::Start, None, true);
        self.activate(id);
    }

    fn open_settings(&mut self) {
        if let Some(tab) = self
            .tabs
            .iter()
            .find(|tab| tab.kind == TabKind::Settings && !tab.external)
        {
            let id = tab.id;
            self.activate(id);
            return;
        }

        let id = self.new_tab(TabKind::Settings, None);
        self.activate(id);
    }

    fn open_home_editor(&mut self) {
        if let Some(tab) = self.active_tab()
            && tab.kind == TabKind::Start
            && !tab.external
        {
            let _ = tab.webview.focus();
            let _ = tab
                .webview
                .evaluate_script("window.mestyToggleEditor && window.mestyToggleEditor()");
            return;
        }

        let incognito = self.active_tab().map(|tab| tab.incognito).unwrap_or(false);
        let id = self.new_tab_with(TabKind::Start, None, incognito);
        self.activate(id);
        self.pending_editor = Some(id);
    }

    fn reopen_tab(&mut self) {
        let Some((kind, url)) = self.closed.pop() else {
            return;
        };
        let id = self.new_tab(kind, url);
        self.activate(id);
    }

    fn cycle_tab(&mut self, step: isize) {
        if self.tabs.is_empty() {
            return;
        }
        let len = self.tabs.len() as isize;
        let index = self
            .tabs
            .iter()
            .position(|tab| tab.id == self.active)
            .unwrap_or(0) as isize;
        let next = (index + step).rem_euclid(len) as usize;
        let id = self.tabs[next].id;
        self.activate(id);
    }

    fn zoom(&mut self, action: &str) {
        let Some(index) = self.tabs.iter().position(|tab| tab.id == self.active) else {
            return;
        };
        let next = match action {
            "in" => (self.tabs[index].zoom + 0.1).min(3.0),
            "out" => (self.tabs[index].zoom - 0.1).max(0.4),
            _ => 1.0,
        };
        let zoom = (next * 10.0).round() / 10.0;
        self.tabs[index].zoom = zoom;
        self.tabs[index].webview.webview().set_zoom_level(zoom);
        self.send_state();
        self.save_session();
    }

    fn toggle_fullscreen(&self) {
        if self.window.fullscreen().is_some() {
            self.window.set_fullscreen(None);
        } else {
            self.window
                .set_fullscreen(Some(Fullscreen::Borderless(None)));
        }
    }

    fn set_panel_height(&mut self, height: f64) {
        self.chrome_height = height.clamp(CHROME_H, 560.0);
        self.layout_all();
    }

    fn find_in_page(&self, text: &str, backwards: bool) {
        let Some(tab) = self.active_tab() else {
            return;
        };
        let Ok(needle) = serde_json::to_string(text) else {
            return;
        };
        let _ = tab
            .webview
            .evaluate_script(&format!("window.find({needle}, false, {backwards}, true)"));
    }

    fn open_find(&mut self) {
        let _ = self.chrome.focus();
        let _ = self
            .chrome
            .evaluate_script("window.mestyOpenFind && window.mestyOpenFind()");
    }

    fn focus_address(&mut self) {
        let _ = self.chrome.focus();
        let _ = self
            .chrome
            .evaluate_script("window.mestyFocusAddress && window.mestyFocusAddress()");
    }

    fn navigate(&mut self, raw: &str) {
        let url = raw.trim();
        if url.is_empty() {
            return;
        }

        if url == "mesty://settings" {
            self.open_settings();
            return;
        }

        if url == "mesty://start" || url == "about:home" {
            self.new_start_tab();
            return;
        }

        let allowed =
            url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://");

        if !allowed {
            return;
        }

        if let Some(tab) = self.active_tab_mut() {
            tab.loading = true;
            let _ = tab.webview.load_url(url);
            let _ = tab.webview.focus();
        }

        self.send_state();
    }

    fn send_state(&self) {
        if !self.chrome_ready {
            return;
        }

        let tabs: Vec<Value> = self
            .tabs
            .iter()
            .map(|tab| {
                json!({
                    "id": tab.id,
                    "title": tab.title,
                    "url": tab.display_url(),
                    "kind": match tab.kind {
                        TabKind::Start => "start",
                        TabKind::Settings => "settings",
                        TabKind::Web => "web",
                    },
                    "loading": tab.loading,
                    "favicon": tab.favicon,
                    "zoom": tab.zoom,
                    "incognito": tab.incognito,
                })
            })
            .collect();

        let active_loading = self.active_tab().map(|tab| tab.loading).unwrap_or(false);
        let state = json!({
            "tabs": tabs,
            "active": self.active,
            "loading": active_loading,
        });

        let _ = self.chrome.evaluate_script(&format!(
            "window.mestySetState && window.mestySetState({state})"
        ));
    }

    fn send_prefs(&self) {
        self.send_prefs_inner(false);
    }

    fn send_prefs_full(&self) {
        self.send_prefs_inner(true);
    }

    fn send_prefs_inner(&self, with_media: bool) {
        let Some(light_json) = self.prefs.to_light_json() else {
            return;
        };
        let full_json = if with_media {
            serde_json::to_string(&self.prefs).ok()
        } else {
            None
        };
        let tab_json = full_json.as_deref().unwrap_or(&light_json);

        if self.chrome_ready {
            let _ = self.chrome.evaluate_script(&format!(
                "window.mestySetPrefs && window.mestySetPrefs({light_json})"
            ));
        }

        for tab in &self.tabs {
            if !tab.external && tab.kind != TabKind::Web {
                let _ = tab.webview.evaluate_script(&format!(
                    "window.mestySetPrefs && window.mestySetPrefs({tab_json})"
                ));
            }
        }
    }

    fn apply_prefs(&mut self, value: Option<&Value>) {
        let Some(value) = value else {
            return;
        };
        let Ok(mut prefs) = serde_json::from_value::<Prefs>(value.clone()) else {
            return;
        };
        prefs.sanitize();
        prefs.save();
        let media_changed = self.prefs.background != prefs.background
            || self.prefs.background_video != prefs.background_video
            || self.prefs.audio != prefs.audio;
        self.prefs = prefs;
        if media_changed {
            self.send_prefs_full();
        } else {
            self.send_prefs();
        }
        self.apply_languages();
        self.send_top_sites();
    }

    fn update_window_title(&self) {
        let Some(tab) = self.active_tab() else {
            self.window.set_title("MestyBrovser");
            return;
        };

        if (tab.external || tab.kind == TabKind::Web) && !tab.title.is_empty() {
            let suffix = if tab.incognito {
                "Incognito — MestyBrovser"
            } else {
                "MestyBrovser"
            };
            self.window
                .set_title(&format!("{} — {}", tab.title, suffix));
        } else if tab.incognito {
            self.window.set_title("Incognito — MestyBrovser");
        } else {
            self.window.set_title("MestyBrovser");
        }
    }

    fn handle_chrome(&mut self, body: &str) {
        let Ok(message) = serde_json::from_str::<Value>(body) else {
            return;
        };

        match message.get("cmd").and_then(Value::as_str).unwrap_or("") {
            "ready" => {
                self.chrome_ready = true;
                self.layout_all();
                self.send_prefs();
                self.send_state();
            }
            "navigate" => {
                if let Some(url) = message.get("url").and_then(Value::as_str) {
                    self.navigate(url);
                }
            }
            "new_tab" => self.new_start_tab(),
            "new_incognito" => self.new_incognito_tab(),
            "home" => self.new_start_tab(),
            "open_settings" => self.open_settings(),
            "home_editor" => self.open_home_editor(),
            "switch_tab" => {
                if let Some(id) = message.get("id").and_then(Value::as_u64) {
                    self.activate(id);
                }
            }
            "close_tab" => {
                if let Some(id) = message.get("id").and_then(Value::as_u64) {
                    self.close_tab(id);
                }
            }
            "close_active" => {
                let id = self.active;
                self.close_tab(id);
            }
            "back" => {
                if let Some(tab) = self.active_tab() {
                    let _ = tab.webview.go_back();
                }
            }
            "forward" => {
                if let Some(tab) = self.active_tab() {
                    let _ = tab.webview.go_forward();
                }
            }
            "reload" => {
                if let Some(tab) = self.active_tab() {
                    let _ = tab.webview.reload();
                }
            }
            "focus_address" => self.focus_address(),
            "save_prefs" => self.apply_prefs(message.get("prefs")),
            "zoom" => {
                if let Some(action) = message.get("action").and_then(Value::as_str) {
                    self.zoom(action);
                }
            }
            "fullscreen" => self.toggle_fullscreen(),
            "reopen_tab" => self.reopen_tab(),
            "panel" => {
                if let Some(height) = message.get("height").and_then(Value::as_f64) {
                    self.set_panel_height(height);
                }
            }
            "find" => {
                let text = message.get("text").and_then(Value::as_str).unwrap_or("");
                let backwards = message
                    .get("backwards")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                self.find_in_page(text, backwards);
            }
            "reorder_tabs" => {
                let ids: Vec<u64> = message
                    .get("ids")
                    .and_then(Value::as_array)
                    .map(|ids| ids.iter().filter_map(Value::as_u64).collect())
                    .unwrap_or_default();
                if !ids.is_empty() {
                    self.reorder_tabs(&ids);
                }
            }
            _ => {}
        }
    }

    fn handle_content(&mut self, tab_id: u64, body: &str) {
        let Ok(message) = serde_json::from_str::<Value>(body) else {
            return;
        };

        match message.get("cmd").and_then(Value::as_str).unwrap_or("") {
            "new_tab" => self.new_start_tab(),
            "new_incognito" => self.new_incognito_tab(),
            "close_active" => self.close_tab(tab_id),
            "focus_address" => self.focus_address(),
            "open_settings" => self.open_settings(),
            "home_editor" => self.open_home_editor(),
            "save_prefs" => self.apply_prefs(message.get("prefs")),
            "zoom_in" => self.zoom("in"),
            "zoom_out" => self.zoom("out"),
            "zoom_reset" => self.zoom("reset"),
            "fullscreen" => self.toggle_fullscreen(),
            "reopen_tab" => self.reopen_tab(),
            "open_find" => self.open_find(),
            "next_tab" => self.cycle_tab(1),
            "prev_tab" => self.cycle_tab(-1),
            "ready" => {
                self.send_top_sites();
                if self.pending_editor == Some(tab_id) {
                    self.pending_editor = None;
                    if let Some(tab) = self.tabs.iter().find(|tab| tab.id == tab_id) {
                        let _ = tab.webview.focus();
                        let _ = tab.webview.evaluate_script(
                            "window.mestyToggleEditor && window.mestyToggleEditor()",
                        );
                    }
                }
            }
            "top_sites" => self.send_top_sites(),
            "save_top_sites" | "reorder_top_sites" => {
                let urls: Vec<String> = message
                    .get("urls")
                    .and_then(Value::as_array)
                    .map(|urls| {
                        urls.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default();
                self.save_top_sites(urls);
            }
            "save_layout" => self.save_layout(message.get("layout")),
            "choose_background" => self.choose_background(),
            "choose_video" => self.choose_video(),
            "clear_background" => self.clear_background(),
            "choose_audio" => self.choose_audio(),
            "clear_audio" => self.clear_audio(),
            _ => {}
        }
    }

    fn handle_load(&mut self, tab_id: u64, event: PageLoadEvent, raw_url: String) {
        let mut is_internal = false;
        let mut visited = None;
        let mut url_changed = false;
        let mut incognito = false;
        let finished = matches!(event, PageLoadEvent::Finished);

        if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == tab_id) {
            match event {
                PageLoadEvent::Started => tab.loading = true,
                PageLoadEvent::Finished => {
                    tab.loading = false;
                    incognito = tab.incognito;
                    if raw_url.starts_with("http://")
                        || raw_url.starts_with("https://")
                        || raw_url.starts_with("file://")
                    {
                        if tab.url != raw_url {
                            url_changed = true;
                        }
                        tab.external = true;
                        tab.url = raw_url.clone();
                        if !tab.incognito
                            && (raw_url.starts_with("http://") || raw_url.starts_with("https://"))
                        {
                            visited =
                                Some((raw_url.clone(), tab.title.clone(), tab.favicon.clone()));
                        }
                    } else {
                        tab.external = false;
                        is_internal = true;
                    }
                }
            }
        }

        if finished && url_changed {
            self.save_session();
        }

        if let Some((url, title, favicon)) = visited
            && !incognito
        {
            self.history.record(&url, &title, &favicon);
            self.send_top_sites();
        }

        if is_internal
            && let Ok(prefs_json) = serde_json::to_string(&self.prefs)
            && let Some(tab) = self.tabs.iter().find(|tab| tab.id == tab_id)
            && !tab.external
            && tab.kind != TabKind::Web
        {
            let active = self.active == tab_id;
            let incognito = tab.incognito;
            let _ = tab.webview.evaluate_script(&format!(
                "window.mestySetPrefs && window.mestySetPrefs({prefs_json});\n\
                 window.mestySetTabActive && window.mestySetTabActive({active});\n\
                 window.mestySetIncognito && window.mestySetIncognito({incognito})"
            ));
        }

        if self.active == tab_id {
            self.send_state();
            self.update_window_title();
        }
    }

    fn handle(&mut self, event: UserEvent) {
        match event {
            UserEvent::Chrome(body) => self.handle_chrome(&body),
            UserEvent::Content(id, body) => self.handle_content(id, &body),
            UserEvent::Title(id, title) => {
                if title.trim().is_empty() {
                    return;
                }
                let mut meta = None;
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id) {
                    tab.title = title;
                    if !tab.incognito
                        && tab.external
                        && (tab.url.starts_with("http://") || tab.url.starts_with("https://"))
                    {
                        meta = Some((tab.url.clone(), tab.title.clone(), tab.favicon.clone()));
                    }
                }
                if let Some((url, title, favicon)) = meta {
                    self.history.update_meta(&url, &title, &favicon);
                    self.send_top_sites();
                }
                if self.active == id {
                    self.send_state();
                    self.update_window_title();
                }
            }
            UserEvent::Load(id, event, url) => self.handle_load(id, event, url),
            UserEvent::Favicon(id, favicon) => {
                let mut meta = None;
                if let Some(tab) = self.tabs.iter_mut().find(|tab| tab.id == id)
                    && tab.favicon != favicon
                {
                    tab.favicon = favicon;
                    if !tab.incognito
                        && tab.external
                        && (tab.url.starts_with("http://") || tab.url.starts_with("https://"))
                    {
                        meta = Some((tab.url.clone(), tab.title.clone(), tab.favicon.clone()));
                    }
                    if self.active == id {
                        self.send_state();
                    }
                }
                if let Some((url, title, favicon)) = meta {
                    self.history.update_meta(&url, &title, &favicon);
                    self.send_top_sites();
                }
            }
            UserEvent::OpenTab(source, url) => {
                if url.starts_with("http://") || url.starts_with("https://") {
                    let incognito = self
                        .tabs
                        .iter()
                        .find(|tab| tab.id == source)
                        .map(|tab| tab.incognito)
                        .unwrap_or(false);
                    let id = self.new_tab_with(TabKind::Web, Some(url), incognito);
                    self.activate(id);
                }
            }
        }
    }
}

fn build_chrome(
    fixed: &gtk::Fixed,
    window: &Window,
    proxy: EventLoopProxy<UserEvent>,
) -> wry::Result<WebView> {
    let size = window.inner_size();
    let scale = window.scale_factor();
    let height = (CHROME_H * scale).round().max(1.0) as i32;
    let width = size.width.max(1) as i32;

    let bounds = Rect {
        position: PhysicalPosition::new(0, 0).into(),
        size: PhysicalSize::new(width, height).into(),
    };

    WebViewBuilder::new()
        .with_bounds(bounds)
        .with_html(chrome_html())
        .with_focused(false)
        .with_ipc_handler(move |request| {
            let _ = proxy.send_event(UserEvent::Chrome(request.body().clone()));
        })
        .build_gtk(fixed)
}

fn page_html(template: &str) -> String {
    template
        .replace("/*__THEME_JS__*/", THEME_JS)
        .replace("/*__I18N__*/", I18N_JS)
        .replace("__LOGO__", LOGO_DATA_URI)
}

fn chrome_html() -> String {
    include_str!("../ui/chrome.html")
        .replace("/*__CHROME_CSS__*/", include_str!("../ui/chrome.css"))
        .replace("/*__CHROME_JS__*/", include_str!("../ui/chrome.js"))
        .replace("/*__I18N__*/", I18N_JS)
        .replace("__LOGO__", LOGO_DATA_URI)
}

fn preferred_languages(language: &str) -> Vec<&'static str> {
    match language {
        "ru" => vec!["ru-RU", "ru", "en-US", "en"],
        "uk" => vec!["uk-UA", "uk", "ru", "en-US", "en"],
        "de" => vec!["de-DE", "de", "en-US", "en"],
        "fr" => vec!["fr-FR", "fr", "en-US", "en"],
        "es" => vec!["es-ES", "es", "en-US", "en"],
        "it" => vec!["it-IT", "it", "en-US", "en"],
        "pt" => vec!["pt-BR", "pt-PT", "pt", "en-US", "en"],
        "pl" => vec!["pl-PL", "pl", "en-US", "en"],
        "tr" => vec!["tr-TR", "tr", "en-US", "en"],
        "cs" => vec!["cs-CZ", "cs", "en-US", "en"],
        "nl" => vec!["nl-NL", "nl", "en-US", "en"],
        "zh" => vec!["zh-CN", "zh", "en-US", "en"],
        "ja" => vec!["ja-JP", "ja", "en-US", "en"],
        _ => vec!["en-US", "en"],
    }
}

fn encode_background(path: &std::path::Path) -> Result<String, String> {
    let image = image::open(path).map_err(|error| error.to_string())?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 {
        return Err("empty image".into());
    }

    let ratio = (1920.0 / width as f64).min(1200.0 / height as f64).min(1.0);
    let image = if ratio < 1.0 {
        image.resize(
            (width as f64 * ratio).round() as u32,
            (height as f64 * ratio).round() as u32,
            image::imageops::FilterType::CatmullRom,
        )
    } else {
        image
    };

    let rgb = image.to_rgb8();
    let mut bytes = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 82);
    encoder
        .encode_image(&rgb)
        .map_err(|error| error.to_string())?;

    if bytes.len() > 6_500_000 {
        return Err("image is too large".into());
    }

    Ok(format!(
        "data:image/jpeg;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

fn encode_video(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Err("empty video file".into());
    }
    if bytes.len() > 8_000_000 {
        return Err("video file is too large".into());
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match extension.as_str() {
        "webm" => "video/webm",
        "ogv" | "ogg" => "video/ogg",
        "mp4" | "m4v" => "video/mp4",
        _ => return Err("unsupported video format".into()),
    };

    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

fn encode_audio(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Err("empty audio file".into());
    }
    if bytes.len() > 8_000_000 {
        return Err("audio file is too large".into());
    }

    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    let mime = match extension.as_str() {
        "ogg" | "oga" => "audio/ogg",
        "wav" => "audio/wav",
        "m4a" | "mp4" | "aac" => "audio/mp4",
        "flac" => "audio/flac",
        "opus" => "audio/opus",
        _ => "audio/mpeg",
    };

    Ok(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

fn load_icon() -> Option<Icon> {
    let bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    Icon::from_rgba(image.into_raw(), width, height).ok()
}

fn main() {
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event()
        .with_app_id("com.mestybrovser.MestyBrovser")
        .build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title("MestyBrovser")
        .with_inner_size(LogicalSize::new(1320.0, 860.0))
        .with_min_inner_size(LogicalSize::new(760.0, 480.0))
        .with_window_icon(load_icon())
        .build(&event_loop)
        .expect("failed to create window");

    gtk::glib::set_prgname(Some("mestybrovser"));
    gtk::glib::set_application_name("MestyBrovser");
    gtk::Window::set_default_icon_name("mestybrovser");

    let vbox = window
        .default_vbox()
        .expect("tao default vbox is not available");
    let fixed = gtk::Fixed::new();
    vbox.pack_start(&fixed, true, true, 0);
    fixed.show_all();

    let chrome = build_chrome(&fixed, &window, proxy.clone()).expect("failed to build chrome");

    let mut app = App::new(window, fixed, chrome, proxy);
    let initial_url = std::env::args()
        .skip(1)
        .find(|arg| !arg.starts_with('-'))
        .map(|arg| {
            if arg.contains("://") || arg.starts_with("file:") {
                arg
            } else {
                format!("https://{arg}")
            }
        });
    app.restore_session();
    if let Some(url) = initial_url {
        let id = app.new_tab(TabKind::Web, Some(url));
        app.activate(id);
    } else if app.tabs.is_empty() {
        let id = app.new_tab(TabKind::Start, None);
        app.activate(id);
    }
    app.finish_restore();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                    app.layout_all();
                }
                WindowEvent::CloseRequested => {
                    app.save_session();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            Event::UserEvent(user_event) => app.handle(user_event),
            _ => {}
        }
    });
}
