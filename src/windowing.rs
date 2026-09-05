//! Windowing: webview window construction, the external-link bridge JS, and
//! external-URL validation/policy for routing links to the OS browser.

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Injected into every webview. Intercepts window.open + clicks on external
/// links and rewrites the navigation to a localhost marker URL that Rust
/// catches in `on_navigation`, then launches the OS default browser.
pub const LINK_BRIDGE_JS: &str = r#"
(function(){
  if (window.__dockwrapBridge) return;
  window.__dockwrapBridge = true;
  const origOpen = window.open.bind(window);
  window.open = function(u, n, f) {
    if (!u) return origOpen(u, n, f);
    const abs = new URL(u, location.href).href;
    if (/^https?:/.test(abs) && new URL(abs).origin !== location.origin) {
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
    if (/^https?:/.test(abs) && new URL(abs).origin !== location.origin) {
      e.preventDefault();
      location.href = "http://127.0.0.1:65535/.external?" + encodeURIComponent(abs);
    }
  }, true);
})();
"#;

pub fn percent_decode_str(s: &str) -> String {
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

/// Decode a URL path segment only when every percent escape is valid UTF-8.
/// Unlike the query-string compatibility decoder above, this preserves literal
/// `+` characters and rejects malformed escapes rather than passing them on.
pub fn strict_percent_decode_path_segment(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if i + 2 >= bytes.len()
                || !bytes[i + 1].is_ascii_hexdigit()
                || !bytes[i + 2].is_ascii_hexdigit()
            {
                return None;
            }
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

/// Validate a URL handed to us for external navigation before it may reach the
/// OS browser launcher. Accepts only absolute http(s) URLs of at most 2048
/// bytes; anything else (file:, javascript:, data:, malformed or oversized
/// payloads) is rejected. Returns the URL unchanged when valid.
// TODO(Task 16): wire into on_navigation and the launcher path; until then this
// is only exercised by tests.
#[allow(dead_code)]
pub fn validated_external_url(url: &str) -> Result<String, String> {
    if url.len() > 2048 || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err("Use a URL without spaces, up to 2048 bytes.".into());
    }
    if !(url.to_ascii_lowercase().starts_with("http://")
        || url.to_ascii_lowercase().starts_with("https://"))
    {
        return Err("Enter a complete http:// or https:// address.".into());
    }
    let parsed = tauri::Url::parse(url).map_err(|_| "Enter a valid app address.".to_string())?;
    if parsed.host_str().is_none() || !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("Use an address with a host and no embedded username or password.".into());
    }
    Ok(url.to_owned())
}

/// Open a URL in the user's default browser via the runtime module.
pub use crate::runtime::launch_browser;

pub fn build_window(
    app: &tauri::AppHandle,
    name: &str,
    url: &str,
    icon: Option<&str>,
) -> Result<(), String> {
    let url = validated_external_url(url)?;
    let parsed = url.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    // Stable, collision-free labels independent of display-name punctuation.
    let label = format!(
        "app-{}",
        name.as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    );
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    let origin = parsed.origin();
    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed))
        .title(name)
        .inner_size(1200.0, 800.0)
        .resizable(true)
        .initialization_script(LINK_BRIDGE_JS)
        .on_navigation(move |url| {
            if let Some(query) = url
                .as_str()
                .strip_prefix("http://127.0.0.1:65535/.external?")
            {
                launch_browser(&percent_decode_str(query));
                return false;
            }
            if !matches!(url.scheme(), "http" | "https") {
                return false;
            }
            if url.origin() != origin {
                launch_browser(url.as_str());
                return false;
            }
            true
        });
    let builder = if let Some(image) = icon.and_then(|p| tauri::image::Image::from_path(p).ok()) {
        builder.icon(image).map_err(|e| e.to_string())?
    } else {
        builder
    };
    builder
        .build()
        .and_then(|w| w.set_focus())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Characterization: percent_decode_str (current v0.4 behavior) ----
    //
    // These tests lock down what the existing implementation ACTUALLY does,
    // so the Task 4 module split cannot silently change it.

    #[test]
    fn percent_decode_full_url() {
        assert_eq!(
            percent_decode_str("https%3A%2F%2Fexample.com%2Fa%3Fb%3D1"),
            "https://example.com/a?b=1"
        );
    }

    #[test]
    fn percent_decode_plus_as_space() {
        // Characterization: the implementation converts '+' to a space
        // (query-string form decoding), not just %XX escapes.
        assert_eq!(percent_decode_str("a+b"), "a b");
    }

    // ---- validated_external_url ----

    #[test]
    fn validated_external_url_accepts_http_and_https() {
        assert_eq!(
            validated_external_url("https://example.com/a?b=1").unwrap(),
            "https://example.com/a?b=1"
        );
        assert_eq!(
            validated_external_url("http://example.com").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn validated_external_url_rejects_non_http_schemes() {
        assert!(validated_external_url("file:///etc/passwd").is_err());
        assert!(validated_external_url("javascript:alert(1)").is_err());
        assert!(validated_external_url("data:text/html,<h1>x</h1>").is_err());
    }

    #[test]
    fn validated_external_url_rejects_malformed() {
        // No scheme at all
        assert!(validated_external_url("example.com").is_err());
        // Scheme-like prefix but not an absolute http(s) URL
        assert!(validated_external_url("https://").is_err());
        assert!(validated_external_url("http:/missing-slash").is_err());
        // Empty / whitespace
        assert!(validated_external_url("").is_err());
        assert!(validated_external_url("   ").is_err());
    }

    #[test]
    fn validated_external_url_rejects_overlong_payloads() {
        // 2048 bytes is the limit; anything longer must be refused.
        let long_path = "/".repeat(2100);
        let url = format!("https://example.com{}", long_path);
        assert!(validated_external_url(&url).is_err());
        // Exactly 2048 bytes must still be accepted.
        let ok_path_len = 2048 - "https://example.com".len();
        let ok_url = format!("https://example.com{}", "/".repeat(ok_path_len));
        assert_eq!(ok_url.len(), 2048);
        assert_eq!(validated_external_url(&ok_url).unwrap(), ok_url);
    }
}
