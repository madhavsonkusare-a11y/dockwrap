//! Typed launcher commands. Remote app windows have no permission to invoke these.
use crate::{
    catalog,
    model::{InstalledApp, RuntimeSpec},
    recipes,
    runtime::{self, AppStatus, DoctorReport},
    storage, windowing,
};
use serde::Serialize;
use std::{
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
static REGISTRY_WRITE: Mutex<()> = Mutex::new(());

#[derive(Serialize)]
pub struct AppView {
    #[serde(flatten)]
    pub app: InstalledApp,
    pub status: AppStatus,
}
pub fn require_launcher(window: &tauri::WebviewWindow) -> Result<(), String> {
    if window.label() == "launcher" {
        Ok(())
    } else {
        Err("Only the launcher can perform this action.".into())
    }
}
fn now() -> Result<u64, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs())
}
fn find_app(id: &str) -> Result<InstalledApp, String> {
    storage::load_or_migrate_registry()
        .map_err(|e| e.to_string())?
        .apps
        .into_iter()
        .find(|app| app.id == id)
        .ok_or_else(|| "This app no longer exists. Refresh My Apps.".into())
}

#[tauri::command]
pub async fn list_apps(window: tauri::WebviewWindow) -> Result<Vec<AppView>, String> {
    require_launcher(&window)?;
    tauri::async_runtime::spawn_blocking(|| {
        let registry = storage::load_or_migrate_registry().map_err(|e| e.to_string())?;
        Ok(registry
            .apps
            .into_iter()
            .map(|app| {
                let status = runtime::status(&app).unwrap_or(AppStatus::Error);
                AppView { app, status }
            })
            .collect())
    })
    .await
    .map_err(|e| e.to_string())?
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
    let registry = storage::load_or_migrate_registry().map_err(|e| e.to_string())?;
    if registry
        .apps
        .iter()
        .any(|app| app.display_name.eq_ignore_ascii_case(&name))
    {
        return Err("That name is already in My Apps. Choose a different name.".into());
    }
    let base = storage::slug_for_display_name(&name);
    let id = crate::model::next_available_installed_app_id(&base, |candidate| {
        !registry.apps.iter().any(|app| app.id == candidate)
    })
    .ok_or("Could not create a safe app ID.")?;
    let timestamp = now()?;
    storage::insert_installed_app(InstalledApp {
        id,
        catalog_id: None,
        display_name: name,
        launch_url: url,
        icon_path: None,
        runtime: RuntimeSpec::External,
        created_at_unix: timestamp,
        updated_at_unix: timestamp,
    })
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn remove_app_cmd(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    require_launcher(&window)?;
    let _lock = REGISTRY_WRITE.lock().map_err(|e| e.to_string())?;
    let app = find_app(&id)?;
    if app.is_managed() {
        return Err("Use Uninstall for an app managed by Local Store.".into());
    }
    storage::remove_installed_app(&id)
        .map(|_| ())
        .map_err(|e| e.to_string())
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
    open::that_detached(windowing::validated_external_url(&url)?).map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn doctor(window: tauri::WebviewWindow) -> Result<DoctorReport, String> {
    require_launcher(&window)?;
    tauri::async_runtime::spawn_blocking(runtime::doctor)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn recipe_details(window: tauri::WebviewWindow, id: String) -> Result<recipes::Recipe, String> {
    require_launcher(&window)?;
    recipes::recipe(&id).ok_or_else(|| "This recipe is not verified for installation.".to_string())
}
#[tauri::command]
pub async fn install_app(window: tauri::WebviewWindow, recipe_id: String) -> Result<(), String> {
    require_launcher(&window)?;
    let recipe = recipes::recipe(&recipe_id)
        .ok_or_else(|| "This recipe is not verified for installation.".to_string())?;
    if storage::load_or_migrate_registry()
        .map_err(|e| e.to_string())?
        .apps
        .iter()
        .any(|app| app.id == recipe.id)
    {
        return Err("Memos is already in My Apps.".into());
    }
    let app = tauri::async_runtime::spawn_blocking(move || runtime::install_recipe(&recipe))
        .await
        .map_err(|e| e.to_string())??;
    if let Err(error) = storage::insert_installed_app(app.clone()) {
        let _ = runtime::uninstall(&app, true);
        return Err(error.to_string());
    }
    Ok(())
}
#[tauri::command]
pub async fn start_app(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    require_launcher(&window)?;
    let app = find_app(&id)?;
    tauri::async_runtime::spawn_blocking(move || runtime::start(&app))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn stop_app(window: tauri::WebviewWindow, id: String) -> Result<(), String> {
    require_launcher(&window)?;
    let app = find_app(&id)?;
    tauri::async_runtime::spawn_blocking(move || runtime::stop(&app))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn app_logs(window: tauri::WebviewWindow, id: String) -> Result<String, String> {
    require_launcher(&window)?;
    let app = find_app(&id)?;
    tauri::async_runtime::spawn_blocking(move || runtime::logs(&app))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn uninstall_app(
    window: tauri::WebviewWindow,
    id: String,
    delete_data: bool,
) -> Result<(), String> {
    require_launcher(&window)?;
    let app = find_app(&id)?;
    let pending = app.clone();
    tauri::async_runtime::spawn_blocking(move || runtime::uninstall(&pending, delete_data))
        .await
        .map_err(|e| e.to_string())??;
    storage::remove_installed_app(&app.id)
        .map(|_| ())
        .map_err(|e| e.to_string())
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
