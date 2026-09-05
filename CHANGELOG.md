# Changelog

## Unreleased

- Refine the Local Store identity with a larger coral tile centered on both axes,
  consistent mark geometry, regenerated platform icons, and a five-panel brand deck.
- Make the launcher dark-only, with warm gradients, a visual featured app shelf,
  bundled app identities, and refined card, form, and dialog spacing.
- Add interruptible dialog transitions and immediate keyboard/reduced-motion
  behavior using native dialogs and the Web Animations API.
- Bind reviewed recipes to loopback, expose storage accurately, and constrain
  managed data deletion to the app's resolved directory.

- Add the Discover, Connect, and My Apps desktop experience with bounded catalog
  search, honest capability labels, responsive layouts, and accessible dialogs.
- Stop treating discovery source URLs as local launch addresses.
- Add fallible atomic launcher storage, explicit operation errors, launcher-only
  Tauri command permissions, CSP, and safe cross-platform browser opening.
- Add Playwright interaction, axe accessibility, and Windows visual regression
  coverage; require frontend and Rust quality gates before platform builds.
- Add the versioned v2 app registry with one-time migration, immutable legacy
  backup, atomic updates, and strict recovery behavior.
- Add the first reviewed install recipe for Memos, pinned to
  `neosmemo/memos:0.30.0` with persistent local data.
- Add Docker and Compose preflight diagnostics, transactional installation,
  HTTP health verification, failure rollback, and managed start, stop, status,
  logs, and uninstall operations.
- Add a reviewed installation UI that exposes the exact image, address, data
  location, prerequisites, and risks before changes are made.
- Graduate n8n 2.37.10 and Uptime Kuma 2.5.3 as the second and third reviewed
  recipes, with exact image pins, app-owned volumes, audit metadata, and
  Docker Compose syntax validation in CI.
- Allow a preserved managed data directory to be reused on reinstall while
  restoring its previous Compose file if setup fails.

All notable changes to this project are documented in this file.

## [Unreleased]

## [0.4.0] - 2026-08-29

### Changed
- Catalog entries gained a `favicon_url` fallback field so every one of the
  1,257 entries renders an icon (verified sources plus favicon fallback;
  coverage raised from ~48% to 100%).
- Launcher and setup wizard render the favicon fallback when no verified
  icon exists.

### Fixed
- Icons no longer appear broken for catalog apps without a directly verified
  icon.
