# ============================================================
# dockwrap Design Notes
# ============================================================
# Problem:
#   - Existing "wrap a website into a desktop app" tools (Nativefier,
#     WebCatalog) assume the web app is already running externally.
#   - They bundle Electron (150MB+ per app) and lack external-link routing.
#   - No maintained tool ties Docker Compose boot to a native window.
#
# dockwrap vision:
#   One binary, GUI subsystem (no console on Windows), loads any local web app
#   as a native window with a custom name + icon. External links open in your
#   default browser (via the /__external marker bridge), keeping the design window
#   intact.
#
# v0.1 scope (historical foundation):
#   - Tauri shell: `cargo tauri build` -> dockwrap.exe
#   - Launcher window listing registered apps (reads %APPDATA%\dockwrap\apps.json)
#   - Click any app -> new native window to its URL
#   - External-link interceptor injected into each window (link-handler.js)
#   - Node CLI `cli.js register <name> --url <u> --icon <i>` writes config
#   - GUI subsystem PE = 2 (no terminal window), Penpot logo icon
#
# v0.2 (shipped):
#   - Rust CLI replaces cli.js (add/list/remove/open/shortcut/presets)
#   - Docker Compose boot + health check before open
#   - Start Menu / .desktop shortcut generation with the app's icon
#   - dockwrap://open/<name> protocol handler (Windows registry, Linux xdg,
#     macOS bundle Info.plist CFBundleURLTypes)
#   - Per-app title-bar icon
#
# Lessons from Penpot build applied here:
#   - External links fail via window.open in WebView2 -> use /__external marker
#   - PE subsystem GUI prevents console windows on Windows launch
#   - on_navigation handler is the proven bridge for external URLs
#
# v0.3 (shipped):
#   - FIX: registry path used hardcoded '\' -> broken on Linux/macOS.
#     Now uses std::path::PathBuf for a native, correct path on every OS.
#   - Added unit tests for the registry (path separator, dedup, preset lookup).
#   - `dockwrap --version` / `dockwrap version` subcommand.
#   - GUI parity with the CLI: icon + compose + health inputs, per-row Remove
#     button, app icon thumbnail and a 🐳 compose badge in the launcher list.
#   - macOS dockwrap:// registration via src/Info.plist (CFBundleURLTypes)
#     referenced from tauri.conf.json bundle.macOS.infoPlist.
#
# v0.4 (shipped as v0.4.0):
#   - Embedded 1,257 self-hosted app catalog entries in the binary.
#   - Added the catalog-backed launcher setup wizard.
#   - Added curated Compose/health metadata for selected apps.
#   - Expanded icon coverage with verified sources and favicon fallback.
#
# Status: v0.4.0 — current released version; catalog-enhanced, cross-platform,
# tested, and distributed through tag-triggered CI releases.
