# Historical agent checkpoints

Archived September 7, 2026. These checkpoints describe earlier states.
Use [agent-handoff.md](agent-handoff.md) and [upgrade-status.md](upgrade-status.md)
for current instructions, status and next work.

---

# Local Store: agent handoff

Updated September 7, 2026. This file is intended for any coding agent or human
maintainer. Read it alongside `docs/upgrade-status.md`, the versioned ledger of
all 33 original upgrade tasks. The ignored `.hermes/` workspace is not needed
to continue development.

## Working agreement

- Product: **Local Store**. GitHub: https://github.com/madhavsonkusare-a11y/local-store.
- The current icon and CLI batches are for **local review only**: do not commit or
  push it without a new instruction from the owner. Earlier development batches
  used commits and pushes directly to `main`, without a PR.
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

## Latest checkpoint: recipe requirements and image-platform evidence (11 complete)

**Read this section and the immediately following readiness checkpoint first.**
Task 11 is complete: schema 3 adds Linux engine, Compose v2, local-storage and
container-platform requirements to all three previews. Public Docker Hub tag
responses are cached in `catalog/recipe-platforms/`; sources, review date and
index digests live in each manifest. The offline checker and four regression
tests run in quality CI. `--refresh` explicitly fetches bounded metadata and
fails on drift; it does not approve changes or pull image layers. See
`docs/recipe-requirements.md` for sources and maintenance instructions.

This is image compatibility evidence, not installer/host support or recipe
graduation. Compose still uses the existing version tags; the recorded digests
are audit evidence. Unknown-platform attestation records are excluded. n8n's
registry returned a pull-rate limit; its public tag listing supplied the metadata.

Review also found the earlier port check accepted substring matches, so port
52300 could pass for 5230. Both Rust and Python now parse the actual loopback
address and port. Port overrides validate the original and change the authority
port once, preserving paths/queries. Compose parsing now has a 30-second bound.

Validation: **144 Rust tests**, strict Clippy, formatting, **four Python platform
tests**, the offline metadata checker and all three Docker Compose configuration
checks pass. The three install-port UI tests also pass against schema 3; no
snapshots changed. Earlier in this continuation, all 44 UI cases and 21 Node
tests were verified as detailed below. No release rebuild, commits or pushes.

Ledger: **23 complete, 8 partial, one not implemented, one final gate not run**;
ten tasks remain. Docker daemon access still times out. Recipe lifecycle proof
remains blocked; do not repeat completed schema/UX work. Remaining work includes
cross-process concurrency, clean installer and non-Windows native evidence,
explicit release architectures/provenance and signing/updater configuration.

## Previous checkpoint: live readiness and connection removal (25 complete)

The owner's connection checks, guarded retries, port options, recipe schema,
data confinement, MSRV/licence gates and security document were read and
preserved. **Task 25 is complete; the ledger is 22 complete, 9 partial, one
not implemented and one final gate not run. Eleven tasks still need work.**

- `src/js/readiness.js` schedules checks 15 seconds after each round, with
  at most four in flight. It checks displayed eligible apps only, pauses on
  navigation/hidden documents, skips busy or stopped managed apps, and ignores
  replies from previous app/address/operation generations. It reuses the
  existing `app_readiness` IPC and does not poll Docker container status.
- Known answers update status text in place; unknown/errors fall back to the
  original status. Unchanged answers do not repaint or announce again. Leaving
  the page disposes timers. No dependency or animation was added.
- Removing an app now restores keyboard focus to the nearest remaining row
  action, or the empty-state action. A connected-app regression covers failed
  removal, retry, busy-dialog dismissal prevention and non-destructive IPC.
- `docs/security-and-privacy.md` now distinguishes the offline launcher from
  embedded third-party pages and acknowledges WebView profiles, Docker storage
  and OS registrations. The previous claims that nothing was stored elsewhere
  or sent off-machine were too broad. Native privilege review remains partial.

Validation: all **21 Node tests** pass (seven new monitor tests). The full UI
run passed 43 of 44; the remaining test assumed Open was the last IPC call.
It now asserts exactly one Open call with the correct ID, independent of
background probes, and its targeted rerun passes. All 44 cases are verified;
no visual baselines changed. Windows sandbox teardown again needed cleanup of
the verified preview process; running Playwright with normal process permissions
then completed cleanly. No commits, pushes, new dependencies or container work.

Docker was checked outside the sandbox again: `docker version --format json`
still timed out at 10 seconds. The service recovery instructions below remain
relevant; do not repeatedly restart Docker or claim recipe lifecycle evidence.
Next unblocked implementation is recipe requirements/platform metadata (11),
followed by remaining concurrency/release gates. Usage has reset since the
historical checkpoints; query the current five-hour allowance before sizing work.

## Previous checkpoint: native activation (September 7)

**Read this section first; older checkpoint sections below are historical.**
The owner's cancellation, readiness and browser-error changes were preserved.
Official `tauri-plugin-single-instance` 2.4.4 (with `deep-link`) and
`tauri-plugin-deep-link` 2.4.10 are now pinned and integrated. One builder and
command handler serve every native launch. The single-instance plugin must
remain first. It forwards URI events before its callback, so the callback must
not dispatch ordinary protocol arguments again.

- `src/activation.rs` validates arguments and resolves only registered apps.
  Stable IDs win; names remain supported by native CLI and legacy links.
  Raw URL validation rejects dot segments, encoded separators and extra args.
- `src/native.rs` dispatches blocking runtime work off the event loop, coalesces
  concurrent opens by app ID and reuses existing windows. Windows forwarding
  repairs the upstream plugin's unescaped pipe delimiter in display names.
- Native errors queue until the launcher drains `take_activation_errors`.
  The new command has the build declaration, launcher grant and runtime guard.
  The event contains only a wakeup; queue draining prevents duplicate messages.
- Both URL schemes are configured in `tauri.conf.json`. Official runtime
  registration replaces handwritten Windows/Linux registration. macOS keeps
  its bundle metadata. Ordinary CLI commands remain independent processes.
  A secondary native process exiting 0 acknowledges delivery, not runtime success.

**Windows evidence:** `cargo tauri build --bundles nsis --no-sign --ci` produced
`target/release/bundle/nsis/Local Store_0.5.0-1_x64-setup.exe`. The first attempt
was blocked downloading NSIS; the approved network retry succeeded.
`pwsh -NoProfile -File scripts/smoke-native-windows.ps1` passed against the
release executable: launcher startup, second-process delivery, repeated window
reuse, OS activation of both registered protocols, and independent CLI exit.
The remote fixture actually invoked four commands (`list_apps`, cancellation,
readiness, activation-error drain); all four returned **not allowed by ACL**.
It used a private registry and WebView profile, restored both protocol handler
backups, and reported no cleanup errors. Local report:
`.cache/native-smoke-fae2bbc6bb0a4d8b80a4dfddc6b8d67d/report.json`.

This proves the release image on this Windows desktop, **not a clean installer
installation**, macOS/Linux activation, all navigation boundaries, managed-app
Docker startup, or packaged migration. Tasks 17/27/31 remain partial. The smoke
harness refuses to run while Local Store is already open and temporarily changes
the current user's protocol registrations; run on an interactive Windows desktop.

Validation: 132 debug Rust tests, 12 Node tests, strict all-target/all-feature
Clippy, formatting and approved brand geometry pass. See the final validation
note below for the UI/release checks. No commits or pushes; the mixed working
tree and 752 catalog icons remain intact. Ledger: **19 complete, 12 partial,
one not implemented, one final gate not run** (14 still need work).

