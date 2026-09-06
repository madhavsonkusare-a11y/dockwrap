//! Data-only OS shortcuts. User text is never evaluated by a shell.
use crate::{brand::URL_SCHEME, model::is_valid_installed_app_id};

pub fn target(id: &str) -> Result<String, String> {
    if !is_valid_installed_app_id(id) {
        return Err("Invalid app ID for shortcut.".into());
    }
    Ok(format!("{URL_SCHEME}://open/{id}"))
}

fn single_line(value: &str) -> Result<&str, String> {
    if value.chars().any(char::is_control) {
        return Err("Shortcut values must not contain control characters.".into());
    }
    Ok(value)
}

pub fn windows_content(id: &str, icon: Option<&str>) -> Result<String, String> {
    let target = target(id)?;
    let mut value = format!("[InternetShortcut]\r\nURL={target}\r\n");
    if let Some(icon) = icon {
        value.push_str(&format!(
            "IconFile={}\r\nIconIndex=0\r\n",
            single_line(icon)?
        ));
    }
    Ok(value)
}

fn desktop_string(value: &str) -> Result<String, String> {
    Ok(single_line(value)?.replace('\\', "\\\\"))
}

fn desktop_argument(value: &str) -> Result<String, String> {
    let escaped = single_line(value)?
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('%', "%%");
    // Desktop Entry value unescaping happens before Exec argument unquoting.
    Ok(format!("\"{}\"", escaped.replace('\\', "\\\\")))
}

pub fn linux_content(
    id: &str,
    name: &str,
    bin: &str,
    icon: Option<&str>,
) -> Result<String, String> {
    let target = target(id)?;
    let icon = match icon {
        Some(icon) => format!("Icon={}\n", desktop_string(icon)?),
        None => String::new(),
    };
    Ok(format!("[Desktop Entry]\nType=Application\nName={}\nExec={} {}\n{icon}Terminal=false\nCategories=Utility;\n",
        desktop_string(name)?, desktop_argument(bin)?, desktop_argument(&target)?))
}

pub fn macos_content(id: &str) -> Result<String, String> {
    Ok(format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<plist version=\"1.0\"><dict><key>URL</key><string>{}</string></dict></plist>\n", target(id)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_use_native_protocol_instead_of_browser_only_cli() {
        assert!(windows_content("memos", None)
            .unwrap()
            .contains("URL=localstore://open/memos"));
        let linux = linux_content("memos", "My notes", "/opt/Local Store/bin", None).unwrap();
        assert!(linux.contains("Exec=\"/opt/Local Store/bin\" \"localstore://open/memos\""));
        assert!(macos_content("memos")
            .unwrap()
            .contains("<string>localstore://open/memos</string>"));
    }

    #[test]
    fn shortcut_values_reject_traversal_and_line_injection() {
        for id in ["../escape", "CON", "a/b", "a\nURL=https://bad.test", ""] {
            assert!(target(id).is_err());
        }
        assert!(windows_content("memos", Some("x\nURL=https://bad.test")).is_err());
        assert!(linux_content("memos", "x\nExec=bad", "/opt/app", None).is_err());
        assert!(linux_content("memos", "notes", "/opt/a\nExec=bad", None).is_err());
    }

    #[test]
    fn desktop_exec_escapes_reserved_characters_and_field_codes() {
        let content =
            linux_content("memos", "Notes & \"ideas\"", "/opt/50%/$app/`bin`", None).unwrap();
        assert!(content.contains("50%%/\\\\$app/\\\\`bin\\\\`"));
        assert!(content.contains("Name=Notes & \"ideas\""));
    }
}
