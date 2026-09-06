# Hermes upgrade status

Updated September 6, 2026. This is the current ledger for the original 33-task
plan; earlier phase documents are historical checkpoints. A complete core
deliverable does not replace the separately counted native/release gates.

**16 core deliverables complete, 14 partial, two not implemented, one final gate
not run. Seventeen tasks still have work remaining.** Counts measure deliverables,
not percentage effort. The requested dark-only direction supersedes light mode.

| # | Deliverable | Status | Current evidence / remaining work |
| --- | --- | --- | --- |
| 1 | Reconcile existing documentation | Complete | Historical checkpoints and current phase/status docs |
| 2 | Toolchain/package metadata | Complete | Pinned toolchain, manifest/metadata tests; minimum-Rust CI remains under 29 |
| 3 | Core characterization | Complete | URL, CLI, registry characterization tests |
| 4 | Module boundaries | Complete | Model/storage/catalog/runtime/platform/command/window modules |
| 5 | Canonical identity | Complete | Approved logo integrated; generated desktop exports and mathematical check |
| 6 | Central product identity | Complete | Constants, config and brand tests |
| 7 | Installed-app model | Complete | V2 runtime variants and stable IDs; native window labels now use IDs |
| 8 | Storage/migration | Complete | Fallible writes, immutable backup and recovery tests; packaged test under 31 |
| 9 | Discovery schema | Complete | Shared typed schema, stable ID map, four pinned source adapters, provenance and build validation |
| 10 | Cached paged Rust search | Complete | OnceLock/precomputed search text, 48-row cap, combined filters and facet counts |
| 11 | Recipe manifest | Partial | Three pinned manifests; finish build schema, requirements and platform support metadata |
| 12 | Recipe graduation | Partial | Three install previews; real install/health/restart/upgrade/preserve/delete evidence absent |
| 13 | Process runner/Doctor | Partial | Docker/Compose checks; deadlines, bounded capture, cancellation, redaction and JSON output remain |
| 14 | Install transaction | Partial | Start/health/rollback; port checks, atomic config, progress, concurrency and commit rollback remain |
| 15 | Lifecycle | Partial | Start/stop/status/logs/uninstall; readiness and real data-preservation evidence remain |
| 16 | Navigation/browser opening | Partial | Stable window IDs now used; native navigation isolation and surfaced browser errors remain |
| 17 | Deep links/single instance | Partial | Validated startup parser and legacy-name handling; official plugins/existing-process delivery remain |
| 18 | Native shortcuts | Complete | Shell-free .url/.webloc files and quoted desktop Exec, with injection/path tests; OS activation under 31 |
| 19 | Typed command results | Partial | Typed success data; stable error codes, events and integration tests remain |
| 20 | CLI semantics | Partial | Native open fixed; parser replacement, paged catalog and Doctor JSON remain |
| 21 | Static modular frontend | Complete | API/render/app/motion/catalog-control modules and deterministic fixtures |
| 22 | Branded shell | Complete | Dark-only shell, About, Settings and standalone Doctor, responsive visual checks |
| 23 | Discover filters | Complete | Category search, capabilities, software licenses, container architectures, collections, counts and full-size paging |
| 24 | Detail/connect/install UX | Partial | Provenance and richer details added; connection checks, setup options and real progress/retry diagnostics remain |
| 25 | My Apps lifecycle UX | Partial | Actions exist; live operation/health state and complete removal coverage remain |
| 26 | Accessibility/failure gates | Complete | Axe, field error association, focus restoration, modal keyboard loop, reduced motion and diagnostic/image failures |
| 27 | CSP/capabilities | Partial | Local-only image CSP and launcher guards; release devtools and native privilege review remain |
| 28 | Safe icon cache | Complete | Bounded validated build-time cache, 508 offline SVGs, checksums and monograms; replaces planned runtime downloader |
| 29 | Quality/build/release CI | Partial | Adds offline catalog/icon/brand gates; dependency/license checks, MSRV, explicit architectures and release provenance remain |
| 30 | Signed updates | Not implemented | Official updater, signatures, update UI and credential setup |
| 31 | Packaged smoke tests | Not implemented | Clean installer, native shortcuts/navigation and packaged migration harness |
| 32 | User/contributor docs | Partial | Current catalog, source licenses, contributor guide and ledger; security/privacy/release procedures remain |
| 33 | Release-candidate gate | Not run | Requires combined native/container/security/update/artifact evidence |

## Next development phase

Use the [agent handoff](agent-handoff.md) for bounded next batches, acceptance
criteria, validation commands and the latest CI repair context. No private
agent workspace or prior chat history is required.

Complete runtime reliability and native integration before expanding automatic
installation: bound/cancel Docker processes, add operation progress and typed
errors, close transaction/concurrency gaps, finish official deep links and prove
remote-window isolation. Run the original three recipes through isolated real
container lifecycle tests on each supported platform.

Then adapt upstream recipes in batches toward ten and twenty-five verified
installs. Discovery growth continues independently through source refreshes.
Finally complete installer smoke tests, signing/updater, release provenance and
the full release-candidate gate. The current 1,672-project catalog is a discovery
count; three recipes are install previews.
