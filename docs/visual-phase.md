# Visual workspace — September 5, 2026

This phase follows the Discover/Connect and managed runtime work. It replaces
the light workspace direction in the earlier phase plan.

## Delivered

- Review the original five-panel Penpot brand deck. Retain its L-and-tile concept
  and Local/Store wordmark; enlarge the coral tile and center it on both axes.
- Unify mark and app-icon geometry, add a flat monochrome source, export desktop
  icons, and add a refined five-panel HTML brand deck to the repository.
- Use dark-only charcoal surfaces, warm gradient actions, a translucent toolbar,
  clearer spacing, larger app identities, and a functional featured app shelf.
- Reuse bundled Inter, Lucide, native dialogs, the Web Animations API, and
  Homarr's existing app icons. No new frontend framework or animation dependency.
- Apply Emil Kowalski's `emil-design-eng`, `apple-design`, and `animate` skills:
  brief pointer feedback, interruptible modal transitions, immediate keyboard and
  reduced-motion behavior, no animated catalog lists.
- Finish n8n and Uptime Kuma recipe integration alongside Memos, including
  catalog inclusion, loopback ports, persistent storage, and explicit reviews.

## Verification

Rust runtime/storage tests use fake process and health adapters. Browser tests
exercise the production frontend through a controlled Tauri adapter. Compose
validation checks configuration syntax without starting containers.

Visual baselines cover desktop, compact desktop, and narrow layouts, plus
connection, installation review, and My Apps surfaces. Axe checks include the
workspace and dialogs; behavior checks cover keyboard focus, reduced motion,
blocked prerequisites, and operation failures.

Validated on Windows: 63 Rust tests, 14 Playwright tests (including axe and six
visual baselines), format, clippy with warnings denied, all three Compose
configurations, and the release binary build.

## Next release gate

Run real-container installs, restart/reinstall with preserved data, and
clean-machine installer checks on Windows, macOS, and Linux. Those remain
distinct from browser fixtures and command-adapter tests. Signing and update
rollout follow successful installer validation.
