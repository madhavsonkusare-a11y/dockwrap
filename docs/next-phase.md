# Discover, connect, and use

## Scope — September 4, 2026

Build a complete connection-first desktop experience on the existing Tauri v2
application. This phase follows the foundation work in the 33-task upgrade plan
and implements catalog paging and the connected-app portions of its UI tasks.
Verified recipe installation is deliberately not advertised until its runtime
and recipes pass the later installation gates.

## Reuse and integration

- Retain Rust/Tauri, vanilla JavaScript, the bundled Inter font, approved Local
  Store mark, existing 1,257-entry catalog, and registry compatibility.
- Cache and query the existing catalog in Rust. Return discovery metadata with
  `source_url`, never an implied installation or local launch address.
- Use the maintained `open` crate for OS browser opening, and Playwright plus
  axe-core for repeatable interaction and accessibility checks.
- Keep the existing V1 runtime registry until the V2 migration is wired through
  CLI, shortcuts, health metadata, and runtime together. Add fallible GUI access
  to the same file rather than introduce a competing store.

## Experience

Paper-colored workspace, ink sidebar, restrained orange selection, generous
spacing, readable app descriptions, and original app identities. No fabricated
ratings, availability indicators, recommendations, or installation counts.

1. Discover: search, category filter, bounded pages, project details, clear
   connection-only capability, loading/error/empty states.
2. Connect: named URL with explicit HTTP(S) validation; never prefill an upstream
   project URL. Preserve input on failure and refuse duplicate names.
3. My Apps: open, shortcut, remove with confirmation, clear operation feedback.
   Removal only removes the saved connection; it never deletes app data.
4. Keyboard: semantic controls, native modal focus management, visible focus,
   Escape dismissal, live feedback, responsive 800x600 minimum desktop layout.

## Gates

- Existing and new Rust tests, format, clippy, production build.
- Browser tests exercise the production frontend with a controlled Tauri adapter;
  these are not evidence of native IPC or platform installer behavior.
- Visual inspection at 1280x800 and 800x600; axe accessibility scan.
- Review changes, commit, push feature branch (including existing ancestor
  commits); no force push, main merge, release tag, or deployment.

## Follow-on phase

Wire V2 storage throughout the application, complete verified recipe manifests
and transactional installs, then implement install/start/stop/logs UX against the
real runtime. Validate three recipes end to end before enabling Install buttons.
Cross-platform installers, signing and update rollout remain separate gates.