Next bounded work: clean Windows installer/shortcut/migration proof, then
macOS/Linux activation and navigation isolation. Preserve the existing native
implementation rather than re-adding plugins. Cross-process registry locks,
real recipe lifecycle proof, remaining setup/removal UX and release gates follow.
The last five-hour usage check was 94%; refresh usage before choosing a new batch.

Final UI validation: `npm test` exited 0 with **12 Node and 32 Playwright tests**,
including unchanged pixel baselines. Windows sandbox teardown hung after all
assertions; terminating only the verified test preview process let Playwright
finish normally (3.7 minutes). Do not treat that teardown delay as a UI failure
or add test retries. Installer SHA-256:
`7fe840ac5134d63b4087a1dd4ff87b1f0e9c38ad95950ddec94bcf0b6a0b1933`.

## Historical checkpoint

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
- Task 13 is **complete**: `src/runtime/process.rs` gives every `CommandSpec` an
  explicit deadline, drains output concurrently into bounded buffers, terminates
  the process tree it owns, exposes a `CancelToken` at the runner boundary and
  redacts credentials from diagnostics. Do not rebuild these.
- A data-loss bug in install rollback was fixed in the same batch; see below.
- Task 20 is **complete**: the CLI contract is now verified against the release
  image, which uncovered and fixed a bug that made the shipped Windows CLI
  print nothing at all from `cmd.exe`. See the packaged-CLI section below.
- Install now has port preflight, atomic Compose writes, per-app operation
  locks and a cancellable health wait. Task 14 remains partial: progress events
  and cross-process locking are still open.
- The local CLI batch completes `doctor --json` and filtered/paged `catalog`
  queries, reusing the existing Doctor report and catalog index. Parsing for
  these commands uses pinned `pico-args` 0.5.0 with no transitive dependencies.
  See `docs/cli.md` for output, filters, paging and exit codes. Implementation is
  in `src/cli_queries.rs`, wired through `src/cli.rs`; do not rebuild these features.
- Validation: the prior CLI batch passed 81 Rust tests, then 83 after strict
  connection/lifecycle argument preflight and removal of the release Tauri
  devtools feature. The bounded-process, redaction, rollback, packaged-CLI and
  install-transaction work, then the typed-error, operation-UI, install-stage
  cancellation-boundary, Cancel-control, readiness and capability batches, bring
  this to **129 Rust tests passing
  locally on Windows** in the debug profile. The twelve CLI tests also pass
  under `cargo test --release`, and the Node adapter/state tests plus the
  31-test Playwright suite are green, with `cargo fmt --all -- --check` and
  `cargo clippy --all-targets --all-features --locked -- -D warnings` clean.
  Release installers have not been rebuilt. Windows/macOS CLI checks were
  added to the existing build workflow and now also run `--test process_runner`;
  Linux quality already runs the whole suite. Remote checks for these local
  changes have not run because publication is paused.
- **Fixed flake:** the Playwright per-test timeout was 20 seconds, which the
  "keyboard shortcut and accessibility" test exceeded whenever the machine was
  busy — it failed both runs launched alongside the Rust and Python gates
  (28.9s) and passed both idle runs (4.5s). `playwright.config.js` now allows
  60 seconds. Verified by rerunning the whole suite under the same concurrent
  load that used to fail it: 21/21 passed, with two tests taking 17.5s and
  21.3s, i.e. over the old limit. The timeout still exists to catch a hang; the
  suite finishes in about 50 seconds idle. Do not "fix" a future timeout here
  by adding retries, which would hide a real regression.
- The 244 added icons remain intact and uncommitted (752 total); the remaining
  920 monograms are explicitly deferred by the owner. No more icon sourcing is
  part of the next runtime batch.
- The ledger is now **18 complete, 13 partial, one not implemented, one final
  gate not run**. Fifteen tasks still need work; this is not an effort estimate.
- Discovery contains **1,672 projects** and **752 offline catalog SVG/PNG icons** from four
  pinned sources. Only **three install previews** exist: Memos, n8n, Uptime Kuma.
  Their real install/restart/upgrade/preserve/delete lifecycle is not yet proven.
- Validation ordering matters on one machine: run `npm test` on its own where
  possible. The Playwright timeout now absorbs concurrent load, but the pixel
  baselines are still the slowest checks in the suite.
- Approved logo geometry at 512 px: tile `(193,95)`, size `224`, radius `56`;
  L stroke `56`, equal gaps `42`, shared facing-corner center `(249,263)`, radii
  `56` and `98`. Top edges align at `95`. `scripts/generate-brand.mjs --check`
  verifies generated exports. Do not redesign the approved geometry.

## Completed: bounded process execution and redaction (task 13, complete)

Implemented in `src/runtime/process.rs`, tested there and in
`tests/process_runner.rs`. Do not rebuild any of this.

- **No process-management dependency was added.** `command-group` and
  `process-wrap` were evaluated and rejected: both would still have left the
  deadline and bounded-capture logic to write by hand, and both pull in a
  further dependency tree (`nix` on Unix) for the one primitive needed.
  Instead the job-object and process-group calls use `windows-sys`, already a
  direct dependency — three feature flags added, no new crate — and `libc`,
  already present transitively through Tauri. **The declared MSRV of 1.77.2 was
  left unchanged**: `libc` 0.2 requires far less, and `Command::process_group`
  has been stable since 1.64. *Later correction: that declaration was already
  false for unrelated reasons — core Tauri dependencies needed 1.88. See the
  MSRV checkpoint near the end of this file.*
- Deadlines are per operation, not global: `DIAGNOSTIC_TIMEOUT` 30s for
  `version`/`ps`/`config`/`logs`, `LIFECYCLE_TIMEOUT` 180s for `stop`/`down`,
  `PROVISION_TIMEOUT` 900s for `up -d`, which may download images.
- Each stream is drained by its own thread into a 256 KiB buffer and keeps
  reading past the cap so the child never blocks on a full pipe.
  `ProcessOutput.truncated` signals the loss and `concise_error` surfaces it.
- On timeout or cancellation the tree is terminated — `TerminateJobObject` on
  Windows, `SIGTERM` then `SIGKILL` to the process group on Unix — and reaped
  within a bound. The Docker daemon and its containers are never in that tree,
  because the CLI reaches the daemon over a socket or named pipe.
- Readers are never joined without a bound, and both share one deadline: a
  grandchild holding an inherited pipe cannot stall a finished command, and
  cannot spend the grace twice. Truncation that lands mid-character drops the
  partial UTF-8 sequence instead of emitting a replacement character.
- `CancelToken` is at the runner boundary only. **Operation-level and UI
  cancellation are still not wired**; they belong to tasks 14 and 24, along
  with the health-wait cancellation path.
- Redaction is applied on **diagnostic surfaces**, not at the runner boundary.
  An earlier draft of this file said to redact every caller's output; that was
  wrong and has been corrected. Blanket redaction would rewrite `logs`, which
  the user explicitly asked to see and which `docker compose logs` would show
  verbatim anyway, hiding real debugging detail while protecting nothing. The
  redactor therefore runs in `concise_error` and in the timeout/cancellation
  message, covering Doctor details, lifecycle failures and any future operation
  event. `logs_with` deliberately returns raw output.

Verified: 8 integration tests over real `sh`/`cmd`/`powershell` children cover
success, nonzero exit, missing executable, timeout with proof the descendant
died, cancellation, simultaneous oversized streams, mid-character UTF-8
truncation and an inherited pipe; 3 unit tests cover decoding and bounds. None
require Docker or touch user app data. All pass on Windows. The Linux and macOS
`cfg(unix)` path was type-checked against `x86_64-unknown-linux-gnu`,
`aarch64-apple-darwin` and `x86_64-apple-darwin` but **has not been executed on
either platform** — the build workflow now runs `--test process_runner` on
Windows and macOS, and Linux runs it inside the full `quality` suite. Read the
workflow for the first pushed commit to confirm.

