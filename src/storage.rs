//! Registry storage: where registered apps live and how the apps.json file is
//! loaded and saved. The single source of truth for wrapped apps, used by BOTH
//! the GUI (Tauri commands) and the CLI (standalone mode).

use crate::{
    brand::{CONFIG_SLUG, LEGACY_CONFIG_SLUG},
    model::AppDef,
};
use std::path::PathBuf;

/// Where registered apps live. Same location on every platform via APPDATA
/// (Windows) or XDG/HOME fallback on Linux/macOS.
fn config_path(slug: &str) -> PathBuf {
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
    p.push(slug);
    p.push("apps.json");
    p
}

pub fn resolve_config() -> String {
    config_path(CONFIG_SLUG).to_string_lossy().into_owned()
}

fn legacy_config() -> String {
    config_path(LEGACY_CONFIG_SLUG)
        .to_string_lossy()
        .into_owned()
}

pub fn load_apps() -> Vec<AppDef> {
    let primary = PathBuf::from(resolve_config());
    let legacy = PathBuf::from(legacy_config());
    load_apps_from_paths(&primary, &legacy)
}

fn load_apps_from_paths(primary: &std::path::Path, legacy: &std::path::Path) -> Vec<AppDef> {
    match std::fs::read_to_string(primary) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            std::fs::read_to_string(legacy)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        }
        Err(_) => Vec::new(),
    }
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
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn fixture_paths() -> (PathBuf, PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!(
            "local-store-storage-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        (
            root.clone(),
            root.join("primary.json"),
            root.join("legacy.json"),
        )
    }

    fn app_json(name: &str) -> String {
        format!(
            r#"[{{"name":"{name}","url":"http://{name}","icon":null,"compose":null,"health":null}}]"#
        )
    }

    #[test]
    fn config_path_uses_native_separator() {
        let p = resolve_config();
        if cfg!(windows) {
            assert!(p.ends_with("local-store\\apps.json"), "got: {}", p);
        } else {
            assert!(!p.contains('\\'), "path must use native separator: {}", p);
            assert!(p.ends_with("local-store/apps.json"), "got: {}", p);
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

    #[test]
    fn primary_registry_wins_over_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&primary, app_json("primary")).unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert_eq!(load_apps_from_paths(&primary, &legacy)[0].name, "primary");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_primary_registry_falls_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert_eq!(load_apps_from_paths(&primary, &legacy)[0].name, "legacy");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn corrupt_primary_registry_does_not_fall_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::write(&primary, "not json").unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert!(load_apps_from_paths(&primary, &legacy).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn unreadable_primary_registry_does_not_fall_back_to_legacy_registry() {
        let (root, primary, legacy) = fixture_paths();
        fs::create_dir(&primary).unwrap();
        fs::write(&legacy, app_json("legacy")).unwrap();

        assert!(load_apps_from_paths(&primary, &legacy).is_empty());
        fs::remove_dir_all(root).unwrap();
    }
}
