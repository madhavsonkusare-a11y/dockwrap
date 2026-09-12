# Original Hermes 33-task traceability

Reconciled September 12, 2026. **Historical mapping, not an active backlog.**
Use [V1_TASKS.md](V1_TASKS.md) for current scope, status and execution order.
The original plan is retained in Git history and the private Hermes workspace;
no agent needs that workspace to continue from this mapping.

The prior count of completed core tasks did not measure release effort and did
not include a managed engine, agent access to all apps, V3 or 100 offerings.
Windows remains the shipping target. The owner's new requirement for signed
delivery and automatic updates supersedes the September 8 deferral.

| # | Original deliverable | Reconciled state | V1 follow-through |
| --- | --- | --- | --- |
| 1 | Documentation reconciliation | Historical complete; renewed for V1 | R02 |
| 2 | Toolchain and package metadata | Complete baseline | R04 |
| 3 | Core characterization tests | Complete baseline | R04 |
| 4 | Module boundaries | Complete baseline | E02/A03 reuse them |
| 5 | Canonical identity | Complete baseline; V3 decides changes | F01/F03 |
| 6 | Central product identity | Complete baseline | F03 |
| 7 | Installed-app model | Complete baseline | E02/A01 extend deliberately |
| 8 | Storage and migration | Complete baseline | S04/R04 |
| 9 | Discovery schema | Complete baseline | C01 |
| 10 | Cached paged search | Complete baseline | F03 |
| 11 | Recipe manifests | Complete baseline; exact digest validation fixed | Q01/C03 |
| 12 | Original recipe graduation | Windows lifecycle and Memos upgrade evidence recorded | Q03/Q04 |
| 13 | Bounded runner and Doctor | Complete baseline | E02/E04 |
| 14 | Install transaction | Complete baseline; interruption/recovery evidence exists | E04/Q03 |
| 15 | Lifecycle | Complete Windows baseline | Q03/Q05 |
| 16 | Navigation/browser opening | Complete baseline | F04/R04 |
| 17 | Deep links/single instance | Windows proof recorded; other native platforms deferred | R04 |
| 18 | Native shortcuts | Complete baseline | R04 |
| 19 | Typed command results | Complete baseline | F02/A03 |
| 20 | CLI semantics | Complete baseline; packaged tests pass | R04 |
| 21 | Modular frontend | Complete baseline | F03 reuses modules |
| 22 | Branded shell | Old shell implemented; V3 required | F01/F03 |
| 23 | Discover filters | Complete baseline | F03/F04 |
| 24 | Detail/connect/install UX | Old UI implemented; V3 required | F03/F04 |
| 25 | My Apps lifecycle UX | Old UI implemented; V3 required | F03/F04 |
| 26 | Accessibility/failure tests | Baseline passes; V3 needs fresh proof | F04 |
| 27 | CSP/capabilities | Windows baseline reviewed; agent gateway adds a boundary | A02/A08/R04 |
| 28 | Safe icon cache | 1,678 local icons/monograms in integrated code | C05/F04 |
| 29 | Quality/build/release CI | All PR quality/platform jobs green; tagged provenance not yet proven | R01/R05 |
| 30 | Signed updates | Required now; prior deferral revoked by owner September 12 | S01–S04 |
| 31 | Packaged smoke tests | Historical Windows proof; new V1 engine/UI/update require rerun | R04 |
| 32 | User/contributor docs | Baseline exists; current reconciliation in progress | R02/R05 |
| 33 | Release-candidate gate | Not complete; new V1 requirements must pass | R05 |

## Evidence anchors

- Original runtime/upgrade/native evidence remains under `docs/evidence/`,
  including `recipe-lifecycle-windows-2026-09-07.json`,
  `recipe-upgrade-memos-windows-2026-09-08.json`,
  `recovery-action-memos-windows-2026-09-08.json` and
  `windows-installer-2026-09-08.json`.
- Interrupted startup and adoption have code tests in
  `tests/install_interruption_stages.rs` and `tests/recovery_adopt.rs`.
- Integrated PR validation: [run 34695407242](https://github.com/madhavsonkusare-a11y/local-store/actions/runs/34695407242).
  A green package build is not a tagged release or a clean Windows V1 proof.
- `PUBLISH.md` documents artifact signatures and publication; actual release
  identity and successful updater behavior remain required evidence.

Native macOS/Linux installation, protocol, privilege and container lifecycle
proof remain outside the current Windows release commitment. CI builds alone
must not be advertised as those platforms' certification.
