//! Platform integration: OS shortcuts and protocol-handler registration.

#[cfg(all(unix, not(target_os = "macos")))]
use crate::brand::CLI_NAME;
#[cfg(any(windows, target_os = "macos"))]
use crate::brand::PRODUCT_NAME;

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

/// Register the configured schemes through the official plugin. macOS uses
/// installer metadata; runtime registration is supported on Windows/Linux.
pub fn register_protocol(app: &tauri::AppHandle) -> crate::error::AppResult<()> {
    #[cfg(any(windows, target_os = "linux"))]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link()
            .register_all()
            .map_err(crate::error::AppError::internal)?;
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    let _ = app;
    Ok(())
}