## Fixed alongside it: install rollback destroyed preserved data

`src/commands.rs` rolled a failed registry commit back with
`runtime::uninstall(&app, true)` — deleting data. The reachable path was:
uninstall keeping data (the directory survives, the registry entry does not),
reinstall (`install_recipe_with` deliberately reuses that directory), then a
registry write failure. The user's preserved data was deleted, despite
`install_recipe_with` tracking `created_project` precisely to avoid that.

The rule now lives in `runtime::rollback_install`, which takes whether the
project directory predated the install and never deletes data that did.
`rollback_after_a_failed_commit_keeps_data_that_predates_the_install` fails if
the rule is reverted. This closes only the data-loss hole; the rest of task 14
is unchanged.

## Completed: packaged CLI verification (task 20 complete, 31 partial)

`tests/cli_packaged.rs` runs the CLI contract against the **release** image.
This matters because on Windows the shipped binary is a different program from
the one every debug test uses: release sets `windows_subsystem = "windows"`, so
it is a GUI-subsystem image. The first test asserts that subsystem, so the suite
fails loudly if the packaging assumption ever changes.

**It found a real bug in the shipped product.** In the released Windows binary,
every CLI command run from `cmd.exe` printed *nothing at all* — no stdout, no
stderr — while still returning the correct exit code. `local-store doctor
--json > out.txt` produced an empty file. Only `--version` worked, because it
prints in `main` before the rest of the CLI starts.

The cause was `ensure_console()` in `src/cli.rs`, called at the top of
`run_cli()`. It ran `AttachConsole(ATTACH_PARENT_PROCESS)` whenever the process
had no console window of its own, which for a GUI-subsystem image is always.
Attaching **replaces the process's standard handles**, so output went to the
console instead of the pipe or file the shell had supplied, and was lost. It now
attaches only when no standard output handle was inherited, which leaves the
interactive case working and stops it stealing a redirect.
`the_windows_command_shell_receives_stdout_and_the_exit_code` fails if that is
reverted; this was verified by reintroducing the bug.

Two related changes: `emit` in `src/cli.rs` writes command output without
panicking when the reader has gone away, so `| head` and a shell that does not
wait end quietly instead of printing a Rust panic; and `main` flushes stdout and
stderr before `process::exit`, which otherwise skips buffered writers. The flush
alone was tried first and did **not** fix the bug — it is kept only because it
guards a trailing partial line.

**Windows PowerShell still cannot run this CLI directly** and no code change can
fix it: PowerShell starts a GUI-subsystem process without waiting, so it returns
before output exists and sets no `$LASTEXITCODE`. `cmd /c` and
`Start-Process -Wait` both work and are documented in `docs/cli.md` with a table
of which invocation gives output and an exit code. If that is unacceptable, the
real fix is a second, console-subsystem executable for CLI use — that is a
packaging and product-naming decision for the owner, not something to add
unilaterally, and it would touch the installer, CI artifacts and `CLI_NAME`.

Task 31 is now **partial**, not complete: this covers the release *image*, not a
*clean installed* app. Installer, native shortcut activation, navigation and the
packaged migration harness are all still untested.

## Completed: install transaction hardening (task 14 still partial)

Four boundaries closed in `src/runtime/mod.rs`, each with a test:

- **Per-app operation locks.** `lock_operation` hands out one slot per app id
  and `OperationLock` returns it on drop, including on a panic. A second
  concurrent `start`/`stop`/`uninstall`/`install` on the same app is refused
  with a clear message rather than queued: a queued operation is
  indistinguishable from a hung one. The lock lives on the public wrappers
  (`start`, `stop`, `uninstall`, `install_recipe`); the `*_with` variants stay
  lock-free so tests drive them directly. `status` and `logs` are read-only and
  deliberately unlocked.
- **Port preflight.** `PortProbe` checks `recipe.port` by *binding*, not
  connecting — a listener that accepts nothing still owns the port. It runs
  before anything is written, so a busy port gives "Port N is already in use"
  instead of an opaque Compose failure over a half-created project directory.
- **Atomic Compose writes.** `storage::write_file_atomically` (the registry's
  existing temp-file-and-rename path, now public) replaces `fs::write`, so
  Docker can never read a torn Compose file. A test asserts no `.tmp` file is
  left behind.
- **Cancellable health wait.** `wait_for_health_with` takes a `CancelToken` and
  checks it between polls, so a cancelled install stops instead of running the
  60-second timeout out, and still rolls back.

`InstallContext` now carries the runner, health probe, port probe and cancel
token, rather than growing the argument list further.

**Known limitation, deliberately not hidden:** the operation lock is
*in-process*. It serializes the launcher's own concurrent commands, which is the
common case, but it does not stop the CLI and the launcher — two processes —
from acting on one app at once. A cross-process guard needs a lock file with
stale-owner detection; that is the remaining concurrency work under task 14.

Task 14 stays **partial**: progress events (tasks 24/25) and cross-process
locking remain. Task 19 was not started — see below.

## Previous checkpoint: typed command and event contract (task 19)

Validation on Windows (September 6): full `cargo test --locked` passed 115
tests, followed by a passing added lifecycle/status propagation test (116
tests total). `npm test` passed 4 adapter tests and all 22 browser tests,
including unchanged visual baselines. Strict all-target/all-feature Clippy,
formatting and `git diff --check` passed. Native release/remote CI were not run.
Five-hour account usage was 46% consumed at the final check.

Task 19's core contract is implemented locally. `src/error.rs` defines shared
`AppError { code, message }`; runtime lifecycle/install/readiness errors and all
Tauri commands, including native open and shortcuts in main.rs, now preserve
named error codes. Storage duplicates/not-found errors have distinct variants.
CLI handlers accept the typed errors while retaining existing 0/1/2 semantics.
Per-app status failures retain their diagnostic in optional `status_error`.

`src/operations.rs` emits launcher-targeted start and terminal events for
install/start/stop/uninstall/open, with correlated operation IDs and typed
failures. Notification failure cannot turn a successful mutation into a failed
command. `listenOperations` supplies the frontend subscription API; the visible
progress UI is intentionally the next task, not implemented by this transport.
See `docs/command-contract.md` for the wire format, error codes and limitations.

No new dependencies, real Docker installs, release artifacts, commits or pushes
were introduced. Existing uncommitted owner changes remain in place. The
registry transaction/cross-process lock gaps and CLI browser-opening failure
path are still outstanding; do not infer those are fixed by typed errors.

**Next implementation: tasks 24/25.** Connect live per-app operation state to the
existing UI, retain IPC completion as authoritative fallback, then add install
stages and operation-wide cancellation. Use the installed design skills for
visible UI/motion changes. Keep native integration and real recipe proof in
separate bounded batches below.

## Previous checkpoint: operation UI (tasks 24/25, partial)

The completed local UI slice uses `src/js/operations.js` for per-app pending
state and diagnostic retention. `app.js` subscribes to operation events before
initial rendering and always settles requests from IPC, including when events
are unavailable or late. Terminal events cannot unlock an unfinished request.
Start/stop remain busy through status refresh. Conflicting row controls use
`aria-disabled` with a click guard so keyboard focus stays in place; Logs remain
available. Row refresh restores focus to the corresponding action (or Open
when Start becomes unavailable). Navigation/search preserve pending state and
errors by stable app ID, not array position.

