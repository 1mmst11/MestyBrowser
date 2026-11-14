use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const MAX_TABS: usize = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SessionTab {
    pub kind: String,
    pub url: String,
    pub zoom: f64,
}

impl Default for SessionTab {
    fn default() -> Self {
        Self {
            kind: "start".into(),
            url: String::new(),
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Session {
    pub tabs: Vec<SessionTab>,
    pub active: usize,
}

impl Session {
    pub fn path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mestybrovser")
            .join("session.json")
    }

    pub fn load() -> Self {
        let mut session = fs::read_to_string(Self::path())
            .ok()
            .and_then(|raw| serde_json::from_str::<Session>(&raw).ok())
            .unwrap_or_default();
        session.sanitize();
        session
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn sanitize(&mut self) {
        self.tabs.truncate(MAX_TABS);

        for tab in &mut self.tabs {
            tab.url = tab.url.trim().to_string();
            tab.kind = match tab.kind.as_str() {
                "settings" => "settings".into(),
                "web" => "web".into(),
                _ => "start".into(),
            };
            if tab.kind == "web" {
                if !(tab.url.starts_with("http://")
                    || tab.url.starts_with("https://")
                    || tab.url.starts_with("file://"))
                {
                    tab.kind = "start".into();
                    tab.url.clear();
                }
            } else {
                tab.url.clear();
            }
            if !tab.zoom.is_finite() {
                tab.zoom = 1.0;
            }
            tab.zoom = tab.zoom.clamp(0.4, 3.0);
            tab.zoom = (tab.zoom * 10.0).round() / 10.0;
        }

        if self.active >= self.tabs.len() {
            self.active = self.tabs.len().saturating_sub(1);
        }
    }
}
