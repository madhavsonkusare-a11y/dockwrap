#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager, WebviewUrl, WebviewWindowBuilder,
};

#[derive(Deserialize, Clone, serde::Serialize)]
struct AppDef {
    name: String,
    url: String,
    icon: Option<String>,
    /// When true the app's window has NO taskbar button — it lives only as a
    /// tray-pinned window (e.g. a long-running local service like Penpot's
    /// Docker stack). Defaults to false for normal apps.
    #[serde(default)]
    skip_taskbar: bool,
}

const EXAMPLE_URL: &str = "http://localhost:9001/#/workspace/3364d985-c11e-8197-8008-89bcbd1341e9/3364d985-c11e-8197-8008-89c3aa739819";

/// Where registered apps live. Same location on every platform via APPDATA.
fn resolve_config() -> String {
    let env = std::env::var("APPDATA").unwrap_or_default();
    env + "\\dockwrap\\apps.json"
}

fn load_apps() -> Vec<AppDef> {
    let data = std::fs::read_to_string(&resolve_config()).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&data).unwrap_or_default()
}

fn save_apps(apps: &[AppDef]) {
    let path = resolve_config();
    if let Some(parent) = std::path::Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, serde_json::to_string(apps).unwrap());
}

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

#[cfg(windows)]
fn set_window_toolwindow(hwnd: isize) {
    use std::os::raw::c_void;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_TOOLWINDOW,
    };
    unsafe {
        let h = hwnd as *mut c_void;
        let ex = GetWindowLongPtrW(h, GWL_EXSTYLE) as u32;
        // Drop the taskbar button (WS_EX_APPWINDOW) and adopt the tool-window
        // style. This is the reliable Windows hide that Tauri's skip_taskbar
        // sometimes misses (tauri-apps/tauri#10422). Tradeoff: the window also
        // leaves Alt+Tab.
        let new_ex = (ex & !WS_EX_APPWINDOW) | WS_EX_TOOLWINDOW;
        SetWindowLongPtrW(h, GWL_EXSTYLE, new_ex as isize);
    }
}

/// Apply the chosen taskbar behavior to a freshly built window.
/// - `skip_taskbar == true`: window gets NO taskbar button. On Windows we set
///   WS_EX_TOOLWINDOW directly (reliable); on other platforms we use Tauri's
///   skip_taskbar. macOS ignores it (unsupported), which is fine.
fn apply_taskbar_visibility(win: &tauri::WebviewWindow, skip_taskbar: bool) {
    if !skip_taskbar {
        return;
    }
    #[cfg(windows)]
    {
        use raw_window_handle::HasWindowHandle;
        if let Ok(handle) = win.window_handle() {
            use raw_window_handle::RawWindowHandle;
            if let RawWindowHandle::Win32(w) = handle.into() {
                set_window_toolwindow(w.hwnd.get() as isize);
            }
        }
    }
    #[cfg(not(windows))]
    {
        let _ = win.set_skip_taskbar(true);
    }
}

fn build_window(app: &tauri::AppHandle, label: &str, appdef: &AppDef) {
    if let Ok(parsed) = appdef.url.parse::<tauri::Url>() {
        if let Some(win) = WebviewWindowBuilder::new(app, label, WebviewUrl::External(parsed))
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
            .ok()
        {
            apply_taskbar_visibility(&win, appdef.skip_taskbar);
            let _ = win.set_focus();
        }
    }
}

#[tauri::command]
fn list_apps() -> Vec<AppDef> {
    load_apps()
}

#[tauri::command]
fn add_app(name: String, url: String, icon: Option<String>) {
    let mut apps = load_apps();
    apps.retain(|a| a.name != name);
    apps.push(AppDef { name, url, icon, skip_taskbar: false });
    save_apps(&apps);
}

#[tauri::command]
fn open_app(app_handle: tauri::AppHandle, name: String) {
    if let Some(appdef) = load_apps().iter().find(|a| a.name == name) {
        build_window(&app_handle, &appdef.name, appdef);
    }
}

/// Build the dockwrap system tray. The tray icon is what sits in the
/// notification / hidden-icons area; from its menu the user can open any
/// registered app, open the launcher, or quit.
fn build_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let open_penpot = MenuItem::with_id(app, "open_penpot", "Open Penpot", true, None::<&str>)?;
    let open_launcher =
        MenuItem::with_id(app, "open_launcher", "Open Launcher", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit dockwrap", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_penpot, &open_launcher, &quit])?;

    let _tray = TrayIconBuilder::with_id("dockwrap-main")
        .tooltip("dockwrap")
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open_penpot" => {
                if let Some(appdef) = load_apps().iter().find(|a| a.name == "penpot") {
                    build_window(app, &appdef.name, appdef);
                }
            }
            "open_launcher" => {
                if let Some(win) = app.get_webview_window("launcher") {
                    let _ = win.show();
                    let _ = win.set_focus();
                } else {
                    let _ = WebviewWindowBuilder::new(
                        app,
                        "launcher",
                        WebviewUrl::App("index.html".into()),
                    )
                    .title("dockwrap")
                    .inner_size(460.0, 340.0)
                    .resizable(true)
                    .build();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Seed an example app on first run so the tool is useful immediately.
            if load_apps().is_empty() {
                save_apps(&[AppDef {
                    name: "penpot".into(),
                    url: EXAMPLE_URL.into(),
                    icon: None,
                    // Long-running local service: tray-pinned, no taskbar button.
                    skip_taskbar: true,
                }]);
            }
            // Build the dockwrap system tray (the thing that lives in the
            // hidden-icons / notification area). It lets the user re-open any
            // app and quit cleanly.
            build_tray(app.handle())?;
            let apps_json =
                serde_json::to_string(&load_apps()).unwrap_or_else(|_| "[]".into());
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
