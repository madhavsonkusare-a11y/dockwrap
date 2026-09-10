# Hermes upgrade status

Updated September 10, 2026 (PrivateBin, Node-RED and flatnotes offered). This is the current ledger for the original 33-task
plan; earlier phase documents are historical checkpoints. A complete core
deliverable does not replace the separately counted native/release gates.

**27 core deliverables complete, 4 partial, one deferred, one final gate
blocked.** Of the four partial ones, only **14 and 15** can be picked up
without something from the owner: what remains there is interruption during
image pull or Compose startup, and adopting an interrupted install rather than
only discarding it. Task 29 needs a tag push, 31 needs macOS and Linux hosts,
30 is deferred and 33 waits on 30.

**Outside the 33: three imported apps are now installable, and the setup form
has a real app at last.** PrivateBin, Node-RED and flatnotes are approved;
CodiMD stays withheld because its images were last rebuilt in August 2020.
`src/offerings.rs` is the single lookup behind the catalog, the install review,
the desktop install and the CLI. flatnotes is the first offered app whose
install asks questions — three typed answers and two generated credentials —
so the long-standing note that the setup form had never rendered a real app's
fields no longer applies. A review may also move an image to a different tag of
the same repository, with a stated reason and an audit of the tag that runs.

**Next outside the 33:** a two-service app with typed setup (`runtipi:planka`),
because nothing yet proves a generated credential reaching a second container
in an app somebody actually installs.

**Scope decision, September 8 2026: Windows is the shipping target.** macOS and
Linux proof is deferred by the owner and is no longer counted as outstanding
work. Tasks 17 and 27 are complete for that target; the deferred platform work
is listed at the end of this document so it is not lost. Counts measure deliverables,
not percentage effort. The requested dark-only direction supersedes light mode.

