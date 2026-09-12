//! Windowing: webview window construction, the external-link bridge JS, and
//! external-URL validation/policy for routing links to the OS browser.

use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

/// Injected into every webview. Intercepts window.open + clicks on external
/// links and rewrites the navigation to a localhost marker URL that Rust
/// catches in `on_navigation`, then launches the OS default browser.
/// The marker a bridged link navigates to, which Rust catches and turns into
/// an OS browser launch. Port 65535 on loopback is never served by anything;
/// the navigation is intercepted before it can be attempted.
pub const EXTERNAL_BRIDGE_PREFIX: &str = "http://127.0.0.1:65535/.external?";

/// What an app window is allowed to do with a navigation it asked for.
///
/// An app window shows somebody else's web application. It holds no IPC
/// permissions — the capability grant names only the launcher — but it can
/// still try to navigate itself somewhere, and where it may go is a decision
/// worth making in one place that can be read and tested.
#[derive(Debug, PartialEq, Eq)]
pub enum Navigation {
    /// Same origin as the app was opened at: this is the app, so let it move.
    Allow,
    /// Somewhere else on the web. It opens in the OS browser, where the person
    /// can see the address bar, rather than inside a window titled after their
    /// app.
    OpenExternally(String),
    /// Not a web page at all. `file:`, `javascript:` and every custom scheme a
    /// page might reach for are refused outright rather than handed to the OS,
    /// which would decide what to run.
    Block,
}

/// Decide a navigation without performing it.
///
/// `origin` is the ASCII serialization of the origin the window was opened at.
/// Kept free of Tauri state so the policy can be exercised directly.
pub fn decide_navigation(url: &tauri::Url, origin: &str) -> Navigation {
    if let Some(query) = url.as_str().strip_prefix(EXTERNAL_BRIDGE_PREFIX) {
        return Navigation::OpenExternally(percent_decode_str(query));
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Navigation::Block;
    }
    if url.origin().ascii_serialization() != origin {
        return Navigation::OpenExternally(url.as_str().to_owned());
    }
    Navigation::Allow
}

pub const LINK_BRIDGE_JS: &str = r#"
(function(){
  if (window.__localStoreBridge) return;
  window.__localStoreBridge = true;
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
/// Reached from `on_navigation`, `build_window` and the `open_project` command,
/// so every URL that can leave the app passes through here.
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

/// Tell the launcher that a link could not be opened.
///
/// The failure happens inside an app window, which owns none of our UI and
/// must never be sent launcher payloads, so the report goes to the launcher.
/// Delivery is best effort: a missing launcher must not crash the app window.
pub fn report_browser_failure(app: &tauri::AppHandle, error: &crate::error::AppError) {
    use tauri::Emitter;
    let _ = app.emit_to(
        tauri::EventTarget::webview_window("launcher"),
        BROWSER_FAILURE_EVENT,
        error,
    );
}
pub const BROWSER_FAILURE_EVENT: &str = "local-store://browser-failure";

pub fn build_window(
    app: &tauri::AppHandle,
    id: &str,
    name: &str,
    url: &str,
    icon: Option<&str>,
) -> Result<(), String> {
    let url = validated_external_url(url)?;
    let parsed = url.parse::<tauri::Url>().map_err(|e| e.to_string())?;
    if !crate::model::is_valid_installed_app_id(id) {
        return Err("Invalid app ID for window.".into());
    }
    let label = format!("app-{id}");
    if let Some(window) = app.get_webview_window(&label) {
        window.show().map_err(|e| e.to_string())?;
        return window.set_focus().map_err(|e| e.to_string());
    }
    let origin = parsed.origin().ascii_serialization();
    let builder = WebviewWindowBuilder::new(app, &label, WebviewUrl::External(parsed))
        .title(name)
        .inner_size(1200.0, 800.0)
        .resizable(true)
        .initialization_script(LINK_BRIDGE_JS)
        .on_navigation({
            let handle = app.clone();
            move |url| match decide_navigation(url, &origin) {
                Navigation::Allow => true,
                Navigation::Block => false,
                Navigation::OpenExternally(target) => {
                    if let Err(error) = launch_browser(&target) {
                        report_browser_failure(&handle, &error);
                    }
                    false
                }
            }
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

    fn origin_of(url: &str) -> String {
        url.parse::<tauri::Url>()
            .unwrap()
            .origin()
            .ascii_serialization()
    }
    fn decide(url: &str, opened_at: &str) -> Navigation {
        decide_navigation(&url.parse::<tauri::Url>().unwrap(), &origin_of(opened_at))
    }

    /// An app window is the app, and nothing else. It holds no IPC permission
    /// — the capability grant names only the launcher — so this is the whole
    /// of what it can still reach for on its own.
    #[test]
    fn an_app_window_may_move_within_its_own_app_and_nowhere_else() {
        let app = "http://localhost:5230";
        // Its own pages, including paths, queries and fragments.
        for target in [
            "http://localhost:5230",
            "http://localhost:5230/notes",
            "http://localhost:5230/notes?tag=a#top",
        ] {
            assert_eq!(decide(target, app), Navigation::Allow, "{target}");
        }

        // Another origin is somebody else's site. It opens where a person can
        // see the address bar, not inside a window titled after their app.
        for target in [
            "https://example.com/",
            "http://localhost:5231/",
            "https://localhost:5230/",
            "http://127.0.0.1:5230/",
        ] {
            assert_eq!(
                decide(target, app),
                Navigation::OpenExternally(target.to_owned()),
                "{target}"
            );
        }
    }

    /// Anything that is not a web page is refused rather than handed to the
    /// OS, which would otherwise decide what to run.
    #[test]
    fn a_navigation_that_is_not_a_web_page_is_refused_outright() {
        let app = "http://localhost:5230";
        for target in [
            "file:///C:/Windows/System32/drivers/etc/hosts",
            "javascript:alert(1)",
            "data:text/html,<script>alert(1)</script>",
            "ms-settings:privacy",
            "vbscript:msgbox(1)",
            "localstore://open/immich",
            "smb://server/share",
        ] {
            assert_eq!(decide(target, app), Navigation::Block, "{target}");
        }
    }

    /// The bridge is how a page asks for the browser. It carries a target, so
    /// what it carries has to be checked before it is launched.
    #[test]
    fn the_external_bridge_hands_its_target_on_without_following_it() {
        let app = "http://localhost:5230";
        let bridged = format!("{EXTERNAL_BRIDGE_PREFIX}https%3A%2F%2Fexample.com%2Fa%3Fb%3D1");
        assert_eq!(
            decide(&bridged, app),
            Navigation::OpenExternally("https://example.com/a?b=1".into())
        );
        // Whatever comes out is still put through the same URL validation the
        // launcher uses before anything is launched.
        assert!(validated_external_url("https://example.com/a?b=1").is_ok());
        for hostile in ["javascript:alert(1)", "file:///etc/passwd"] {
            assert!(validated_external_url(hostile).is_err(), "{hostile}");
        }
    }

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
