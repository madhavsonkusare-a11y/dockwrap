# Contributing

dockwrap is small by design. Contributions welcome.

## Dev setup

```bash
# Rust + Tauri v2 toolchain
cargo install tauri-cli --version "^2" --locked
cargo run            # launches the launcher
cargo tauri build    # release binary + installer in target/release/bundle
```

## Conventions

- Keep it dependency-light. The current footprint is Tauri, serde, serde_json,
  and narrowly scoped platform APIs; add dependencies only with a clear need.
- External-link routing lives in `src/main.rs` (`LINK_BRIDGE_JS` + `on_navigation`).
  Don't break that contract — it's the one feature that separates dockwrap
  from a bare webview.
- App registry is `%APPDATA%/dockwrap/apps.json` (or `~/.config/dockwrap/`
  on Linux/macOS). Treat it as the single source of truth.
- GUI-subsystem binary on Windows: keep `#![cfg_attr(not(debug_assertions),
  windows_subsystem = "windows")]` — no console window on launch.

## Before opening a PR

- `cargo build` is clean
- `cargo test` passes
- README roadmap item checked off if you shipped a feature

## Current focus

The v0.4.0 release added the embedded catalog and setup wizard. Keep follow-up
work small and focused on catalog quality, reliability, or post-v0.4 roadmap
items as they are defined in the README.
