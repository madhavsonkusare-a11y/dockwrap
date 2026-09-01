//! Windowing: webview window construction, the external-link bridge JS, and
//! external-URL validation/policy for routing links to the OS browser.

use tauri::{WebviewUrl, WebviewWindowBuilder};

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

/// Validate a URL handed to us for external navigation before it may reach the
/// OS browser launcher. Accepts only absolute http(s) URLs of at most 2048
/// bytes; anything else (file:, javascript:, data:, malformed or oversized
/// payloads) is rejected. Returns the URL unchanged when valid.
// TODO(Task 16): wire into on_navigation and the launcher path; until then this
// is only exercised by tests.
#[allow(dead_code)]
pub fn validated_external_url(url: &str) -> Result<String, String> {
    if url.len() > 2048 {
        return Err(format!(
            "external URL too long ({} bytes, max 2048)",
            url.len()
        ));
    }
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| format!("not an absolute URL: {:?}", url))?;
    if !scheme.eq_ignore_ascii_case("http") && !scheme.eq_ignore_ascii_case("https") {
        return Err(format!("unsupported scheme {:?} (only http/https)", scheme));
    }
    // The authority (host[:port]) must be non-empty: take everything up to the
    // first path/query/fragment separator and require it to be present.
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("").trim();
    if authority.is_empty() {
        return Err(format!("missing host in URL: {:?}", url));
    }
    Ok(url.to_string())
}

/// Open a URL in the user's default browser via the runtime module.
pub use crate::runtime::launch_browser;

pub fn build_window(app: &tauri::AppHandle, label: &str, url: &str, icon: Option<&str>) {
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
