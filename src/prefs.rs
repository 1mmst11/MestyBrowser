use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const THEMES: &[&str] = &["paper", "mono", "slate", "sand", "custom"];
const WALLPAPERS: &[&str] = &[
    "none", "fog", "glow", "dawn", "dusk", "ocean", "forest", "sunset", "aurora", "lavender",
];
const ENGINES: &[&str] = &["duckduckgo", "google", "bing", "brave", "startpage"];

pub const LANGUAGES: &[&str] = &[
    "en", "ru", "uk", "de", "fr", "es", "it", "pt", "pl", "tr", "cs", "nl", "zh", "ja",
];

const LAYOUT_KEYS: &[&str] = &["hero", "search", "frequent", "footer"];

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Placement {
    pub x: f64,
    pub y: f64,
    pub r: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CustomTheme {
    pub base: String,
    pub surface: String,
    pub ink: String,
    pub muted: String,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            base: "#f6f7f9".into(),
            surface: "#ffffff".into(),
            ink: "#1d1f23".into(),
            muted: "#71757c".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Prefs {
    pub theme: String,
    pub accent: String,
    pub custom_theme: CustomTheme,
    pub wallpaper: String,
    pub compact_tabs: bool,
    pub animations: bool,
    pub rounded: bool,
    pub search_engine: String,
    pub language: String,
    pub background: String,
    pub background_video: String,
    pub top_sites: Vec<String>,
    pub layout: BTreeMap<String, Placement>,
    pub audio: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: "paper".into(),
            accent: "#9a3f8c".into(),
            custom_theme: CustomTheme::default(),
            wallpaper: "none".into(),
            compact_tabs: false,
            animations: true,
            rounded: true,
            search_engine: "google".into(),
            language: "en".into(),
            background: String::new(),
            background_video: String::new(),
            top_sites: Vec::new(),
            layout: BTreeMap::new(),
            audio: String::new(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LightPrefs<'a> {
    theme: &'a str,
    accent: &'a str,
    custom_theme: &'a CustomTheme,
    wallpaper: &'a str,
    compact_tabs: bool,
    animations: bool,
    rounded: bool,
    search_engine: &'a str,
    language: &'a str,
    top_sites: &'a [String],
    layout: &'a BTreeMap<String, Placement>,
}

impl Prefs {
    pub fn to_light_json(&self) -> Option<String> {
        serde_json::to_string(&LightPrefs {
            theme: &self.theme,
            accent: &self.accent,
            custom_theme: &self.custom_theme,
            wallpaper: &self.wallpaper,
            compact_tabs: self.compact_tabs,
            animations: self.animations,
            rounded: self.rounded,
            search_engine: &self.search_engine,
            language: &self.language,
            top_sites: &self.top_sites,
            layout: &self.layout,
        })
        .ok()
    }

    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mestybrovser")
            .join("prefs.json")
    }

    pub fn load() -> Self {
        let mut prefs = fs::read_to_string(Self::config_path())
            .ok()
            .and_then(|raw| serde_json::from_str::<Prefs>(&raw).ok())
            .unwrap_or_default();
        prefs.sanitize();
        prefs
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn sanitize(&mut self) {
        if !THEMES.contains(&self.theme.as_str()) {
            self.theme = "paper".into();
        }
        if !WALLPAPERS.contains(&self.wallpaper.as_str()) {
            self.wallpaper = "none".into();
        }
        if !ENGINES.contains(&self.search_engine.as_str()) {
            self.search_engine = "google".into();
        }
        if !LANGUAGES.contains(&self.language.as_str()) {
            self.language = "en".into();
        }
        if !is_hex_color(&self.accent) {
            self.accent = "#9a3f8c".into();
        }
        for (value, fallback) in [
            (&mut self.custom_theme.base, "#f6f7f9"),
            (&mut self.custom_theme.surface, "#ffffff"),
            (&mut self.custom_theme.ink, "#1d1f23"),
            (&mut self.custom_theme.muted, "#71757c"),
        ] {
            if !is_hex_color(value) {
                *value = fallback.into();
            }
        }
        if !self.background.is_empty()
            && (!self.background.starts_with("data:image/") || self.background.len() > 8_000_000)
        {
            self.background.clear();
        }
        if !self.background_video.is_empty()
            && (!(self.background_video.starts_with("data:video/mp4;base64,")
                || self.background_video.starts_with("data:video/webm;base64,")
                || self.background_video.starts_with("data:video/ogg;base64,"))
                || self.background_video.len() > 12_000_000)
        {
            self.background_video.clear();
        }
        if !self.audio.is_empty()
            && (!self.audio.starts_with("data:audio/") || self.audio.len() > 12_000_000)
        {
            self.audio.clear();
        }
        let mut seen = std::collections::HashSet::new();
        self.top_sites.retain(|url| {
            url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://")
        });
        self.top_sites.retain(|url| seen.insert(url.clone()));
        self.top_sites.truncate(24);
        self.layout
            .retain(|key, _| LAYOUT_KEYS.contains(&key.as_str()));
        for placement in self.layout.values_mut() {
            if !placement.x.is_finite() {
                placement.x = 0.0;
            }
            if !placement.y.is_finite() {
                placement.y = 0.0;
            }
            if !placement.r.is_finite() {
                placement.r = 0.0;
            }
            placement.x = (placement.x.clamp(-3000.0, 3000.0) * 10.0).round() / 10.0;
            placement.y = (placement.y.clamp(-3000.0, 3000.0) * 10.0).round() / 10.0;
            placement.r = (placement.r.clamp(-180.0, 180.0) * 10.0).round() / 10.0;
        }
    }
}

fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    let valid_len = bytes.len() == 4 || bytes.len() == 7;
    valid_len && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}
