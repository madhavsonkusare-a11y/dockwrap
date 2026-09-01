//! Platform integration: OS shortcuts and protocol-handler registration.

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
    let start_menu =
        std::env::var("APPDATA").unwrap_or_default() + "\\Microsoft\\Windows\\Start Menu\\Programs";
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
    let _ = std::fs::set_permissions(
        &desktop,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    );
    Ok(desktop)
}

/// Register the `dockwrap://` URL scheme with the OS (best-effort; failures are
/// logged but never fatal). Windows uses the registry; Linux uses xdg-settings
/// against a .desktop file. On macOS the scheme is registered at install time
/// via the `CFBundleURLTypes` entry in tauri.conf.json `bundle.macOS` (the
/// bundle's Info.plist), so no runtime step is needed there.
pub fn register_protocol() {
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
        let _ = std::process::Command::new("cmd")
            .args(["/C", &cmd])
            .status();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let dir = std::env::var("XDG_DATA_HOME").unwrap_or_else(|_| {
            format!("{}/.local/share", std::env::var("HOME").unwrap_or_default())
        }) + "/applications";
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