Busy rows use the existing dark palette with a restrained orange border and
status label. Install review shows a readable pending notice; it never invents
a percentage. Typed errors provide relevant recovery guidance without automatic
retry. Failed app-status diagnostics now appear in the row. Uninstall locks the
data-deletion checkbox while its captured choice is executing. No new animation
library, native runtime changes, commits or pushes were added in this UI slice.

Validation: six Node adapter/state tests and all 25 Playwright tests passed,
including unchanged visual baselines, keyboard/reduced-motion checks, delayed
IPC, duplicate-action guards, late events and failure persistence. Busy row and
install screenshots were inspected; the install notice was moved above recipe
details after visual review. Rust is unchanged from the prior validated batch.

**Next bounded work:** install-stage events and cancellation through the entire
transaction (tasks 14/24), then readiness/health state and complete uninstall
coverage (25). Current progress reports the operation boundary only. It does
not expose a Cancel operation or promise that Docker containers are ready just
because their process is running. Tasks 24/25 remain partial. Also verify native
event delivery in the native integration gate; browser tests use the adapter.
The latest usage check was 64% of the five-hour window consumed.

## Previous checkpoint: install stages (task 24, partial)

Validation on Windows: 117 Rust tests, 7 Node adapter/state tests and all 25
Playwright tests passed, including unchanged visual baselines. Strict Clippy,
formatting and diff whitespace checks passed. No native release or remote CI
run was performed. Five-hour usage was 90% consumed at the final check; stop at
this completed checkpoint rather than starting cancellation with that reserve.

The latest batch adds runtime-driven install progress, using the existing
launcher-only operation channel. `InstallContext.progress` is an optional
callback, so CLI callers and test adapters retain their existing behavior.
`install_recipe_with_progress` reports system checks, file preparation, Compose
validation, container startup, health wait and rollback entry. The command
reports registry save and commit-failure rollback. No durations or percentages
are invented. `track_install` correlates the worker-thread events and emits the
terminal command result after work returns. See `docs/command-contract.md`.

The frontend accepts `progress` envelopes, validates known stage labels and
updates the existing install notice. An unrelated operation ID or a late
progress event after the terminal event cannot overwrite its display. IPC
still owns completion and keeps controls blocked until the operation returns.

**Next: expose cancellation (24/25).** The backend prerequisites this section
asked for are now done; see the cancellation checkpoint below.

No commits, pushes, new dependencies or real Docker installs in this batch.

## Follow-up checkpoint: rollback failure reporting

Validation: all 118 Rust tests passed, including the new cleanup-failure
regression. Strict all-target/all-feature Clippy passed. Frontend files were
unchanged in this batch; the prior 25 browser tests remain the last UI run.
No native release or remote CI run was performed.

The final small batch before the usage reset adds `rollback_failed`, retaining
both the original setup error and cleanup failure in the readable diagnostic.
If Compose down fails or cannot run, install rollback preserves the Compose
file and data for recovery instead of deleting them underneath potentially
running containers. Filesystem cleanup failures also propagate; restoration
of a pre-existing Compose file uses the existing atomic writer. Registry-commit
rollback now reports failure rather than silently discarding it.

This does not make rollback infallible or add cancellation. Next work still
needs transaction-wide locking/token propagation, a commit cutoff and tests
before exposing Cancel. CLI install commit handling and cross-process races
also remain. No publication or real Docker operations in this batch.

## Previous checkpoint: cancellation boundaries (14/24, backend only)

Every prerequisite the previous checkpoint listed before a Cancel control is
now implemented and tested. **No Cancel command or button was added** — that is
deliberate, and it is the next batch.

- **One token spans the transaction.** `begin_install` takes the token from the
  app's operation lock and threads it through preflight, Compose `config`,
  Compose `up` and the health wait. `checked_run_cancellable` and
  `start_with_cancel` carry it; the non-cancellable `checked_run`/`start_with`
  remain for lifecycle commands and rollback.
- **Checkpoints, not interruption.** Cancellation is honoured before the Docker
  probes, before the first file is written, before Compose validation, before
  container start, and between health polls. A Compose command already running
  is never cut off mid-flight; the runner's own deadline covers a hung child.
- **The commit is the cutoff.** `PendingInstall::commit` ignores the token by
  design: past that point containers are running, and honouring a late cancel
  would leave them up with no registry entry.
- **The lock spans the commit.** `PendingInstall` holds the operation lock until
  the registry write lands or rolls back, closing the window in which a
  successor operation could previously start. This also fixed the CLI, which
  committed outside the lock; `install_recipe` now commits internally and
  `src/cli.rs` no longer inserts separately.
- **Rollback runs on a fresh token,** so a cancelled install is still cleaned
  up, and pre-existing data is preserved.
- **A cancel names its target.** `cancel_operation(app_id, operation_id)` acts
  only when the running operation's id matches, so a late request cannot stop
  the operation that replaced it. Ids come from `OperationLock::id()`.

Two existing tests had used a pre-cancelled token as a shortcut to force the
unhealthy path. That conflated "unhealthy" with "cancelled" and broke once
cancellation was honoured earlier. Both now cancel *at* the health wait through
the progress callback, so they still assert the full stage sequence and the
`rollback_failed` reporting they were written for.

Validation on Windows: **122 Rust tests**, the Node adapter/state tests and all
**25 Playwright tests** passed, with unchanged visual baselines. Strict Clippy,
formatting, whitespace and all offline catalog/icon/brand/recipe gates passed.
No release build, no remote CI run, no Docker, no commits, no new dependencies.

**Next: see the cancellation control checkpoint below.**

## Previous checkpoint: the Cancel control (24/25)

The control the previous checkpoint prepared for is now shipped end to end.

- `cancel_app_setup { id, operationId } -> bool` is the only new IPC command. It
  requires the launcher window and returns whether a matching operation was
  still running; `false` is a normal outcome, not an error.
- The operation lock is now taken in `commands::install_app` **before** any
  worker thread is spawned, so a busy app is reported at once and the runtime
  operation id travels on the very first event. `begin_install` takes the lock
  as a parameter rather than acquiring it.
- `OperationEvent` gained `cancel_id`, the runtime operation a cancel must name.
  It is omitted entirely for the short lifecycle commands, which finish faster
  than a cancel could reach them — a consumer is never offered a control the
  backend would not honour.
- The launcher shows **Stop setup** inside the install progress notice while
  `cancel_id` is known and the stage is before `saving_app`. It disappears at
  the commit cutoff and on any terminal event, and a click withdraws the offer
  immediately while IPC still settles the install.
- The label is deliberately "Stop setup", not "Cancel": the install dialog
  already has a Cancel button that means *close this dialog*.

**Focus:** when the control disappears while focused, focus moves to the live
progress region, which has `tabindex="-1"` for the purpose. The first attempt
used the confirm button and failed its test — that button is disabled during an
install and cannot hold focus. The dialog's close controls were rejected as a
target because a keypress there would dismiss the dialog mid-install.

Validation on Windows: **124 Rust tests**, 7 Node adapter/state tests and all
**27 Playwright tests** passed, with **visual baselines unchanged** — the new
control is hidden unless an install is in flight. Strict Clippy, formatting,
whitespace and every offline catalog/icon/brand/recipe gate passed. No release
build, no remote CI, no Docker, no commits, no new dependencies.

**Next: see the readiness checkpoint below.**

## Previous checkpoint: per-app readiness (15/25)

My Apps now distinguishes "the containers are up" from "the app is answering".

- `app_readiness { id } -> "ready" | "unreachable" | "unknown"` is a separate
  command from `list_apps` **on purpose**: probing inside the listing would make
  the whole list as slow as the least reachable app. The launcher asks per app
  once the rows are on screen, in parallel, and fills them in as answers arrive.
