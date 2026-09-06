# Local Store design notes

This document records the project's original direction and development history.
Current architecture, commands and registry paths are documented in
[README.md](README.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
The repository is [Local Store](https://github.com/madhavsonkusare-a11y/local-store).
Git history preserves the original implementation and release terminology.

## Product direction

Bring self-hosted apps closer to the desktop: discover projects, connect existing
web interfaces, and open them in dedicated native windows. Adapt reviewed Docker
Compose recipes where installation can be supported and tested. Reuse Tauri and
the operating system's webview to keep the launcher small.

An app's project website is distinct from its running instance. Links outside
that origin open in the user's browser. Local Store keeps connection and managed
app records on the user's computer.

## Development checkpoints

- **v0.1:** native windows, a local app registry, external-link interception,
  a small Node CLI, and a Windows GUI-subsystem executable.
- **v0.2:** Rust CLI, Compose boot and health checks, application shortcuts,
  custom URL handling, and per-app title-bar icons.
- **v0.3:** native path handling across operating systems, registry tests,
  version commands, launcher configuration and macOS URL-scheme registration.
- **v0.4:** an embedded 1,257-entry discovery snapshot, catalog setup wizard,
  twelve reference recipe definitions and expanded remote icon coverage.
- **v0.5 preview:** the versioned registry, migration/recovery, managed lifecycle,
  a dark-only visual workspace, the approved L-and-tile identity, 1,672 discovery
  projects, 508 bundled catalog icons, combined filters and Settings/Doctor.
  Three recipes remain install previews pending real-container lifecycle tests.

## Current design rules

Use the Local Store product name, `local-store` executable/config slug and
`localstore://` launch scheme. Keep the approved top alignment, equal facing
gaps and concentric corner geometry; canonical assets live in `branding/`.
Build on the existing Inter, Lucide, native dialogs and restrained motion system.

The [33-task ledger](docs/upgrade-status.md) tracks outstanding runtime, native
integration, installer, signing and update work. Successful builds and mocked
tests do not replace those release gates.
