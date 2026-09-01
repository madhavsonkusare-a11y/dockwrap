# dockwrap

Native desktop shells for self-hosted web apps. One binary, no Electron.

Point dockwrap at any local web app — Penpot, your homelab dashboard, a
self-hosted tool — and it opens in a real native window with that app's name
and icon. External links (help docs, community, anything `http(s)://` not on
`localhost`) open in your **default browser**, not inside the app frame.

## Why

- **Nativefier** bundles a full Chromium per app (~150 MB each) and is
  Linux/macOS-first with spotty maintenance.
- **WebCatalog** is closed-source and commercial.
- **No maintained tool** ties a self-hosted Docker Compose stack to a one-click
  native window.

dockwrap fills that gap: small Tauri binary, WebView2 on Windows, open source,
MIT.

## How it works

- Apps are registered (name + URL, optional icon/compose/health) in a registry
  file (`%APPDATA%/dockwrap/apps.json` on Windows, `~/.config/dockwrap/apps.json`
  on Linux/macOS).
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

On first launch an example app (Penpot on `localhost:9001`) is seeded so the
launcher is useful immediately. Add your own via the UI or the CLI:

```bash
dockwrap add penpot --url http://localhost:9001 --icon /path/to/penpot.png
dockwrap add n8n --preset n8n --compose /opt/n8n/docker-compose.yml
dockwrap list
dockwrap open n8n
dockwrap shortcut n8n
dockwrap remove penpot
dockwrap --version
```

## Roadmap

### v0.2 (shipped ✅)
- [x] `dockwrap add <name> --url <u> --icon <i>` Rust CLI (replaces `cli.js`)
- [x] Docker Compose boot: `docker compose up -d` + health check before open
- [x] Start Menu shortcut generation with the app's icon
- [x] `dockwrap://open/<name>` protocol handler
- [x] Per-app icon on the native window title bar

### v0.3 (shipped ✅)
- [x] **Cross-platform registry path** — `apps.json` now uses `PathBuf` (was hardcoded `\`, broken on Linux/macOS)
- [x] **Unit tests** for the registry (path, dedup, preset lookup)
- [x] **`dockwrap --version`** / `dockwrap version` subcommand
- [x] **GUI parity with CLI** — icon, compose, and health inputs; per-row Remove button; app icon + 🐳 compose badge in the launcher
- [x] **macOS `dockwrap://`** registered via bundle `Info.plist` (`CFBundleURLTypes`)

### v0.4 (shipped ✅ — current release)
- [x] **Embedded app catalog** — 1,257 self-hosted app entries bundled into the binary
- [x] **Catalog-backed setup wizard** — browse and configure catalog apps from the launcher
- [x] **Reference recipe data** — 12 curated entries document Compose and health-check metadata for future integration
- [x] **Broad icon coverage** — verified icon sources plus favicon fallback for entries without one

### Post-v0.4
- [ ] Wire verified Compose and health-check recipes into the runtime catalog and setup wizard
- [ ] Define the next roadmap after catalog feedback and maintenance work

## License

MIT — see [LICENSE](LICENSE).