- **`unknown` is returned rather than guessed.** `HttpHealthProbe` speaks plain
  HTTP only, so an `https://` connected app is reported unknown instead of being
  wrongly called unreachable on the strength of a check that never ran. This is
  the detail most likely to be "simplified" by a later change — do not.
- The launcher treats `unknown`, an IPC error and a missing answer identically:
  the row keeps its container status. Only `ready` and `unreachable` change what
  a row says, and readiness never overrides the busy label of an operation in
  flight. Stopped managed apps are not probed at all.

**Visual baselines are unchanged, including `my-apps.png`.** That is not an
accident of mocking: the test fixture returns `undefined` for `app_readiness`,
which is the same "not checked" path a real build takes for an app it cannot
probe, and that path deliberately renders exactly what it rendered before. The
new states are covered by DOM assertions rather than by minting new reviewed
artwork, which is the owner's call.

Validation on Windows: **125 Rust tests**, 7 Node adapter/state tests and all
**29 Playwright tests** passed. Strict Clippy, formatting, whitespace and every
offline catalog/icon/brand/recipe gate passed. No release build, no remote CI,
no Docker, no commits, no new dependencies.

**Next: see the capability checkpoint below.**

## Previous checkpoint: capability grants and browser errors (16/27)

### A shipped-broken bug, found and fixed

`cancel_app_setup` and `app_readiness` — added in the two batches before this
one — were registered in `generate_handler!` but **never declared in `build.rs`
and never granted by the launcher capability**. Tauri generates a permission per
command from the `build.rs` list; a command with no grant is refused at runtime.
So in a real build the Stop setup button and every readiness probe would have
done nothing, while all 29 Playwright tests passed, because the frontend suites
drive a mocked `window.__TAURI__` with no access control at all.

This was confirmed against the real build output, not inferred: the generated
`acl-manifests.json` listed 14 `allow-*` permissions and neither new command.
After the fix it lists 16 and both are present.

**`tests/capabilities.rs` now makes this class of bug impossible to repeat.** It
cross-checks three lists that must agree — the `generate_handler!` registrations,
the `build.rs` declarations that generate permissions, and the launcher
capability's grants — in both directions, so a stale grant is caught as well as
a missing one. Verified by removing a grant and watching it fail. It also pins
the security boundary: the launcher capability must apply to exactly one window
named `launcher`, and the CSP must keep its `default-src`/`object-src`/
`frame-src`/`base-uri` directives and never allow inline or evaluated script.

**Any new IPC command needs all three entries.** The test will tell you which.

### Browser failures are no longer silent

`launch_browser` printed to stderr, which an app window does not have — a link
that failed to open was indistinguishable from a dead page. It now returns a
typed error, and every caller reports it: app windows emit
`local-store://browser-failure` to the launcher, which shows a toast; the CLI
prints it and exits non-zero. The compiler found a third caller during the
change — `local-store open --browser` had been discarding the result and
reporting success regardless.

Also removed a stale `TODO(Task 16)` and `#[allow(dead_code)]` claiming
`validated_external_url` was unwired. It is reached from `on_navigation`,
`build_window` and `open_project`, so every URL that can leave the app already
passes through it; the misleading `allow(dead_code)` would have hidden real dead
code later.

### Not done, deliberately

**The official single-instance and deep-link plugins (task 17) were not
adopted.** They are new dependencies whose whole purpose is cross-process
behaviour — delivering a second launch's arguments to the running instance, and
receiving an OS protocol activation. Neither can be verified from a test
harness: both need a packaged, installed app launched twice with a registered
protocol handler. Adding them here would have meant shipping unverified
plumbing against this project's own standard of proving each boundary. Task 17
is unchanged and still needs its own batch, ideally alongside the packaged
smoke tests of task 31, which are the only place that evidence can come from.

Validation on Windows: **129 Rust tests**, 7 Node adapter/state tests and all
**31 Playwright tests** passed, with unchanged visual baselines. Strict Clippy,
formatting, whitespace and every offline gate passed. No release build, no
remote CI, no Docker, no commits, no new dependencies.

**Next: deep links and single instance (17/31).** Do it together with packaged
smoke tests so the cross-process behaviour has real evidence. Until then, do not
mark task 17 complete on the strength of the plugins compiling.

## Earlier checkpoint: shared native startup routing (17/31, partial)

The updated cancellation, readiness, capability and browser-error code was
reviewed and preserved. This batch closes a prerequisite for the official
single-instance/deep-link integration: **there is now one Tauri builder and one
command handler for every native launch**. Previously `open` and protocol
activation entered a separate builder that never created the launcher or
installed its command handler. Native opening now creates the launcher and
then focuses the selected app window, using the same startup configuration.

`src/activation.rs` owns the side-effect-free argument parser and registry
lookup. Primary links resolve stable IDs only; legacy links and native CLI open
also support display names, with exact IDs taking precedence. Invalid stored
launch URLs fail before Docker starts. Malformed protocol requests (including
extra arguments, control characters, encoded separators and dot-segment paths)
fail before registry or runtime work. Native CLI names may still contain path
separators because they are names resolved from the registry, not paths.

The special `--version` early return was removed: version output now follows
the same console attachment and flushing path as the rest of the CLI.
New executable tests verify malformed activations exit 2 and unknown targets
exit 1 using a private config root, without creating registry files. They run
in the existing packaged-CLI suite, so CI exercises both new cases in release.
No frontend, capability, protocol-registration, dependency or installer changes.

**Superseded — read the next checkpoint.** At the time this was written no
official plugins, second-process delivery or OS protocol activation existed.
The batch that followed added and proved all three on Windows. The paragraph
below is kept as the historical record of that intermediate state.

**Task 17 remained partial at this point.** No official plugins, second-process
delivery or OS protocol activation were added or claimed tested. The actual GUI bootstrap
still needs installed-app smoke evidence; command-line rejection tests do not
prove it. Managed-app startup still precedes the GUI event loop as it did
before; the plugin callback must instead schedule blocking work off the UI
thread. Use this shared request parser/resolver for callbacks, wire both
startup and running-instance activation, and test duplicate delivery, existing
window focus and command access in an installed build. Preserve ordinary CLI
commands as short-lived processes, not activations sent to the first instance.

No commits or pushes were made. Updated ledger entries also now acknowledge
the owner's already-completed transaction-wide lock and cancellation work.

## Previous checkpoint: native integration proved on Windows (17/27/31)

The official `tauri-plugin-single-instance` (with the `deep-link` feature) and
`tauri-plugin-deep-link` are adopted and wired into the single builder from the
previous checkpoint. Activation requests — a second launch, an OS protocol
activation, or a native `open` — all resolve through `src/activation.rs` and
dispatch through `src/native.rs`, which queues failures for the launcher to
drain via `take_activation_errors`.

**This is backed by a real desktop run, not by compilation.**
`scripts/smoke-native-windows.ps1` drives the release image on an interactive
Windows desktop and asserts:

- a second launch reaches the **original** process — the forwarded app window
  is found under the primary process id, not a new one;
- repeated opens reuse one window, with an explicit no-duplicate assertion;
- every registered scheme in `tauri.conf.json` opens the app in the original
  process after the window is closed;
- a remote page is **denied all four** launcher commands it probes.

The harness refuses to run if Local Store is already open, backs up each
`HKCU\Software\Classes\<scheme>` registration before touching it, and restores
them in a `finally` that fails loudly and names the backup file if restoration
does not succeed. Registrations were confirmed restored on the recorded run.

