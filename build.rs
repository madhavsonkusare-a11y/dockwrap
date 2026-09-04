fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "list_apps",
            "add_app",
            "open_app",
            "create_shortcut",
            "remove_app_cmd",
            "search_catalog",
            "open_project",
        ]),
    ))
    .expect("build Tauri application");
}
