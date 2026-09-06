//! Platform integration: OS shortcuts and protocol-handler registration.

#[cfg(all(unix, not(target_os = "macos")))]
use crate::brand::CLI_NAME;
use crate::brand::{LEGACY_URL_SCHEME, PRODUCT_NAME, URL_SCHEME};

pub mod shortcut;

/// Write a native protocol shortcut using a stable ID as the filename.
pub fn create_shortcut_for(
    id: &str,
    name: &str,
    bin: &str,
    icon: Option<&str>,
) -> Result<String, String> {
    #[cfg(windows)]
    {
        let _ = (name, bin);
        let content = shortcut::windows_content(id, icon)?;
        let dir =
            std::path::PathBuf::from(std::env::var_os("APPDATA").ok_or("APPDATA is unavailable")?)
                .join("Microsoft/Windows/Start Menu/Programs");
        write_shortcut(&dir, &format!("{PRODUCT_NAME} - {id}.url"), &content)
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let content = shortcut::linux_content(id, name, bin, icon)?;
        let dir = std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(|home| std::path::PathBuf::from(home).join(".local/share"))
            })
            .ok_or("No application data directory is available")?
            .join("applications");
        write_shortcut(&dir, &format!("{CLI_NAME}-{id}.desktop"), &content)
    }
    #[cfg(target_os = "macos")]
    {
        let _ = (name, bin, icon);
        let content = shortcut::macos_content(id)?;
        let dir = std::path::PathBuf::from(std::env::var_os("HOME").ok_or("HOME is unavailable")?)
            .join("Desktop");
        write_shortcut(&dir, &format!("{PRODUCT_NAME} - {id}.webloc"), &content)
    }
}

fn write_shortcut(dir: &std::path::Path, filename: &str, content: &str) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    let path = dir.join(filename);
    std::fs::write(&path, content).map_err(|error| error.to_string())?;
    Ok(path.to_string_lossy().into_owned())
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
