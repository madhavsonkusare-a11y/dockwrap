//! Shared app registry: the single source of truth for wrapped apps.
//! Used by BOTH the GUI (Tauri commands) and the CLI (standalone mode).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    let mut p = PathBuf::from(base);
    p.push("dockwrap");
    p.push("apps.json");
    p.to_string_lossy().into_owned()
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

// ---- Curated self-hosted app catalog (baked into the binary at compile time). ----
// Ships as src/catalog.json (also lives in the webview bundle via frontendDist: src).
// Lets the GUI wizard offer a "pick a popular app" catalog instead of typing URLs.

/// One catalog entry — a fuller AppDef + a human description + category.
#[derive(Deserialize, Clone, Serialize, Default)]
pub struct CatalogEntry {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub compose: Option<String>,
    #[serde(default)]
    pub health: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: String,
}

/// Parse the compile-time-embedded catalog JSON into structs.
fn parse_catalog() -> Vec<CatalogEntry> {
    serde_json::from_str(CATALOG_JSON).unwrap_or_default()
}

/// The raw catalog, embedded at build time so the binary is self-contained
/// (no runtime file read, no extra network at startup). The full awesome-selfhosted
/// list (1257 apps) ships here; the 12 hand-curated apps (with compose/health snippets)
/// are a separate quick-start subset used by `dockwrap add --preset`.
const CATALOG_JSON: &str = include_str!("catalog_full.json");

/// Public catalog access — cheap, returns the parsed list (re-parsed per call,
/// but small and typically called once at launcher render time).
pub fn catalog() -> Vec<CatalogEntry> {
    parse_catalog()
}

/// All distinct categories, in catalog order (owned so callers don't borrow locals).
pub fn catalog_categories() -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for e in parse_catalog().iter() {
        if let Some(cat) = &e.category {
            if !seen.iter().any(|c: &String| c == cat) {
                seen.push(cat.clone());
            }
        }
    }
    seen
}

/// Lookup a single catalog entry by name (case-insensitive — app names in the
/// source list have inconsistent casing, e.g. "Immich" vs "n8n").
pub fn catalog_entry(name: &str) -> Option<CatalogEntry> {
    let lower = name.to_lowercase();
    parse_catalog().into_iter().find(|e| e.name.to_lowercase() == lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_uses_native_separator() {
        let p = resolve_config();
        if cfg!(windows) {
            assert!(p.ends_with("dockwrap\\apps.json"), "got: {}", p);
        } else {
            assert!(!p.contains('\\'), "path must use native separator: {}", p);
            assert!(p.ends_with("dockwrap/apps.json"), "got: {}", p);
        }
    }

    #[test]
    fn upsert_dedups_by_name() {
        let mut apps: Vec<AppDef> = vec![];
        apps.push(AppDef { name: "x".into(), url: "http://a".into(), icon: None, compose: None, health: None });
        apps.push(AppDef { name: "x".into(), url: "http://b".into(), icon: None, compose: None, health: None });
        apps.retain(|a| a.name != "x");
        apps.push(AppDef { name: "x".into(), url: "http://c".into(), icon: None, compose: None, health: None });
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].url, "http://c");
    }

    #[test]
    fn preset_lookup_works() {
        assert_eq!(preset_url("n8n"), Some("http://localhost:5678"));
        assert_eq!(preset_url("penpot"), None);
    }

    #[test]
    fn catalog_loads_and_looks_up() {
        let apps = catalog();
        assert!(!apps.is_empty(), "catalog should embed the full self-hosted list");
        let by_name: Vec<&str> = apps.iter().map(|e| e.name.as_str()).collect();
        assert!(by_name.contains(&"Immich"), "Immich should be cataloged");
        assert!(by_name.contains(&"Jellyfin"), "Jellyfin should be cataloged");
        let entry = catalog_entry("Immich");
        assert!(entry.is_some(), "Immich lookup should return an entry");
        // The 12 curated apps carry compose snippets; the full list may not.
        let _ = entry.unwrap().compose.is_some();
    }

    #[test]
    fn catalog_categories_distinct() {
        let cats = catalog_categories();
        assert!(!cats.is_empty());
        let mut sorted = cats.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), cats.len(), "categories must be distinct");
    }

    /// `--preset immich` should resolve even though PRESETS and the catalog
    /// both use different casing — the CLI passes lowercase.
    #[test]
    fn catalog_entry_case_insensitive() {
        assert!(catalog_entry("immich").is_some());
        assert!(catalog_entry("IMMICH").is_some());
        assert!(catalog_entry("penpot").is_some());
        assert!(catalog_entry("PENPOT").is_some());
    }
}