### What this does **not** prove

- **No clean-machine installer test.** An unsigned NSIS installer builds; nobody
  has installed it on a machine without a development checkout and run it.
- **No macOS or Linux evidence.** The harness is Windows-only by construction.
- Task 31 covers shortcuts, navigation and the packaged migration harness; none
  of those are exercised here.

Tasks 17, 27 and 31 therefore stay **partial**, and the ledger says so.

### Preview server shutdown

The previous agent reported Playwright hanging while shutting down its preview
server, and recorded it rather than fixing it. `scripts/preview.mjs` created an
`http.Server` with no signal handling and never closed connections: a browser
holds keep-alive sockets well past the last request, and an `http.Server` stays
alive while any socket is open, so the process could outlive the run and keep
port 4173 bound. It now closes connections, lowers `keepAliveTimeout`, exits on
the usual signals and on stdin closing, and has a 2-second backstop so cleanup
cannot itself hang.

**Honest limit: the hang was not reproduced here.** `npm test` completed
normally before and after the change, so this is a defensive fix for a clearly
wrong shutdown path, not a confirmed repair of an observed failure. If it
recurs, check for an orphaned `node scripts/preview.mjs` holding port 4173 —
`reuseExistingServer` is true, so a leftover server is silently adopted.

Validation on Windows for this checkpoint: **132 Rust tests**, the Node
adapter/state tests and all **32 Playwright tests** passed, with port 4173
released cleanly afterwards. The release CLI suites (`cli_packaged`,
`cli_queries`) pass under `cargo test --release`. Strict Clippy, formatting,
whitespace and every offline gate passed. No commits or pushes.

**Next: see the connection check checkpoint below.**

## Previous checkpoint: connection address checks (24)

Connecting an app now offers a **Check address** control beside the address
field. It reports one of three things and **never blocks saving**: an app the
user has simply not started yet is a perfectly good connection to save, so
"no answer" is phrased as information rather than as a rejection. `unknown`
keeps the same meaning it has for readiness — the probe speaks plain HTTP only,
so an `https://` address is never called unreachable on the strength of a check
that did not run. A rejected URL reports against the field with
`aria-invalid`; a verdict is cleared as soon as the address is edited.

`check_address` reuses the readiness probe through the new
`runtime::address_readiness_with`, so a saved app and an unsaved address are
judged on identical terms.

**The `tests/capabilities.rs` guard did its job.** `check_address` was
registered in `generate_handler!` and the test failed immediately with
"registered but absent from build.rs, so no permission is generated" — the same
defect that shipped twice before it existed, caught this time in seconds.

### One reviewed baseline changed — please look at it

`tests/ui/app.spec.js-snapshots/connect-dialog.png` was regenerated. A visible
control is inherent to this feature, so the reviewed dialog genuinely changed;
this is not a baseline refreshed to hide a failure. **It is the only snapshot
touched**, and it needs your eye, since the dialog is approved design.

Worth knowing how it was placed: the control first went into `modal-actions`
next to the primary button, which crowded the action row and — more concerning —
slipped **under** the 1% pixel tolerance, so the baseline silently stopped
matching what ships. Moving it beside the address hint, where it belongs in the
visual hierarchy, pushed the diff to 3% and failed loudly, which is the correct
behaviour. If a future change to a reviewed surface passes suspiciously easily,
check the diff ratio rather than trusting the pass.

### Preview server

Left as described in the previous checkpoint: hardened shutdown, hang not
reproduced here. `npm test` again completed normally and released port 4173.

Validation on Windows: **133 Rust tests**, the Node adapter/state tests and all
**34 Playwright tests** passed. Release CLI suites pass under
`cargo test --release`. Strict Clippy, formatting, whitespace and every offline
gate passed. No commits, pushes or new dependencies.

**Next: see the retry and removal checkpoint below.**

## Previous checkpoint: guarded retry and removal coverage (25)

### Setup options are blocked on the recipe manifest — do not start them

The previous checkpoint said to check the manifest before building setup-option
UI. Checked, and the answer is no. Each recipe's compose is a **pinned literal
YAML string**, and the port number appears twice in it: once in
`ports: - "127.0.0.1:HOST:CONTAINER"` and once as an internal value such as
`MEMOS_PORT`. The manifest carries a single `port` field that cannot say which
occurrence is the host side. A substitution to honour a user-chosen port would
rewrite the container port too and produce a broken container, and it would do
so silently. All three recipes have this shape.

**Offering a port choice therefore requires task 11's schema work first** —
the manifest must distinguish host from container port. The ledger entry for 11
now records this dependency. Do not attempt setup options by string-replacing
the pinned compose.

### A blind retry over a failed cleanup is now refused

When an install fails and its cleanup *also* fails, the error is
`rollback_failed` and containers or files may still be on disk. The dialog was
re-enabling the same Install button, inviting a retry over that wreckage. It now
disables it and reads "Review needed before retrying"; reopening the review is
the deliberate way back and clears the block. Every other failure rolled back
cleanly, so those still offer a retry immediately — `retryIsUnsafe` draws that
line in one place and is unit-tested against each code.

Recovery guidance was added for `rollback_failed`, `cancelled` and
`already_exists`; an unknown code still shows its message rather than swallowing
it.

### Removal coverage

The destructive path had no test at all. Writing one found a guard I had not
expected: deleting app data takes **three** deliberate steps — the checkbox, the
changed button wording, and a native confirmation. Playwright auto-dismisses
dialogs, so the first version of the test silently did nothing and looked like a
product bug. The test now asserts both directions: dismissing the confirmation
deletes nothing and leaves the dialog open, and accepting it sends
`deleteData: true`. A second test asserts the destructive choice is not
inherited by the next uninstall.

Validation on Windows: **133 Rust tests**, 11 Node adapter/state tests and all
**38 Playwright tests** passed. No snapshot changed in this batch; the only
baseline touched in this session remains `connect-dialog.png` from the previous
checkpoint, which still wants your review. Strict Clippy, formatting, whitespace
and every offline gate passed. No commits, pushes or new dependencies.

**Next: see the manifest schema checkpoint below.**

## Previous checkpoint: recipe manifest schema v2 (11)

The manifest now distinguishes the two ports that were previously one field:

- `host_port` — published on the user's machine. This is the one a user could
  be offered a choice about, and the one `launch_url`, `health_url` and the
  install port preflight use.
- `container_port` — what the image listens on inside the container. Fixed by
  the image; changing it breaks the mapping rather than relocating the app.

`schema_version` is `2`; a manifest still declaring `1` is refused. All three
recipes were migrated, and today all three happen to publish host and container
on the same number — which is exactly why the old single field looked adequate
and silently was not.

### The manifest is now checked against the Compose it ships

`Recipe::validate()` requires the Compose mapping to be
`"127.0.0.1:{host_port}:{container_port}"` and requires the launch and health
addresses to use the **published** port. This runs before every install.

`scripts/validate-recipes.py` was rewritten to check the same agreement at build
time using **Docker's own parser** rather than a regex: it runs
`docker compose config --format json` and compares the resolved `published`,
`target` and `host_ip` against the manifest, along with the image pin and the
schema version. It also enforces exactly one published mapping, bound to
loopback. Compose syntax is still validated, as a side effect of parsing it.

Both directions were proved by breaking them on purpose: a manifest claiming a
port the Compose file does not publish is rejected, and a genuine remap that
leaves `launch_url` pointing at the old port is rejected. A *consistent* remap
to a different host port is accepted — that positive case is the one that
demonstrates setup options are now actually unblocked, rather than just
differently spelled.

### Why this mattered

