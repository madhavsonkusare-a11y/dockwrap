#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dockwrap::{catalog, commands, model, platform, runtime, storage, windowing};

mod cli;

const EXAMPLE_URL: &str = "http://localhost:9001/#/workspace/3364d985-c11e-8197-8008-89bcbd1341e9/3364d985-c11e-8197-8008-89c3aa739819";

#[tauri::command]
fn catalog_cmd() -> Vec<model::CatalogEntry> {
    catalog::catalog()
}

#[tauri::command]
fn create_shortcut(name: String) -> Result<String, String> {
    let apps = storage::load_apps();
    let appdef = apps
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| format!("No app named \"{}\" found.", name))?;
    let bin = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();
    platform::create_shortcut_for(&name, &bin, appdef.icon.as_deref())
}

#[tauri::command]
fn open_app(app_handle: tauri::AppHandle, name: String) {
    if let Some(appdef) = storage::load_apps().iter().find(|a| a.name == name) {
        // Best-effort: boot the app's compose stack and wait for health.
        // Non-fatal — we still open the window even if docker/health fails.
        if let Err(e) = runtime::boot_and_wait(appdef) {
            eprintln!("warn: {}", e);
        }
        windowing::build_window(
            &app_handle,
            &appdef.name,
            &appdef.url,
            appdef.icon.as_deref(),
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // Protocol handler: `dockwrap://open/<name>` (passed as a single arg by the OS).
    if let Some(first) = args.get(1) {
        if let Some(rest) = first.strip_prefix("dockwrap://") {
            let name = rest
                .trim_start_matches("open/")
                .trim_start_matches("open")
                .trim_matches('/')
                .to_string();
            // Boot the referenced app's stack, then open its window.
            let apps = storage::load_apps();
            if let Some(appdef) = apps.iter().find(|a| a.name == name) {
                if let Err(e) = runtime::boot_and_wait(appdef) {
                    eprintln!("warn: {}", e);
                }
                let open_name = appdef.name.clone();
                let url = appdef.url.clone();
                let icon = appdef.icon.clone();
                tauri::Builder::default()
                    .setup(move |app| {
                        windowing::build_window(app.handle(), &open_name, &url, icon.as_deref());
                        Ok(())
                    })
                    .run(tauri::generate_context!())
                    .expect("error while running dockwrap");
                return;
            } else {
                eprintln!("dockwrap://open/{}: no such app", name);
            }
        }
    }

    // If invoked with CLI subcommands (e.g. `dockwrap add ...`), run as CLI and exit.
    if args.len() > 1 {
        let first = args[1].as_str();
        if first == "--version" || first == "-V" {
            println!("dockwrap {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        std::process::exit(cli::run_cli());
    }

    tauri::Builder::default()
        .setup(|app| {
            // Register the dockwrap:// protocol handler with the OS (best-effort).
            platform::register_protocol();
            // Seed an example app on first run so the tool is useful immediately.
            if storage::load_apps().is_empty() {
                storage::upsert_app("penpot", EXAMPLE_URL, None, None, None);
            }
            let apps_json =
                serde_json::to_string(&storage::load_apps()).unwrap_or_else(|_| "[]".into());
            let init = format!("window.__APPS__ = {};", apps_json);
            let script = format!("{}\n{}", init, windowing::LINK_BRIDGE_JS);
            let win = tauri::WebviewWindowBuilder::new(
                app,
                "launcher",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("dockwrap")
            .inner_size(480.0, 420.0)
            .resizable(true)
            .initialization_script(&script)
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
            catalog_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running dockwrap");
}
