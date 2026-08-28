#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cli;

mod registry;

use registry::AppDef;
use std::time::{Duration, Instant};
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

/// Boot the app's compose stack (if any) and wait until its health target responds.
/// Non-fatal by design: callers should still open the window even if this errors.
pub fn boot_and_wait(app: &AppDef) -> Result<(), String> {
    if let Some(compose) = &app.compose {
        // Use -f for an explicit file; if it's a directory, docker falls back to
        // compose.yml/compose.yaml inside it.
        let status = std::process::Command::new("docker")
            .args(["compose", "-f", compose, "up", "-d"])
            .status();
        match status {
            Ok(s) if s.success() => {}
            Ok(s) => return Err(format!("docker compose up exited {}", s)),
            Err(e) => return Err(format!("failed to spawn docker: {}", e)),
        }
    }

    let target = app
        .health
        .clone()
        .unwrap_or_else(|| app.url.clone());
    wait_for_health(&target, Duration::from_secs(60))
}

/// Poll `target` (an http(s):// URL or host:port) until it responds or the
/// deadline passes. Uses curl (present on Windows/macOS/Linux) for a portable,
/// dependency-free check.
fn wait_for_health(target: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    loop {
        if is_up(target) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(format!("health timeout after {:?} for {}", timeout, target));
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn is_up(target: &str) -> bool {
    std::process::Command::new("curl")
        .args(["-sf", "-o", "/dev/null", "--max-time", "2", target])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Create an OS shortcut (Start Menu on Windows, ~/.local/share/applications on
/// Linux) that launches `dockwrap open <name>` with the app's icon.
/// Returns the path written, or an error string.
pub fn create_shortcut_for(name: &str, bin: &str, icon: Option<&str>) -> Result<String, String> {
    #[cfg(windows)]
    {
        create_shortcut_windows(name, bin, icon)
    }
    #[cfg(not(windows))]
    {
        create_shortcut_unix(name, bin, icon)
    }
}

#[cfg(windows)]
fn create_shortcut_windows(name: &str, bin: &str, icon: Option<&str>) -> Result<String, String> {
    let start_menu = std::env::var("APPDATA").unwrap_or_default()
        + "\\Microsoft\\Windows\\Start Menu\\Programs";
    let _ = std::fs::create_dir_all(&start_menu);
    let lnk_path = format!("{}\\dockwrap - {}.lnk", start_menu, name);
    // Use WScript.Shell COM (the standard, dependency-free way to write a .lnk)
    // via PowerShell. Arguments are passed with single-quoted heredoc-safe escaping.
    let esc = |s: &str| s.replace('\'', "''");
    let icon_arg = match icon {
        Some(ic) => format!("$sc.IconLocation='{},0';", esc(ic)),
        None => String::new(),
    };
    let ps = format!(
        "$ws=New-Object -ComObject WScript.Shell; \
         $sc=$ws.CreateShortcut('{}'); \
         $sc.TargetPath='{}'; \
         $sc.Arguments='open {}'; \
         {} \
         $sc.Save();",
        esc(&lnk_path),
        esc(bin),
        esc(name),
        icon_arg
    );
    let status = std::process::Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps])
        .status();
    match status {
        Ok(s) if s.success() => Ok(lnk_path),
        Ok(s) => Err(format!("powershell shortcut failed: {}", s)),
        Err(e) => Err(format!("failed to spawn powershell: {}", e)),
    }
}

#[cfg(not(windows))]
fn create_shortcut_unix(name: &str, bin: &str, icon: Option<&str>) -> Result<String, String> {
    let dir = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap_or_default()))
        + "/applications";
    let _ = std::fs::create_dir_all(&dir);
    let desktop = format!("{}/dockwrap-{}.desktop", dir, name);
    let icon_line = icon.map(|i| format!("Icon={}\n", i)).unwrap_or_default();
    let content = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec=\"{bin}\" open {name}\n{icon_line}Terminal=false\nCategories=Utility;\n"
    );
    std::fs::write(&desktop, content).map_err(|e| e.to_string())?;
    let _ = std::fs::set_permissions(&desktop, std::os::unix::fs::PermissionsExt::from_mode(0o755));
    Ok(desktop)
}