A mismatch here fails nowhere until a user installs the app and is sent to a
port nothing is listening on. There is no test that would have caught it and no
error at build time. That is why the check uses Docker's resolution of the file
that actually ships, not a second parser that could drift from it.

Validation on Windows: **135 Rust tests**, 11 Node adapter/state tests and all
**38 Playwright tests** passed. Strict Clippy, formatting, whitespace and every
offline gate passed, including the rewritten recipe validator. No snapshots
changed. No commits, pushes or new dependencies.

**Next: see the setup options checkpoint below.**

## Previous checkpoint: install setup options (24 complete)

The install review can now publish a recipe on a port of the user's choosing.
**Task 24 is complete**; the ledger moves to 20 complete, 11 partial.

### The pinned recipe is never mutated

`Recipe::with_host_port` returns a *republished copy*. It rewrites only the host
side of the Compose mapping, carries the launch address, health address and risk
notes along, and then runs the full manifest validation on the result — so the
schema-v2 agreement check from the previous checkpoint is what guarantees the
rewrite is complete. A rewrite that left an address behind is refused before
Docker is invoked, not discovered by a user on a dead port.

The container port never moves; it belongs to the image. A test asserts the
remapped Memos compose still sets `MEMOS_PORT: "5230"` internally while
publishing on 8080, which is exactly the mistake the old single `port` field
would have made.

### The ordinary install is byte-for-byte unchanged

`hostPort` is omitted from the IPC payload entirely when the field is untouched.
The pre-existing test asserting `install_app` is called with `{recipeId:'memos'}`
still passes **unmodified** — that is the evidence that the default path did not
change, and it is worth keeping that test exactly as it is for that reason.

### Where the choice lives

On the install call, not on the recipe. Recipes are pinned reviewed constants
and stay that way; the resulting `InstalledApp` carries the chosen address
because it is built from the republished recipe. Nothing persists a port choice
against a recipe, so re-reviewing a recipe always shows its pinned default.

### Interface

The field is revealed by a "Use a different port" control rather than shown by
default, so the ordinary case — accept the pinned port — is unchanged in use.
The stated address updates live while the number is edited, and the help text
names the container port so it is clear what is and is not moving. A port
outside 1024-65535 is refused in the launcher before any command is sent, with
`aria-invalid` on the field.

### Baseline

`install-review.png` was regenerated: a visible affordance is inherent to the
feature. **That is now two reviewed baselines changed this session** —
`connect-dialog.png` and `install-review.png` — and both still want your review.
No other snapshot has been touched.

Validation on Windows: **137 Rust tests**, 11 Node adapter/state tests and all
**41 Playwright tests** passed. Strict Clippy, formatting, whitespace and every
offline gate passed, including the recipe validator. No commits, pushes or new
dependencies.

**Next: see the data boundaries checkpoint below.**

## Previous checkpoint: data-safety boundaries, and why data location is not offered (15)

The previous checkpoint said to test three boundaries before offering a data
location option. Done — and testing them showed the option cannot be added the
way it was imagined.

### Do not offer an arbitrary data location

Deleting app data is guarded by `confined_to_managed_root`, which requires the
project directory to be **exactly** `<managed root>/<app id>` and to *resolve*
directly inside the managed root. That guard is the only thing standing between
"delete this app's data" and a recursive delete of an arbitrary directory.

A user-chosen data location outside the managed root therefore leaves two
options, both bad: either deletion is refused for those apps, so a user can
never remove data through the app that created it, or the guard is weakened,
which is the protection itself. **Neither is acceptable as a bolt-on.** If this
option is wanted, it needs the guard rebuilt around proof of ownership — a
marker Local Store writes and verifies — rather than around path shape, and that
is a batch of its own with these tests as its floor.

### The boundaries are now pinned by tests

Previously untested; all three are data-loss surfaces:

- **Confinement.** `confined_to_managed_root` was extracted from `uninstall`
  into a named function and tested against a sibling app's directory, a path
  outside the root, the root itself, a `..` traversal, and a deeper nesting.
- **A link wearing the managed directory's name.** This is the case the
  resolved-path half of the guard exists for: a junction at `<root>/memos`
  pointing elsewhere passes the name check and is caught only by resolving it.
  Verified non-vacuous by deleting that half of the guard and watching the test
  fail. Junctions need no privileges on Windows; the test skips with a message
  if a machine refuses to create one.
- **Preservation and deletion.** `uninstall` keeps `data/` when deletion is not
  asked for, and removes the directory and volumes when it is.
- **Unexpected files.** A preserved directory containing a file Local Store did
  not put there is refused for reinstall, the file is left untouched, and the
  refusal happens before Docker is asked to do anything.

### Recipe proof is blocked here

`docker version` did not return within 120 seconds on this machine: the CLI is
installed but the daemon is not running. Real container lifecycle evidence for
tasks 11/12/15 needs a running daemon and will pull images, bind ports and write
data, so it should be run deliberately on a machine where that is expected.
Incidentally, that hang is the exact condition task 13's 30-second diagnostic
deadline exists for.

Validation on Windows: **141 Rust tests**, 11 Node adapter/state tests and all
**41 Playwright tests** passed. Strict Clippy, formatting, whitespace and every
offline gate passed. No snapshots changed in this batch; `connect-dialog.png`
and `install-review.png` from earlier checkpoints still want your review. No
commits, pushes or new dependencies.

**Next: see the MSRV checkpoint below.**

## Previous checkpoint: an honest minimum Rust version (2/29)

`Cargo.toml` declared `rust-version = "1.77.2"`. **The dependency graph has
needed 1.88.0 all along.** Seventeen crates require it, including `image`,
`plist` and `time`, which come from Tauri itself — so this predates the plugins
and everything done in this session. Cargo never checks `rust-version` against
the graph, and the pinned toolchain is 1.97.1, so the false claim compiled
cleanly and passed every test.

It is now **1.88.0**, and that number is both justified and tested:

- `scripts/check-msrv.py` reads the resolved graph from `cargo metadata
  --locked` and fails if any crate needs more than we declare, naming the
  offenders. Run before `cargo fmt` in the quality job. Confirmed by running it
  against the old value, which listed all seventeen crates.
- A `minimum-rust` CI job reads the declared version straight out of
  `Cargo.toml`, installs exactly that toolchain, and runs
  `cargo check --locked --all-targets` with `RUSTUP_TOOLCHAIN` set so
  `rust-toolchain.toml` cannot override it. The two checks answer different
  questions: the script says the number is not too low for our dependencies,
  the job says our own code actually builds there.
- **Verified locally:** the whole tree, all targets included, checks clean on a
  real 1.88.0 toolchain. The declaration is not an assertion.

`tests/package_metadata.rs` no longer hardcodes the number; it asserts the
manifest text agrees with what Cargo parsed, so a future bump stays honest
without editing a literal.

### Recipe proof is still blocked

Checked again this session: `docker version` and `docker info` both time out.
The CLI is installed, the daemon is not reachable. Real container evidence for
11/12/15 needs a running daemon and will pull images, bind ports and write data.

Validation on Windows: **141 Rust tests**, 11 Node adapter/state tests and all
**41 Playwright tests** passed on the pinned toolchain, plus a clean
`cargo check --all-targets` on 1.88.0. Strict Clippy, formatting, whitespace and
every offline gate passed, including the new MSRV check. No snapshots changed;
`connect-dialog.png` and `install-review.png` still want your review. No
commits, pushes or new dependencies.

**Next: see the dependency licence checkpoint below.**

## Previous checkpoint: dependency licence checking (29/32)

Local Store ships an MIT binary linking **495 Rust crates**. Nothing checked
what those crates permit. An upstream bump introducing a GPL dependency would
have changed what may legally be shipped, silently.

