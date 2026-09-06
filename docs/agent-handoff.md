# Local Store: agent handoff

Updated September 6, 2026. This file is intended for any coding agent or human
maintainer. Read it alongside `docs/upgrade-status.md`, the versioned ledger of
all 33 original upgrade tasks. The ignored `.hermes/` workspace is not needed
to continue development.

## Working agreement

- Product: **Local Store**. GitHub: https://github.com/madhavsonkusare-a11y/local-store.
- The owner requested commits and pushes directly to `main`, without a PR.
  Inspect status and fetch before editing; preserve unrelated work and never
  force-push. Hermes was paused for this session; coordinate before another
  agent starts editing the same checkout.
- Keep batches small enough to finish implementation, validation, documentation
  and publication with usage remaining. If allowance is tight, select a smaller
  batch before editing. Leave no partially implemented feature at a checkpoint.
- Reuse established libraries and upstream deployment instructions. Do not
  automatically turn discovery entries into installable recipes.
- UI is dark-only. Preserve the approved logo, gradients, spacing, restrained
  motion, keyboard behavior and reduced-motion support. For future UI work,
  apply Emil Kowalski and Apple design guidance where available; use existing
  components and motion helpers first.
- The local checkout directory still has its historical name. Do not rename it
  while tools are using it. The remaining legacy constants and URL scheme in
  `src/brand.rs` and `src/Info.plist` intentionally support migration and old links.

## Current checkpoint

- Main baseline `5df4651` renamed the GitHub repository and in-repo identity.
- This follow-up fixes Chromium startup in Linux CI. The old configuration put
  all browser temp files inside the checkout. The longer renamed checkout path
  exceeded Chromium's Unix socket path limit; all 19 tests failed at launch in
  run `34030915173`, before any UI assertions. The workspace temp override is
  now Windows-only; POSIX keeps the OS temp directory. Failure traces are
  uploaded by CI for seven days. Do not restore a checkout-relative POSIX TMPDIR.
- Local validation for this repair: all 19 Playwright tests passed on Windows,
  including the existing visual baselines, keyboard and accessibility checks.
  Remote quality and native build status must be read from the workflow for the
  repair commit; the old failed run is historical and will stay red.
- No Docker runner implementation changes were started in this batch. The
  process-library research is preliminary; no dependency was selected or added.
- The ledger remains **16 complete, 14 partial, two not implemented, one final
  gate not run**. Seventeen tasks still need work; this is not an effort estimate.
- Discovery contains **1,672 projects** and **508 offline catalog SVGs** from four
  pinned sources. Only **three install previews** exist: Memos, n8n, Uptime Kuma.
  Their real install/restart/upgrade/preserve/delete lifecycle is not yet proven.
- Approved logo geometry at 512 px: tile `(193,95)`, size `224`, radius `56`;
  L stroke `56`, equal gaps `42`, shared facing-corner center `(249,263)`, radii
  `56` and `98`. Top edges align at `95`. `scripts/generate-brand.mjs --check`
  verifies generated exports. Do not redesign the approved geometry.

## Next batch: bounded process execution (task 13)

Implement this independently before adding progress UI or more recipes.

1. Read `src/runtime/mod.rs`, particularly `CommandSpec`, `ProcessRunner`,
   `SystemProcessRunner`, Doctor and lifecycle callers. Preserve the injectable
   runner used by existing tests. It currently calls `Command::output()` without
   a deadline and captures unbounded output.
2. Select an established process-group/job-object library after checking its
   license, maintenance, API and minimum Rust version. `command-group` and
   `process-wrap` are candidates, not decisions. The current manifest declares
   Rust 1.77.2 while `rust-toolchain.toml` pins a newer compiler; MSRV is not
   verified by CI. Do not silently raise it to adopt a dependency.
3. Apply explicit command-specific deadlines: short diagnostics/status,
   longer image download/start operations. Drain stdout and stderr concurrently
   into bounded buffers. Preserve useful failure output and signal truncation.
