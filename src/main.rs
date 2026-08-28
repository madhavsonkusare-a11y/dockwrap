#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;
mod registry;

use registry::AppDef;
use tauri::{WebviewUrl, WebviewWindowBuilder};

const EXAMPLE_URL: &str = "http://localhost:9001/#/workspace/3364d985-c11e-8197-8008-89bcbd1341e9/3364d985-c11e-8197-8008-89c3aa739819";

/// Injected into every webview. Intercepts window.open + clicks on external
/// links and rewrites the navigation to a localhost marker URL that Rust
/// catches in `on_navigation`, then launches the OS default browser.
const LINK_BRIDGE_JS: &str = r#"
(function(){
  if (window.__dockwrapBridge) return;
  window.__dockwrapBridge = true;
  const origOpen = window.open.bind(window);
  window.open = function(u, n, f) {
    if (!u) return origOpen(u, n, f);
    const abs = new URL(u, location.href).href;
    if (/^https?:/.test(abs) && !abs.includes("localhost")) {
      location.href = "http://127.0.0.1:65535/.external?" + encodeURIComponent(abs);
      return null;
    }
    return origOpen(u, n, f);
  };
  document.addEventListener("click", function(e) {
    const el = e.target && e.target.closest ? e.target.closest("a[href]") : null;
    if (!el) return;
    const h = el.getAttribute("href");
    if (!h) return;
    const abs = new URL(h, location.href).href;
    if (/^https?:/.test(abs) && !abs.includes("localhost")) {
      e.preventDefault();
      location.href = "http://127.0.0.1:65535/.external?" + encodeURIComponent(abs);
    }
  }, true);
})();
"#;

/// Open a URL in the user's default browser, hidden (no console flash).
#[cfg(windows)]
fn launch_browser(url: &str) {
    use std::os::windows::process::CommandExt;
    let _ = std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .creation_flags(0x08000000)
        .spawn();
}

#[cfg(not(windows))]
fn launch_browser(url: &str) {
    // Best-effort: xdg-open (Linux) / open (macOS). Fails silently if absent.
    let _ = std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" })
        .arg(url)
        .spawn();
}

fn percent_decode_str(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(v) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(v) = u8::from_str_radix(v, 16) {
                    out.push(v);
                    i += 3;
                    continue;
                }
            }
        }
        out.push(if bytes[i] == b'+' { b' ' } else { bytes[i] });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn build_window(app: &tauri::AppHandle, label: &str, url: &str) {
    if let Ok(parsed) = url.parse::<tauri::Url>() {
        let _ = WebviewWindowBuilder::new(app, label, WebviewUrl::External(parsed))
            .title(label)
            .inner_size(1600.0, 900.0)
            .resizable(true)
            .initialization_script(LINK_BRIDGE_JS)
            .on_navigation(move |url| {
                let s = url.as_str();
                if let Some(query) = s.strip_prefix("http://127.0.0.1:65535/.external?") {
                    launch_browser(&percent_decode_str(query));
                    return false;
                }
                true
            })
            .build()
            .and_then(|w| w.set_focus());
    }
}

#[tauri::command]
fn list_apps() -> Vec<AppDef> {
    registry::load_apps()
}

#[tauri::command]
fn add_app(name: String, url: String, icon: Option<String>) {
    registry::upsert_app(&name, &url, icon);
}

#[tauri::command]
fn open_app(app_handle: tauri::AppHandle, name: String) {
    if let Some(appdef) = registry::load_apps().iter().find(|a| a.name == name) {
        build_window(&app_handle, &appdef.name, &appdef.url);
    }
}

fn main() {
    // If invoked with arguments (e.g. `dockwrap add ...`), run as a CLI and exit.
    // GUI mode is launched only with no extra arguments.
    if std::env::args().count() > 1 {
        std::process::exit(cli::run_cli());
    }

    tauri::Builder::default()
        .setup(|app| {
            // Seed an example app on first run so the tool is useful immediately.
            if registry::load_apps().is_empty() {
                registry::upsert_app("penpot", EXAMPLE_URL, None);
            }
            let apps_json =
                serde_json::to_string(&registry::load_apps()).unwrap_or_else(|_| "[]".into());
            let init = format!("window.__APPS__ = {};", apps_json);
            let script = format!("{}\n{}", init, LINK_BRIDGE_JS);
            let win = WebviewWindowBuilder::new(
                app,
                "launcher",
                WebviewUrl::App("index.html".into()),
            )
            .title("dockwrap")
            .inner_size(460.0, 340.0)
            .resizable(true)
            .initialization_script(&script)
            .build()?;
            let _ = win.set_focus();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![list_apps, add_app, open_app])
        .run(tauri::generate_context!())
        .expect("error while running dockwrap");
}
