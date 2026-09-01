//! Registry storage: where registered apps live and how the apps.json file is
//! loaded and saved. The single source of truth for wrapped apps, used by BOTH
//! the GUI (Tauri commands) and the CLI (standalone mode).

use crate::model::AppDef;
use std::path::PathBuf;

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
        .or_else(|| std::env::var("HOME").ok().map(|h| format!("{}/.config", h)))
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
        apps.push(AppDef {
            name: "x".into(),
            url: "http://a".into(),
            icon: None,
            compose: None,
            health: None,
        });
        apps.push(AppDef {
            name: "x".into(),
            url: "http://b".into(),
            icon: None,
            compose: None,
            health: None,
        });
        apps.retain(|a| a.name != "x");
        apps.push(AppDef {
            name: "x".into(),
            url: "http://c".into(),
            icon: None,
            compose: None,
            health: None,
        });
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].url, "http://c");
    }
}