`scripts/check-licenses.py` now evaluates each crate's SPDX expression against
the licences this project accepts and fails the build if any crate cannot be
satisfied permissively. It runs in the quality job.

### On not adding `cargo deny`

The obvious tool was rejected deliberately. It is a new dependency **and** a
`deny.toml` policy that starts failing builds on upstream advisories — a policy
the owner should set, not an agent. This script answers only the question that
was actually open, using `cargo metadata` and no new dependency, and leaves
advisory scanning to a decision that is yours.

### What the graph actually contains

- No GPL or AGPL anywhere. LGPL appears only as one option of three on `r-efi`,
  where MIT is taken.
- Five crates are MPL-2.0 — `cssparser`, `cssparser-macros`, `dtoa-short`,
  `option-ext`, `selectors` — which is *file-level* copyleft. Shipping them
  unmodified is fine; modifying an MPL file would carry a source-availability
  obligation. The script lists them on every run so they stay visible rather
  than quietly accumulating, and `THIRD_PARTY_NOTICES.md` now names them.
- Nineteen ICU crates are Unicode-3.0. Every crate declares an SPDX licence;
  the script fails if one ever does not, since that needs a human.

### The evaluator is tested, not trusted

SPDX expressions here include `OR`, `AND`, `WITH`, parentheses and the legacy
`/` form, and getting `AND` wrong would silently bless a GPL crate.
`--self-test` runs twelve expressions, including `MIT AND GPL-3.0-only`, which
must fail because `AND` means every term applies. CI runs the self-test before
the real check, so a broken evaluator cannot pass the graph.

`THIRD_PARTY_NOTICES.md` previously said automated licence reporting was
planned work and pointed at the manifests; it now describes what is actually
checked and what is actually in the tree. A generated SBOM remains planned.

### Recipe proof is still blocked

Checked a third time: `docker info` times out. The daemon is not reachable.

Validation on Windows: **141 Rust tests**, 11 Node adapter/state tests and all
**41 Playwright tests** passed, with the MSRV and licence checks green. Strict
Clippy, formatting, whitespace and every offline gate passed. No snapshots
changed. No commits, pushes or new dependencies.

**Next: see the security document checkpoint below.**

## Previous checkpoint: security and privacy documentation (32 complete)

`docs/security-and-privacy.md` is new, linked from the README, and **task 32 is
complete**. The ledger moves to 21 complete, 10 partial.

The privacy claims were verified against the code, not assumed. The only
outbound network call the application makes is the health probe's
`TcpStream::connect_timeout` in `src/runtime/mod.rs`; there is no HTTP client
dependency, and the frontend contains no `fetch`, `XMLHttpRequest` or
`WebSocket` at all. That supports a plain statement — no telemetry, no
analytics, no account — which would have been reckless to write without
checking.

The document states the boundaries that have tests behind them (launcher-only
capability, CSP, URL validation, navigation isolation, recipe constraints, data
confinement, credential redaction, bounded processes, licence checking) and,
deliberately at equal length, **what Local Store does not protect against**:
unsigned installers, no updater, unaudited third-party apps, install previews
whose lifecycle is unproven, an address check that cannot verify HTTPS,
in-process-only operation locking, Windows-only privilege review, and no
clean-machine installer test.

That limits section should be kept honest as tasks close. If it ever reads
better than the ledger does, one of the two is wrong.

Vulnerability reporting points at GitHub security advisories on the repository
and states plainly that this is a one-person preview with no response guarantee
and no bounty. **Confirm private vulnerability reporting is enabled** on the
repository, or change that section.

Validation on Windows: **141 Rust tests** and every offline gate passed;
formatting, MSRV and licence checks clean. Documentation-only change to the
shipped product. No commits, pushes or new dependencies.

**Next: recipe proof (11/12/15) once Docker runs** — see the diagnosis and
recovery section below, which is the first thing to act on. The remaining
unblocked work is thin: recipe manifest requirements and platform metadata
(11), and live health polling plus connected-app removal edge cases (25).
Everything else needs a Docker daemon, a clean machine, macOS/Linux hardware,
or signing credentials.

## Docker on this machine: diagnosis and recovery

Recipe proof (11/12/15) has been blocked all session. Docker was not merely
stopped — **Docker Desktop 4.68.0 was crash-looping**, and the `docker` CLI hung
rather than erroring because it connected to a named pipe nothing was serving.

Two causes were found and fixed, neither needing elevation:

1. **Inference manager.** An orphaned zero-byte AF_UNIX socket file at
   `%LOCALAPPDATA%\Docker
un\dockerInference` (dated 31 Aug) that Windows
   refuses to delete — `File.Delete` fails with "The file cannot be accessed by
   the system", exactly as it failed for Docker itself. The backend log shows
   the path with an unexpanded `<HOME>` placeholder, and `<`/`>` are illegal in
   Windows paths, so the listener could never be created.
2. **Secrets Engine.** The identical failure at
   `%LOCALAPPDATA%\docker-secrets-engine\engine.sock`, which only surfaced once
   the first was cleared.

Neither file can be deleted, so each parent directory was **renamed aside**;
Docker recreates them clean. The parked copies hold only the broken sockets and
can be deleted once the machine has rebooted:

- `%LOCALAPPDATA%\Docker
un-stale-20260907-112914`
- `%LOCALAPPDATA%\docker-secrets-engine-stale-20260907-113236`

After both fixes the backend log shows **no error payloads** — the crash loop is
gone. What remains is that `com.docker.service` (StartType Manual) is Stopped,
and starting it needs an elevated token. `EnableLUA=1` and
`ConsentPromptBehaviorAdmin=5` mean elevation raises a consent dialog; the
account is in Administrators but the shell holds a filtered token, and the
service ACL grants start rights to `BA` only. An elevation request from this
shell is cancelled by Windows immediately without ever displaying a dialog, so
elevation cannot be obtained from an agent session at all.

**To recover, from an elevated PowerShell on the machine:**

```powershell
Start-Service com.docker.service
Start-Process "$env:ProgramFiles\Docker\Docker\Docker Desktop.exe"
```

A reboot is the more thorough option: it clears orphaned socket files properly.
Confirm with `docker version` returning a Server version — a *hang* rather than
an error means the engine still is not serving.

## Following batches, in order

1. **Recipe proof (11/12/15) when a Docker daemon is available.** Needs
   isolated data roots and ports; `with_host_port` and `LOCAL_STORE_CONFIG_DIR`
   make that possible. This is the largest remaining honesty gap. A user-chosen
   data location is **not** a next step — see the data boundaries checkpoint for
   why it needs the ownership guard rebuilt first.
2. **Deep links and single instance (17), with packaged smoke tests (31):**
   the official plugins and Windows release-image proof are done. Add clean
   installer and cross-platform proof. Capability grants, the CSP boundary and
   browser-opening errors are already implemented; expand native evidence.
3. **Recipe proof (11/12/15):** use isolated data roots and ports to verify the
   original three recipes through install, health, restart, upgrade, preserve
   and explicit data deletion on supported platforms. Record image versions and
   results. Then adapt upstream recipes in small batches toward 10 and 25
   verified installs. Refresh discovery independently; keep provenance/licenses.
4. **Release gates (29–33):** dependency/license checks, honest MSRV policy,
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
python scripts/check-msrv.py
python scripts/check-licenses.py --self-test
python scripts/check-licenses.py
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo test --locked --release --test cli_packaged
python scripts/test_catalog.py
python scripts/test_recipe_platforms.py
python scripts/check-recipe-platforms.py
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
