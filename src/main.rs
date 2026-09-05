#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use local_store::{
    brand::{CLI_NAME, LEGACY_URL_SCHEME, PRODUCT_NAME, URL_SCHEME},
    commands, platform, runtime, storage, windowing,
};

mod cli;

#[tauri::command]
fn create_shortcut(window: tauri::WebviewWindow, id: String) -> Result<String, String> {
    commands::require_launcher(&window)?;
    let app = storage::load_or_migrate_registry()
        .map_err(|e| e.to_string())?
        .apps
        .into_iter()
        .find(|app| app.id == id)
        .ok_or("This app no longer exists.")?;
    let bin = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();
    platform::create_shortcut_for(
        &app.id,
        &bin,
        app.icon_path.as_ref().and_then(|path| path.to_str()),
    )
}

#[tauri::command]
async fn open_app(
    window: tauri::WebviewWindow,
    app_handle: tauri::AppHandle,
    id: String,
) -> Result<(), String> {
    commands::require_launcher(&window)?;
    let app = storage::load_or_migrate_registry()
        .map_err(|e| e.to_string())?
        .apps
        .into_iter()
        .find(|app| app.id == id)
        .ok_or("This app no longer exists.")?;
    if app.is_managed() {
        let pending = app.clone();
        tauri::async_runtime::spawn_blocking(move || runtime::start(&pending))
            .await
            .map_err(|e| e.to_string())??;
    }
    windowing::build_window(
        &app_handle,
        &app.display_name,
        &app.launch_url,
        app.icon_path.as_ref().and_then(|path| path.to_str()),
    )
}

fn parse_deep_link(input: &str) -> Option<(String, String)> {
    let url = tauri::Url::parse(input).ok()?;
    let scheme = url.scheme();
    if scheme != URL_SCHEME && scheme != LEGACY_URL_SCHEME {
        return None;
    }
    if url.host_str()? != "open"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return None;
    }
    let name_segment = url.path().strip_prefix('/')?;
    if name_segment.is_empty() || name_segment.contains('/') {
        return None;
    }
    let name = windowing::strict_percent_decode_path_segment(name_segment)?;
    (!name.is_empty() && !name.contains('/')).then(|| (scheme.to_string(), name))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Primary and legacy protocol handlers are passed as a single argument by the OS.
    if let Some(first) = args.get(1) {
        if let Some((scheme, name)) = parse_deep_link(first) {
            // Boot the referenced app's stack, then open its window.
            let apps = storage::load_or_migrate_registry()
                .map(|registry| registry.apps)
                .unwrap_or_default();
            if let Some(appdef) = apps.iter().find(|a| a.id == name) {
                if appdef.is_managed() {
                    let _ = runtime::start(appdef);
                }
                let open_name = appdef.display_name.clone();
                let url = appdef.launch_url.clone();
                let icon = appdef.icon_path.clone();
                tauri::Builder::default()
                    .setup(move |app| {
                        windowing::build_window(
                            app.handle(),
                            &open_name,
                            &url,
                            icon.as_ref().and_then(|path| path.to_str()),
                        )
                        .map_err(std::io::Error::other)?;
                        Ok(())
                    })
                    .run(tauri::generate_context!())
                    .expect("error while running Local Store");
                return;
            } else {
                eprintln!("{scheme}://open/{name}: no such app");
            }
        }
    }

    // If invoked with CLI subcommands (e.g. `local-store add ...`), run as CLI and exit.
    if args.len() > 1 {
        let first = args[1].as_str();
        if first == "--version" || first == "-V" {
            println!("{CLI_NAME} {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        std::process::exit(cli::run_cli());
    }

    tauri::Builder::default()
        .setup(|app| {
            // Register primary and one-release legacy protocol handlers (best-effort).
            platform::register_protocol();
            storage::load_or_migrate_registry().map_err(std::io::Error::other)?;
            let win = tauri::WebviewWindowBuilder::new(
                app,
                "launcher",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title(PRODUCT_NAME)
            .theme(Some(tauri::Theme::Dark))
            .inner_size(1180.0, 760.0)
            .min_inner_size(800.0, 600.0)
            .resizable(true)
            .build()?;
            let _ = win.set_focus();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_apps,
            commands::add_app,
            open_app,
            create_shortcut,
            commands::remove_app_cmd,
            commands::search_catalog,
            commands::open_project,
            commands::doctor,
            commands::recipe_details,
            commands::install_app,
            commands::start_app,
            commands::stop_app,
            commands::app_logs,
            commands::uninstall_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Local Store");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primary_deep_link_with_one_percent_decoded_name_segment() {
        assert_eq!(
            parse_deep_link("localstore://open/My%20App"),
            Some(("localstore".to_string(), "My App".to_string()))
        );
    }

    #[test]
    fn parses_legacy_deep_link() {
        assert_eq!(
            parse_deep_link(&format!("{LEGACY_URL_SCHEME}://open/penpot")),
            Some((LEGACY_URL_SCHEME.to_string(), "penpot".to_string()))
        );
    }

    #[test]
    fn rejects_malformed_deep_links() {
        for input in [
            "https://open/penpot",
            "localstore://wrong/penpot",
            "localstore://open/",
            "localstore://open/penpot/extra",
            "localstore://open/penpot%2Fextra",
            "localstore://open/open/penpot",
            "localstore://open/penpot?x=1",
            "localstore://open/penpot#fragment",
            "localstore://open/bad%ZZ",
            "localstore://open/bad%",
            "localstore://open/%E0%A4%A",
            "localstore://open/%FF",
        ] {
            assert_eq!(parse_deep_link(input), None, "accepted {input}");
        }
    }
}