4. On timeout/cancellation, terminate the owned child process tree and reap it;
   do not terminate the user's Docker daemon or unrelated containers. Account
   for a grandchild holding an inherited output pipe after the parent exits.
   Keep argument arrays and the existing hidden Windows console behavior.
5. Add a cancellation primitive at the runner boundary. Document that operation
   and UI cancellation remain separate work until wired through transactions.

Acceptance criteria: deterministic tests with real controlled child processes
cover success, nonzero exit, missing executable, timeout, cancellation,
simultaneous oversized stdout/stderr, UTF-8 truncation and inherited pipes.
Tests must not require Docker or alter user app data. Run these tests on
Windows, macOS and Linux in CI. No unbounded reader-thread join after timeout.
Keep task 13 partial until redaction and all remaining Doctor work are complete.

## Following batches, in order

1. **Machine-readable diagnostics (tasks 13/20):** add `local-store doctor
   --json` using the existing `DoctorReport` schema; stdout must contain only
   JSON, readiness must determine exit status, and invalid options must fail.
   Test both healthy and missing-Docker reports through injected runners.
   Finish secret redaction before exposing captured process output in events.
2. **Install transaction (14/19):** typed error codes, per-app operation locks,
   port preflight, atomic Compose writes and rollback if registry commit fails.
   Preserve existing data on every failure; test each boundary. Review current
   install/rollback in `src/runtime/mod.rs` and commits in `src/commands.rs`
   and `src/cli.rs`. Health waits also need an operation-wide cancellation path.
3. **Operation UX (24/25):** progress events, retry diagnostics, cancel and
   readiness states using the existing frontend modules. Validate keyboard,
   reduced motion and interrupted transitions; retain current visual baselines.
4. **Native integration (16/17/27):** reuse official Tauri single-instance and
   deep-link plugins; verify existing-process delivery, remote-window isolation
   and browser-opening errors. Review release devtools/capabilities.
5. **Recipe proof (11/12/15):** use isolated data roots and ports to verify the
   original three recipes through install, health, restart, upgrade, preserve
   and explicit data deletion on supported platforms. Record image versions and
   results. Then adapt upstream recipes in small batches toward 10 and 25
   verified installs. Refresh discovery independently; keep provenance/licenses.
6. **Release gates (29–33):** dependency/license checks, honest MSRV policy,
   explicit architectures, packaged migration/shortcut/navigation smoke tests,
   signing/updater credentials and artifact provenance. Do not claim shippable
   or complete task 33 until these gates have real evidence.

## Validation and publication

Run from the repository root. Rust, Node/npm, Python with
`scripts/requirements-catalog.txt`, and the platform Tauri dependencies are
required. Install the Playwright Chromium browser with `npx playwright install
chromium` (CI also uses `--with-deps`).

```text
git status --short --branch
git remote -v
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
python scripts/test_catalog.py
python scripts/catalog_pipeline.py --check
python scripts/cache-catalog-icons.py --check
node scripts/generate-brand.mjs --check
python scripts/validate-recipes.py
npm test
```

Windows owns the reviewed pixel baselines. On other platforms, run `npm test --
--ignore-snapshots` for interaction/accessibility coverage. Compose validation
checks syntax only and is not evidence of real container lifecycle success.
For Rust scratch files on a space-constrained Windows machine, create
`.cache/rust-tmp` and set this shell's TEMP and TMP to its absolute path. Never
delete unrelated caches to make space.

Before publication, inspect `git diff --check` and the full diff, update this
handoff and the ledger with actual evidence, then commit and push `main` without
force. Obtain the workflow run for the pushed commit using `gh run list` and
`gh run view <id> --json status,conclusion,jobs,url`. Confirm all three native
build jobs as well as quality. If a run fails, use `gh run view <id> --log-failed`
and inspect only relevant excerpts; do not weaken gates or regenerate visual
baselines to hide an unrelated failure. Report any still-running remote check
as pending rather than claiming success.
