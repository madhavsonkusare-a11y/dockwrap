//! Platform integration: OS shortcuts and protocol-handler registration.

#[cfg(not(windows))]
use crate::brand::CLI_NAME;
use crate::brand::{LEGACY_URL_SCHEME, PRODUCT_NAME, URL_SCHEME};

/// Create an OS shortcut (Start Menu on Windows, ~/.local/share/applications on
/// Linux) that launches the current CLI with the app's icon.
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
    let start_menu =
        std::env::var("APPDATA").unwrap_or_default() + "\\Microsoft\\Windows\\Start Menu\\Programs";
    let _ = std::fs::create_dir_all(&start_menu);
    let lnk_path = format!("{}\\{} - {}.lnk", start_menu, PRODUCT_NAME, name);
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
    let desktop = format!("{}/{}-{}.desktop", dir, CLI_NAME, name);
    let icon_line = icon.map(|i| format!("Icon={}\n", i)).unwrap_or_default();
    let content = format!(
        "[Desktop Entry]\nType=Application\nName={name}\nExec=\"{bin}\" open {name}\n{icon_line}Terminal=false\nCategories=Utility;\n"
    );
    std::fs::write(&desktop, content).map_err(|e| e.to_string())?;
    let _ = std::fs::set_permissions(
        &desktop,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    );
    Ok(desktop)
}

/// Register primary and one-release legacy URL schemes with the OS (best-effort;
/// failures are logged but never fatal). On macOS schemes are registered in
/// `src/Info.plist` at install time.
pub fn register_protocol() {
    let bin = match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => return,
    };
    register_protocol_scheme(URL_SCHEME, &bin);
    register_protocol_scheme(LEGACY_URL_SCHEME, &bin);
}

#[cfg(any(test, all(unix, not(target_os = "macos"))))]
fn protocol_handler_desktop_id(scheme: &str) -> String {
    format!("{scheme}-urlhandler.desktop")
}

#[cfg(any(test, all(unix, not(target_os = "macos"))))]
fn protocol_handler_command_args(scheme: &str) -> [String; 4] {
    [
        "set".to_string(),
        "default-url-scheme-handler".to_string(),
        scheme.to_string(),
        protocol_handler_desktop_id(scheme),
    ]
}

#[cfg(windows)]
fn register_protocol_scheme(scheme: &str, bin: &str) {
    let cmd = format!(
        "reg add HKCU\\Software\\Classes\\{scheme} /f /ve /t REG_SZ /d \"URL:{PRODUCT_NAME} Protocol\" && \
         reg add HKCU\\Software\\Classes\\{scheme} /f /v \"URL Protocol\" /t REG_SZ /d \"\" && \
         reg add HKCU\\Software\\Classes\\{scheme}\\DefaultIcon /f /ve /t REG_SZ /d \"{bin},0\" && \
         reg add HKCU\\Software\\Classes\\{scheme}\\shell\\open\\command /f /ve /t REG_SZ /d \"\\\"{bin}\\\" \\\"%1\\\"\""
    );
    let _ = std::process::Command::new("cmd")
        .args(["/C", &cmd])
        .status();
}

#[cfg(all(unix, not(target_os = "macos")))]
fn register_protocol_scheme(scheme: &str, bin: &str) {
    let dir = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{}/.local/share", std::env::var("HOME").unwrap_or_default()))
        + "/applications";
    let _ = std::fs::create_dir_all(&dir);
    let desktop_id = protocol_handler_desktop_id(scheme);
    let desktop = format!("{dir}/{desktop_id}");
    let content = format!(
        "[Desktop Entry]\nType=Application\nName={PRODUCT_NAME} URL Handler\nExec=\"{bin}\" %u\nMimeType=x-scheme-handler/{scheme}\nTerminal=false\nNoDisplay=true\n"
    );
    if std::fs::write(&desktop, content).is_ok() {
        let _ = std::process::Command::new("xdg-settings")
            .args(protocol_handler_command_args(scheme))
            .status();
    }
}

#[cfg(target_os = "macos")]
fn register_protocol_scheme(_: &str, _: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_handler_uses_desktop_file_id_not_path() {
        assert_eq!(
            protocol_handler_desktop_id("localstore"),
            "localstore-urlhandler.desktop"
        );
        assert_eq!(
            protocol_handler_command_args("localstore"),
            [
                "set",
                "default-url-scheme-handler",
                "localstore",
                "localstore-urlhandler.desktop"
            ]
        );
    }
}
