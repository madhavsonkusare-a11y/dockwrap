# Local Store

Discover self-hosted software, install reviewed local recipes, connect the
instances you already run, and open them in dedicated desktop windows.

Point Local Store at any local web app — Penpot, your homelab dashboard, a
self-hosted tool — and it opens in a real native window with that app's name
and icon. External links (help docs, community, anything `http(s)://` not on
`localhost`) open in your **default browser**, not inside the app frame.

## Why

- **Nativefier** bundles a full Chromium per app (~150 MB each) and is
  Linux/macOS-first with spotty maintenance.
- **WebCatalog** is closed-source and commercial.
- **No maintained tool** ties a self-hosted Docker Compose stack to a one-click
  native window.

Local Store fills that gap: small Tauri binary, WebView2 on Windows, open source,
MIT.

## How it works

- **Discover** searches the embedded project catalog in bounded pages. A project
  website is presented as a source link and is never treated as your instance.
- **Verified install** currently supports Memos with a pinned container image,
  a Docker/Compose preflight check, persistent local data, health verification,
  and rollback when setup fails.
- **Connect an app** saves its name and reachable HTTP(S) address. Local Store
  does not seed an example or imply that catalog projects are already installed.
- **My Apps** opens connections and starts, stops, inspects, or uninstalls apps
  managed by Local Store. Uninstall preserves app data unless deletion is
  explicitly selected and confirmed.
- Apps use a versioned registry at `%APPDATA%/local-store/registry-v2.json` on
  Windows (or the platform config directory elsewhere). Existing v1 and legacy
  registries are imported once with a backup.
- The launcher lists them; clicking **Open** spawns a native window to that URL.
- A tiny injected script intercepts `window.open` and external `<a>` clicks,
  rewriting the navigation to a `localhost` marker URL. Rust catches that in
  `on_navigation` and launches the OS default browser.
- Apps with a `compose` file boot their Docker stack and wait for a health
  check before opening.

## Build

```bash
# Rust + Tauri v2 toolchain required
cargo tauri build          # produces release binary + installer
# or just run it:
cargo run
```

Add your own connection through the launcher or CLI:

```bash
local-store doctor
local-store install memos
local-store add penpot --url http://localhost:9001
local-store list
local-store status memos
local-store logs memos
local-store stop memos
local-store start memos
local-store open memos --browser
local-store shortcut memos
local-store remove penpot
local-store uninstall memos
local-store --version
```

Compatibility (one release): legacy dockwrap registry and dockwrap:// deep links are imported/recognized.

## Roadmap

### v0.2 (shipped ✅)
- [x] `local-store add <name> --url <u> --icon <i>` Rust CLI (replaces `cli.js`)
- [x] Docker Compose boot: `docker compose up -d` + health check before open
- [x] Start Menu shortcut generation with the app's icon
- [x] `localstore://open/<name>` protocol handler
- [x] Per-app icon on the native window title bar

### v0.3 (shipped ✅)
- [x] **Cross-platform registry path** — `apps.json` now uses `PathBuf` (was hardcoded `\`, broken on Linux/macOS)
- [x] **Unit tests** for the registry (path, dedup, preset lookup)
- [x] **`local-store --version`** / `local-store version` subcommand
- [x] **GUI parity with CLI** — icon, compose, and health inputs; per-row Remove button; app icon + 🐳 compose badge in the launcher
- [x] **macOS `localstore://`** registered via bundle `Info.plist` (`CFBundleURLTypes`)

### v0.4 (shipped ✅ — current release)
- [x] **Embedded app catalog** — 1,257 self-hosted app entries bundled into the binary
- [x] **Catalog-backed setup wizard** — browse and configure catalog apps from the launcher
- [x] **Reference recipe data** — 12 curated entries document Compose and health-check metadata for future integration
- [x] **Broad icon coverage** — verified icon sources plus favicon fallback for entries without one

### v0.5 (in progress)
- [x] Versioned v2 registry with one-time legacy migration and recovery
- [x] Reviewed Memos recipe with pinned image and persistent data
- [x] Docker doctor, transactional install, health verification, and rollback
- [x] Managed app status, start, stop, logs, and data-preserving uninstall
- [ ] Expand reviewed recipes after end-to-end recipe validation

## License

MIT — see [LICENSE](LICENSE).
