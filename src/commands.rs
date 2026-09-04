//! Launcher commands. Remote app windows have no permission to invoke these.
use crate::{catalog, model::AppDef, storage, windowing};
use std::sync::Mutex;

static REGISTRY_WRITE: Mutex<()> = Mutex::new(());

pub fn require_launcher(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "launcher" {
        Ok(())
    } else {
        Err("Only the launcher can perform this action.".into())
    }
}

#[tauri::command]
pub fn list_apps(window: tauri::WebviewWindow) -> Result<Vec<AppDef>, String> {
    require_launcher(&window)?;
    storage::try_load_apps().map_err(|e| e.to_string())
}

pub fn validate_connection(name: &str, url: &str) -> Result<(String, String), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 80 || name.chars().any(char::is_control) {
        return Err(
            "Use an app name between 1 and 80 characters without control characters.".into(),
        );
    }
    Ok((name.into(), windowing::validated_external_url(url.trim())?))
}

#[tauri::command]
pub fn add_app(window: tauri::WebviewWindow, name: String, url: String) -> Result<(), String> {
    require_launcher(&window)?;
    let (name, url) = validate_connection(&name, &url)?;
    let _lock = REGISTRY_WRITE.lock().map_err(|e| e.to_string())?;
    let mut apps = storage::try_load_apps().map_err(|e| e.to_string())?;
    if apps.iter().any(|app| app.name.eq_ignore_ascii_case(&name)) {
        return Err("That name is already in My Apps. Choose a different name.".into());
    }
    apps.push(AppDef {
        name,
        url,
        icon: None,
        compose: None,
        health: None,
    });
    storage::try_save_apps(&apps).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_app_cmd(window: tauri::WebviewWindow, name: String) -> Result<(), String> {
    require_launcher(&window)?;
    let _lock = REGISTRY_WRITE.lock().map_err(|e| e.to_string())?;
    let mut apps = storage::try_load_apps().map_err(|e| e.to_string())?;
    let before = apps.len();
    apps.retain(|app| app.name != name);
    if before == apps.len() {
        return Err("This connection no longer exists. Refresh My Apps.".into());
    }
    storage::try_save_apps(&apps).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn search_catalog(
    window: tauri::WebviewWindow,
    query: String,
    category: String,
    offset: usize,
    limit: usize,
) -> Result<catalog::CatalogPage, String> {
    require_launcher(&window)?;
    Ok(catalog::search_catalog(&query, &category, offset, limit))
}

#[tauri::command]
pub fn open_project(window: tauri::WebviewWindow, url: String) -> Result<(), String> {
    require_launcher(&window)?;
    let url = windowing::validated_external_url(&url)?;
    open::that_detached(url).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn connection_validation_accepts_lan_and_rejects_unsafe_or_empty_values() {
        assert_eq!(
            validate_connection(" My app ", " http://192.168.1.2:3000 ").unwrap(),
            ("My app".into(), "http://192.168.1.2:3000".into())
        );
        for (name, url) in [
            ("", "https://example.com"),
            ("app", "file:///tmp/a"),
            ("app", "https://user:pass@example.com"),
            ("app", "http://bad host"),
            ("app", "https://"),
        ] {
            assert!(validate_connection(name, url).is_err());
        }
    }
}
