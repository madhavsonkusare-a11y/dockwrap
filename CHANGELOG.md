# Changelog

## Unreleased

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
  `neosmemo/memos:0.30.0` with persistent local data and private mode.
- Add Docker and Compose preflight diagnostics, transactional installation,
  HTTP health verification, failure rollback, and managed start, stop, status,
  logs, and uninstall operations.
- Add a reviewed installation UI that exposes the exact image, address, data
  location, prerequisites, and risks before changes are made.

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
