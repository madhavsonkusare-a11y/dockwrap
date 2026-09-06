# Contributing

Local Store is small by design. Contributions welcome.

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
- External-link routing lives in `src/windowing.rs`. Preserve origin checks and
  the launcher-only Tauri capability boundary.
- App registry is `%APPDATA%/local-store/registry-v2.json` on Windows, under the
  platform config directory elsewhere. Preserve migration/recovery semantics.
- GUI-subsystem binary on Windows: keep `#![cfg_attr(not(debug_assertions),
  windows_subsystem = "windows")]` — no console window on launch.

## Before publishing changes

- `cargo fmt --all -- --check`, clippy with warnings denied, and `cargo test --locked`
- `npm ci` and `npm test` (Windows visual baselines; Linux CI checks interactions/axe)
- Offline catalog/icon/brand checks described in [the catalog guide](docs/catalog.md)
- Update the current [task ledger](docs/upgrade-status.md) with actual evidence

For the next bounded development batch, start with the
[agent handoff](docs/agent-handoff.md). It contains scope, acceptance criteria,
validation commands and current limitations without requiring chat history.

## Current focus

The v0.5 preview combines discovery, connected instances and three install
previews. Reuse upstream metadata and deployment instructions, then adapt and
test recipes individually. Never promote an imported Compose file automatically.
Runtime reliability, native integration and clean-machine verification are next.