| # | Deliverable | Status | Current evidence / remaining work |
| --- | --- | --- | --- |
| 1 | Reconcile existing documentation | Complete | Historical checkpoints and current phase/status docs |
| 2 | Toolchain/package metadata | Complete | Pinned toolchain, manifest/metadata tests, and an honest declared minimum Rust version checked against the dependency graph and built in CI |
| 3 | Core characterization | Complete | URL, CLI, registry characterization tests |
| 4 | Module boundaries | Complete | Model/storage/catalog/runtime/platform/command/window modules |
| 5 | Canonical identity | Complete | Approved logo integrated; generated desktop exports and mathematical check |
| 6 | Central product identity | Complete | Constants, config and brand tests |
| 7 | Installed-app model | Complete | V2 runtime variants and stable IDs; native window labels now use IDs |
| 8 | Storage/migration | Complete | Fallible writes, immutable backup and recovery tests; packaged test under 31 |
| 9 | Discovery schema | Complete | Shared typed schema, stable ID map, four pinned source adapters, provenance and build validation |
| 10 | Cached paged Rust search | Complete | OnceLock/precomputed search text, 48-row cap, combined filters and facet counts |
| 11 | Recipe manifest | Complete | Schema 3 declares Linux engine, Compose v2, local storage and image platforms; cached upstream tag evidence is checked offline in CI. Exact loopback URL/port validation and Docker-resolved Compose agreement are tested; real lifecycle proof remains under 12/15 |
| 12 | Recipe graduation | Complete | All three previews pass real Windows/Docker Linux-amd64 CLI install, health, stop/start, keep-data reinstall and deletion. A version upgrade with application content is now proven: Memos 0.29.1 installs, an account and a note are created through the running app's own API, a keep-data uninstall preserves the SQLite database, 0.30.0 installs over it, and the same credentials sign in and read that note back. A run that discards data instead fails the test. Evidence: `docs/evidence/recipe-upgrade-memos-windows-2026-09-08.json`. **Other host platforms are deferred by owner decision** |
| 13 | Process runner/Doctor | Complete | Docker/Compose checks, CLI Doctor JSON with readiness exit codes, per-operation deadlines, bounded concurrent capture, process-tree termination, a runner cancellation token and credential redaction on diagnostic surfaces |
| 14 | Install transaction | Partial | Start/health/rollback, port preflight, atomic Compose writes, per-app operation locks, cancellable health wait and commit rollback that preserves earlier data, explicit cleanup failures and live install stages; transaction-wide lock through commit and checkpoint cancellation implemented; cross-process registry locking proven with real subprocesses; all three real CLI installs/commits pass on Windows Docker; cross-process lifecycle exclusion and lock-holder crash release proven; real healthy pre-commit interruption and explicit data-preserving recovery pass for Memos; read-only recovery inventory implemented; optional read-only Docker ownership verification passes with real interrupted Memos; Settings recovery inspection UI validated. **A recovery action now exists**: `local-store recover <id> [--delete-data]` takes the app's operation lock, re-derives the candidate and re-verifies Docker ownership under that lock rather than trusting the snapshot a person was shown, then removes only what the retained Compose file owns. It refuses an ownership mismatch, an app that is properly installed, a busy app, and a removal Docker did not complete; seven unit tests cover those refusals and the real interruption test now recovers through this command instead of a hand-typed `compose down`. Evidence: `docs/evidence/recovery-action-memos-windows-2026-09-08.json`. Interruption during image pull or Compose startup, and adopting an interrupted install rather than clearing it, remain |
| 15 | Lifecycle | Partial | All three real container lifecycles pass on Windows/Docker Linux-amd64, including mounted-file persistence and complete test-resource removal. Fixed IPv6-first localhost health failures. Cross-process lifecycle coordination and atomic uninstall/registry removal now implemented; Memos interrupted-install recovery now passes through the product's own `recover` action, which keeps data by default and preserved unrelated containers on a real run. Other interruption stages remain; **other host platforms are deferred by owner decision** |
| 16 | Navigation/browser opening | Complete | Stable window IDs, validated external URLs on every path, and browser failures surfaced to the launcher and the CLI exit code. Navigation policy is now a pure `decide_navigation` rather than a closure, with tests: an app window may move within its own origin, any other origin opens in the OS browser, and `file:`/`javascript:`/`data:`/custom schemes are refused outright. App windows hold no IPC permission — the capability grant names only `launcher` |
| 17 | Deep links/single instance | Partial | Official plugins, one bootstrap, strict parser, queued errors and background opening. A clean Windows install now proves both schemes register, cold-start activation opens the named app, and a second activation on either scheme reuses the running process; a malformed link is refused with the expected form and exit 2. See `docs/evidence/windows-installer-2026-09-08.json`. **macOS/Linux proof is deferred by owner decision**, so this is complete for the shipping target |
| 18 | Native shortcuts | Complete | Shell-free .url/.webloc files and quoted desktop Exec, with injection/path tests; OS activation under 31 |
| 19 | Typed command results | Complete | Shared typed runtime/storage/IPC errors, compatible CLI failures, launcher-targeted lifecycle events, frontend adapters and contract tests; visible progress remains under 24/25 |
| 20 | CLI semantics | Complete | Native open, filtered/paged catalog, Doctor JSON, strict action argument preflight, and release-image smoke tests proving stdout, exit codes and shell behaviour in the shipped configuration |
| 21 | Static modular frontend | Complete | API/render/app/motion/catalog-control modules and deterministic fixtures |
| 22 | Branded shell | Complete | Dark-only shell, About, Settings and standalone Doctor, responsive visual checks |
| 23 | Discover filters | Complete | Category search, capabilities, software licenses, container architectures, collections, counts and full-size paging |
| 24 | Detail/connect/install UX | Complete | Provenance, live install stages, typed recovery guidance, checkpoint cancellation, an advisory connection address check, and an optional published-port choice validated before Docker is invoked |
| 25 | My Apps lifecycle UX | Complete | Busy/event reconciliation, diagnostics, cancellation, guarded retry, bounded visibility-aware readiness polling with stale reply protection, and connection removal failure/retry/focus coverage; real lifecycle evidence remains under 15 |
| 26 | Accessibility/failure gates | Complete | Axe, field error association, focus restoration, modal keyboard loop, reduced motion and diagnostic/image failures |
| 27 | CSP/capabilities | Partial | Launcher guards, CSP, no release devtools and command/grant consistency tests; Windows remote fixture receives ACL denial for four sensitive commands. The navigation half of the review is done under 16. **Cross-platform privilege review is deferred by owner decision**; the Windows review is complete |
| 28 | Safe icon cache | Complete | Bounded validated build-time cache, 752 offline SVG/PNG icons, checksums and monograms; replaces planned runtime downloader |
| 29 | Quality/build/release CI | Partial | Offline gates, packaged CLI, MSRV and license checks; builds require MSRV, use explicit native targets and stage checksums/unsigned build metadata. Advisory scanning is implemented and passing: `scripts/check-advisories.py` runs cargo-audit, refuses to pass on a missing tool or unfetched database, and requires a recorded reason for every accepted finding — seven transitive Tauri advisories are accepted in `catalog/advisory-policy.json`, and a stale acceptance fails the gate. Authenticated provenance is configured through `actions/attest-build-provenance` with the OIDC permissions it needs. **The matrix is now verified remotely** on PR #3: quality, minimum-rust and all three build targets (windows-2022, ubuntu-24.04, macos-15) pass, with the advisory gate running in the quality job. Provenance itself is still unverified: the release job only runs on a tag, so it skipped |
| 30 | Signed updates | Deferred | **Deferred by owner decision, 8 September 2026: no key is generated and no updater ships until after full deployment.** Prerequisites and the exact commands are in PUBLISH.md for when that happens. Nothing in the build references an updater, so there is no half-built path to trip over. Consequence while deferred: there is no update channel, so a new version reaches people only by downloading and reinstalling |
| 31 | Packaged smoke tests | Partial | Release CLI tests plus reproducible Windows release-image activation/remote IPC harness; unsigned NSIS builds. A clean Windows installer now passes end to end from a machine with no prior install: silent install, complete resource tree, Start Menu shortcut that launches, both schemes registered, packaged CLI reading a registry migrated from the legacy format and taking its lock, then silent uninstall removing the tree, shortcut and Add/Remove entry while preserving user data. Evidence in `docs/evidence/windows-installer-2026-09-08.json`, which also records what this host could not observe. **Windows is the shipping target; macOS/Linux native proof is deferred by owner decision and is not counted against this task** |
| 32 | User/contributor docs | Complete | Current catalog, source licenses with a Rust dependency licence summary, contributor guide, ledger, and a security/privacy document covering enforced boundaries, stated limits, vulnerability reporting and the release checklist |
| 33 | Release-candidate gate | Blocked | Requires combined native/container/security/update/artifact evidence. **Cannot pass while 30 is deferred**, because the update half of that evidence does not exist. Reachable once the key system lands |