fn build_window(app: &tauri::AppHandle, label: &str, url: &str, icon: Option<&str>) {
    if let Ok(parsed) = url.parse::<tauri::Url>() {
        let builder = WebviewWindowBuilder::new(app, label, WebviewUrl::External(parsed))
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
            });
        // Per-app title-bar icon (best-effort; falls back to the app default).
        let result = if let Some(ic) = icon {
            match tauri::image::Image::from_path(ic) {
                Ok(img) => builder.icon(img),
                Err(_) => Ok(builder),
            }
        } else {
            Ok(builder)
        };
        if let Ok(b) = result {
            let _ = b.build().and_then(|w| w.set_focus());
        }
    }
}

/// Register the `dockwrap://` URL scheme with the OS (best-effort; failures are
/// logged but never fatal). Windows uses the registry; Linux uses xdg-settings
/// against a .desktop file. On macOS the scheme is registered at install time
/// via the `CFBundleURLTypes` entry in tauri.conf.json `bundle.macOS` (the
/// bundle's Info.plist), so no runtime step is needed there.
fn register_protocol() {
    let bin = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => return,
    };
    #[cfg(windows)]
    {
        let cmd = format!(
            "reg add HKCU\\Software\\Classes\\dockwrap /f /ve /t REG_SZ /d \"URL:dockwrap Protocol\" && \
             reg add HKCU\\Software\\Classes\\dockwrap /f /v \"URL Protocol\" /t REG_SZ /d \"\" && \
             reg add HKCU\\Software\\Classes\\dockwrap\\DefaultIcon /f /ve /t REG_SZ /d \"{},0\" && \
             reg add HKCU\\Software\\Classes\\dockwrap\\shell\\open\\command /f /ve /t REG_SZ /d \"\\\"{}\\\" \\\"%1\\\"\"",
            bin, bin
        );
        let _ = std::process::Command::new("cmd").args(["/C", &cmd]).status();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = std::env::var("XDG_DATA_HOME")
            .unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap_or_default()))
            + "/applications";
        let _ = std::fs::create_dir_all(&dir);
        let desktop = format!("{}/dockwrap-urlhandler.desktop", dir);
        let content = format!(
            "[Desktop Entry]\nType=Application\nName=dockwrap URL Handler\nExec=\"{}\" %u\nMimeType=x-scheme-handler/dockwrap\nTerminal=false\nNoDisplay=true\n",
            bin
        );
        if std::fs::write(&desktop, content).is_ok() {
            let _ = std::process::Command::new("xdg-settings")
                .args(["set", "default-url-scheme-handler", "dockwrap", &desktop])
                .status();
        }
    }
}

#[tauri::command]
fn list_apps() -> Vec<AppDef> {
    registry::load_apps()
}

#[tauri::command]
fn add_app(
    name: String,
    url: String,
    icon: Option<String>,
    compose: Option<String>,
    health: Option<String>,
) {
    registry::upsert_app(&name, &url, icon, compose, health);
}

#[tauri::command]
fn remove_app_cmd(name: String) -> bool {
    registry::remove_app(&name)
}

#[tauri::command]
fn create_shortcut(name: String) -> Result<String, String> {
    let apps = registry::load_apps();
    let appdef = apps
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| format!("No app named \"{}\" found.", name))?;
    let bin = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();
    create_shortcut_for(&name, &bin, appdef.icon.as_deref())
}

#[tauri::command]
fn open_app(app_handle: tauri::AppHandle, name: String) {
    if let Some(appdef) = registry::load_apps().iter().find(|a| a.name == name) {
        // Best-effort: boot the app's compose stack and wait for health.
        // Non-fatal — we still open the window even if docker/health fails.
        if let Err(e) = crate::boot_and_wait(&appdef) {
            eprintln!("warn: {}", e);
        }
        build_window(&app_handle, &appdef.name, &appdef.url, appdef.icon.as_deref());
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
            let apps = registry::load_apps();
            if let Some(appdef) = apps.iter().find(|a| a.name == name) {
                if let Err(e) = crate::boot_and_wait(&appdef) {
                    eprintln!("warn: {}", e);
                }
                let open_name = appdef.name.clone();
                let url = appdef.url.clone();
                let icon = appdef.icon.clone();
                tauri::Builder::default()
                    .setup(move |app| {
                        build_window(app.handle(), &open_name, &url, icon.as_deref());
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
            register_protocol();
            // Seed an example app on first run so the tool is useful immediately.
            if registry::load_apps().is_empty() {
                registry::upsert_app("penpot", EXAMPLE_URL, None, None, None);
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
            .inner_size(480.0, 420.0)
            .resizable(true)
            .initialization_script(&script)
            .build()?;
            let _ = win.set_focus();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_apps,
            add_app,
            open_app,
            create_shortcut,
            remove_app_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running dockwrap");
}
