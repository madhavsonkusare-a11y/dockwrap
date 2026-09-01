//! Thin Tauri command wrappers over the library modules. Each command is a
//! serializable entry point the launcher webview can invoke.

use crate::model::AppDef;
use crate::storage;

#[tauri::command]
pub fn list_apps() -> Vec<AppDef> {
    storage::load_apps()
}

#[tauri::command]
pub fn add_app(
    name: String,
    url: String,
    icon: Option<String>,
    compose: Option<String>,
    health: Option<String>,
) {
    storage::upsert_app(&name, &url, icon, compose, health);
}

#[tauri::command]
pub fn remove_app_cmd(name: String) -> bool {
    storage::remove_app(&name)
}
