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

- Apps are registered (name + URL) and stored in `%APPDATA%/dockwrap/apps.json`.
- The launcher lists them; clicking **Open** spawns a native window to that URL.
- A tiny injected script intercepts `window.open` and external `<a>` clicks,
  rewriting the navigation to a `localhost` marker URL. Rust catches that in
  `on_navigation` and launches the OS default browser.

## Build

```bash
# Rust + Tauri v2 toolchain required
cargo tauri build          # produces release binary + installer
# or just run it:
cargo run
```

On first launch an example app (Penpot on `localhost:9001`) is seeded so the
launcher is useful immediately. Add your own via the UI.

## Roadmap

- [ ] `dockwrap add <name> --url <u> --icon <i>` Rust CLI (replaces `cli.js`)
- [ ] Docker Compose boot: `docker compose up -d` + health check before open
- [ ] Start Menu shortcut generation with the app's icon
- [ ] `dockwrap://open/<name>` protocol handler
- [ ] Per-app icon on the native window title bar

## License

MIT — see [LICENSE](LICENSE).
