# Changelog

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