## Next development phase

Use the [agent handoff](agent-handoff.md) for bounded next batches, acceptance
criteria, validation commands and the latest CI repair context. No private
agent workspace or prior chat history is required.

Complete runtime reliability and native integration before expanding automatic
installation. Docker commands are now bounded, cancellable at the runner
boundary and redacted before they reach a diagnostic. Progress, cancellation,
readiness polling and official native plugins are implemented. What remains is
interrupted-command recovery and complete native isolation evidence. Run the
original three recipes through isolated real container lifecycle tests on each
supported platform.

Then adapt upstream recipes in batches toward ten and twenty-five verified
installs. Discovery growth continues independently through source refreshes.
Finally complete installer smoke tests, signing/updater, release provenance and
the full release-candidate gate. The current 1,672-project catalog is a discovery
count; three recipes are install previews.

## Deferred: signing and updates (task 30)

Deferred by the owner on 8 September 2026, to be done after full deployment.
The key will be generated then and kept secret. Until then:

- No signing key exists, and none should be generated by an agent.
- No updater plugin, config or UI is in the build. That is deliberate: a
  half-wired updater with a placeholder key is worse than none.
- **There is no update channel.** A new version reaches people only if they
  download and reinstall it. Say so in release notes.
- Windows SmartScreen will warn on install. Code signing is a separate
  purchase from a CA and is also outstanding.
- Task 33 cannot pass until this lands.

`PUBLISH.md` has the commands for when you pick it up.

## Deferred: macOS and Linux

Not cancelled, not counted. Windows is the shipping target as of September 8
2026, and these need hosts nobody has run yet:

- **17** — installer, protocol registration and single-instance proof on both.
- **27** — cross-platform privilege and navigation review.
- **31** — native packaging, shortcuts and activation on both.
- **12/15** — recipe lifecycles on a non-Windows Docker host.

What is *not* missing: both platforms build and bundle cleanly in CI, and the
offline test suite passes on Linux. The gap is native behaviour on a real
desktop — installing, registering a protocol, opening a window — which no CI
runner exercises.

The code paths exist and are cross-platform; what is missing is evidence, and
evidence needs the machine. Pick this up when there is one.
