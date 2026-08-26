# ============================================================
# dockwrap v0.1 Design Notes
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
#   default browser (via /__external marker bridge), keeping the design window
#   intact.
#
# v0.1 scope (in-vault MVP — real, builds):
#   - Tauri shell: `cargo tauri build` -> dockwrap.exe
#   - Launcher window listing registered apps (reads %APPDATA%\dockwrap\apps.json)
#   - Click any app -> new native window to its URL
#   - External-link interceptor injected into each window (link-handler.js)
#   - Node CLI `cli.js register <name> --url <u> --icon <i>` writes config
#   - GUI subsystem PE = 2 (no terminal window), Penpot logo icon
#
# v0.2 (planned):
#   - `dockwrap init <name> --url <u> --icon <i>` in Rust (replaces cli.js)
#   - Docker Compose boot: read compose dir, `docker compose up -d`, health check
#   - Start Menu shortcut generation with app icon
#   - Protocol handler dockwrap://open/<name>
#
# Lessons from Penpot build applied here:
#   - External links fail via window.open in WebView2 -> use /__external marker
#   - PE subsystem GUI prevents console windows on Windows launch
#   - on_navigation handler is the proven bridge for external URLs
#
# Status: scaffolded. v0.1.0
