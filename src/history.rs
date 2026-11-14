use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

const MAX_ENTRIES: usize = 400;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Site {
    pub url: String,
    pub title: String,
    pub favicon: String,
    pub visits: u64,
    pub last: u64,
}

#[derive(Debug, Default)]
pub struct History {
    entries: HashMap<String, Site>,
    path: PathBuf,
}

impl History {
    pub fn load() -> Self {
        let path = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mestybrovser")
            .join("history.json");

        let mut entries = HashMap::new();
        if let Ok(raw) = fs::read_to_string(&path)
            && let Ok(saved) = serde_json::from_str::<Vec<Site>>(&raw)
        {
            for mut site in saved {
                site.url = normalize(&site.url);
                if !site.url.is_empty() {
                    entries.insert(site.url.clone(), site);
                }
            }
        }

        Self { entries, path }
    }

    pub fn record(&mut self, url: &str, title: &str, favicon: &str) {
        let url = normalize(url);
        if url.is_empty() {
            return;
        }

        let now = now();
        let entry = self.entries.entry(url.clone()).or_insert_with(|| Site {
            url,
            ..Site::default()
        });
        entry.visits += 1;
        entry.last = now;
        if !title.trim().is_empty() {
            entry.title = title.trim().to_string();
        }
        if !favicon.is_empty() {
            entry.favicon = favicon.to_string();
        }

        if self.entries.len() > MAX_ENTRIES {
            let mut ranked: Vec<(String, u64, u64)> = self
                .entries
                .iter()
                .map(|(url, site)| (url.clone(), site.visits, site.last))
                .collect();
            ranked.sort_by_key(|(_, visits, last)| (*visits, *last));
            for (url, _, _) in ranked.into_iter().take(self.entries.len() - MAX_ENTRIES) {
                self.entries.remove(&url);
            }
        }

        self.save();
    }

    pub fn update_meta(&mut self, url: &str, title: &str, favicon: &str) {
        let url = normalize(url);
        let Some(entry) = self.entries.get_mut(&url) else {
            return;
        };

        let mut changed = false;
        if !title.trim().is_empty() && entry.title != title.trim() {
            entry.title = title.trim().to_string();
            changed = true;
        }
        if !favicon.is_empty() && entry.favicon != favicon {
            entry.favicon = favicon.to_string();
            changed = true;
        }
        if changed {
            self.save();
        }
    }

    pub fn top(&self, custom_order: &[String], limit: usize) -> Vec<Site> {
        let mut result = Vec::with_capacity(limit);
        let mut used = HashSet::new();

        for url in custom_order {
            if result.len() >= limit {
                break;
            }
            let url = normalize(url);
            if url.is_empty() || !used.insert(url.clone()) {
                continue;
            }
            if let Some(site) = self.entries.get(&url) {
                result.push(site.clone());
            } else if url.starts_with("http://") || url.starts_with("https://") {
                result.push(Site {
                    url: url.clone(),
                    ..Site::default()
                });
            }
        }

        let mut rest: Vec<&Site> = self
            .entries
            .values()
            .filter(|site| !used.contains(&site.url))
            .collect();
        rest.sort_by(|a, b| b.visits.cmp(&a.visits).then(b.last.cmp(&a.last)));

        for site in rest {
            if result.len() >= limit {
                break;
            }
            result.push(site.clone());
        }

        result
    }

    fn save(&self) {
        let Some(dir) = self.path.parent() else {
            return;
        };
        let _ = fs::create_dir_all(dir);
        let mut sites: Vec<&Site> = self.entries.values().collect();
        sites.sort_by(|a, b| b.visits.cmp(&a.visits).then(b.last.cmp(&a.last)));
        if let Ok(json) = serde_json::to_string(&sites) {
            let _ = fs::write(&self.path, json);
        }
    }
}

fn normalize(url: &str) -> String {
    let url = url.trim();
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return String::new();
    }

    let mut url = url.split('#').next().unwrap_or(url).to_string();
    if let Some(scheme) = url.find("://") {
        let rest = &url[scheme + 3..];
        if let Some(slash) = rest.find('/')
            && slash + 1 == rest.len()
        {
            url.pop();
        }
    }
    url
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
