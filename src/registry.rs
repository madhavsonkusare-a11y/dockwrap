//! Shared app registry: the single source of truth for wrapped apps.
//! Used by BOTH the GUI (Tauri commands) and the CLI (standalone mode).

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone, Serialize, Default)]
pub struct AppDef {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    /// Optional path to a docker-compose.yml (or a directory containing one).
    /// When set, opening the app runs `docker compose up -d` first.
    #[serde(default)]
    pub compose: Option<String>,
    /// Optional health URL or host:port to poll after compose boot.
    /// Defaults to `url` when omitted.
    #[serde(default)]
    pub health: Option<String>,
}

/// Curated, most-used self-hosted web apps (default localhost ports).
/// Lets a user run `dockwrap add --preset n8n` instead of typing the URL.
pub const PRESETS: &[(&str, &str)] = &[
    ("n8n", "http://localhost:5678"),
    ("open-webui", "http://localhost:3000"),
    ("immich", "http://localhost:2283"),
    ("stirling-pdf", "http://localhost:8080"),
    ("uptime-kuma", "http://localhost:3001"),
    ("memos", "http://localhost:5230"),
    ("coolify", "http://localhost:8000"),
    ("glance", "http://localhost:8080"),
    ("filebrowser", "http://localhost:80"),
    ("changedetection", "http://localhost:5000"),
];

/// Where registered apps live. Same location on every platform via APPDATA
/// (Windows) or XDG/HOME fallback on Linux/macOS.
pub fn resolve_config() -> String {
    let base = std::env::var("APPDATA")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| {
            std::env::var("XDG_CONFIG_HOME")
                .ok()
                .filter(|v| !v.is_empty())
        })
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| format!("{}/.config", h))
        })
        .unwrap_or_else(|| ".".to_string());
    format!("{}\\dockwrap\\apps.json", base)
}

pub fn load_apps() -> Vec<AppDef> {
    let data = std::fs::read_to_string(resolve_config()).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_default()
}

pub fn save_apps(apps: &[AppDef]) {
    let path = resolve_config();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, serde_json::to_string(apps).unwrap());
}

/// Insert or replace an app by name (dedup on name).
pub fn upsert_app(
    name: &str,
    url: &str,
    icon: Option<String>,
    compose: Option<String>,
    health: Option<String>,
) {
    let mut apps = load_apps();
    apps.retain(|a| a.name != name);
    apps.push(AppDef {
        name: name.to_string(),
        url: url.to_string(),
        icon,
        compose,
        health,
    });
    save_apps(&apps);
}

pub fn remove_app(name: &str) -> bool {
    let mut apps = load_apps();
    let before = apps.len();
    apps.retain(|a| a.name != name);
    if apps.len() != before {
        save_apps(&apps);
        true
    } else {
        false
    }
}

pub fn preset_url(name: &str) -> Option<&'static str> {
    PRESETS
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, u)| *u)
}
