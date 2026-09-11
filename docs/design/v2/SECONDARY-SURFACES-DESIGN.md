# Activity, Settings, and Recovery — Phase 12

These surfaces handle infrequent but consequential work. Activity answers what
happened, Settings answers whether the local environment is ready, and Recovery
answers what Local Store can safely inspect or remove. None uses decorative
charts or fabricated preferences.

## Design changes

| Before | After | Why |
| --- | --- | --- |
| Four undifferentiated Activity rows | Searchable, app-filtered, type-filtered timeline with expandable operation evidence | Operational questions are easier to answer when identity, outcome, time, and diagnostics stay together |
| Activity looked like durable history | Persistent amber contract banner plus a separate current-session signal | Live events exist, but timestamps and cross-session history do not |
| Settings was a stack of descriptive rows | System-readiness hero, real two-check Doctor results, recovery scan, offline catalog facts, build facts, and introduction re-entry | Settings now prioritizes controls and support facts people can act on |
| Docker failure was a generic banner | Failed prerequisite names its consequence and recovery instruction while linked apps remain usable | The failure boundary is clear without overstating impact |
| Recovery exposed one vague “clear” action | Separate stop-and-preserve and permanent-delete paths, each with a consequence review | Data preservation and data destruction must never look interchangeable |
| Displayed ownership snapshot implied permission | Every cleanup says it re-verifies under an operation lock; mismatch removes all cleanup actions | The Rust safety contract is visible at the point of action |
| Permanent deletion needed one click | Dedicated danger surface plus exact app-name confirmation | Irreversible data loss receives extra friction and distinct language |

## Activity model

The current `OperationEvent` provides operation ID, app ID, kind, state,
optional install stage, optional error, and an install cancellation token. It
has no timestamp and is not persisted. The V2 Activity model therefore keeps
two layers visibly distinct:

| Layer | Fields | Truth status |
| --- | --- | --- |
| Current session | App, operation kind, state, stage, error, operation ID | Available now from launcher events |
| Durable history | Timestamp, day grouping, search, cross-session app/type filters | Backend work |

The concept timeline supports All, Installs, Lifecycle, and Failures; an app
selector; text search; grouped dates; and expandable raw evidence. It omits CPU,
memory, uptime, duration, and success-rate charts because the backend does not
collect those signals and no current operational question requires them.

Empty and failed-history states preserve access to My Apps and explain that
live in-context operation feedback remains authoritative. A live install state
uses the real indeterminate operation/stage model rather than a percentage.

## Settings model

System readiness is the first and strongest Settings section. It contains the
only two Doctor checks—Docker Engine and Docker Compose—and makes **Run Docker
check** the primary action. Busy, ready, and failure conditions keep the same
geometry so results do not jump around.

Diagnostics & Recovery owns the read-only retained-setup scan. Its explanatory
copy states that scanning reads managed paths and Docker labels but never starts,
stops, or deletes anything. Candidate identity routes into the dedicated
Recovery task.

The right column contains bundled catalog facts and build information. These
are support facts, not fake settings. Runtime preferences and updater controls
remain absent because there is no backend settings model. First Run re-entry is
visible but labeled Concept until an onboarding-completion field exists.

## Recovery safety model

`inspect_recovery` is available in the launcher today. `recovery::discard`
implements protected cleanup in Rust, but there is no launcher mutation command,
so cleanup controls are explicitly marked **Cleanup needs launcher wiring**.

| Ownership/result state | Cleanup offered | Explanation |
| --- | --- | --- |
| `verified` | Stop owned containers; review permanent deletion | Displayed snapshot is useful evidence, but action-time verification still runs |
| `no_containers` | Preserve files or review deletion | No container is claimed; retained setup/data may still matter |
| `mismatch` | None | A Compose project-name collision could belong to another service |
| `not_checked` or scan failure | None | Unknown ownership cannot authorize a mutation |
| Keep-data success | Reports containers removed and `data_deleted: false` | Compose and data remain available for manual review |
| Delete-data success | Reports containers removed and `data_deleted: true` | Docker succeeds before the managed directory is deleted |
| Docker cleanup failure | Reports that nothing was deleted | Files remain when Docker refuses cleanup, preserving recoverability |

The permanent branch names the managed Memos directory, requires typing
`Memos`, and repeats the order: re-verify ownership, stop matching containers,
then delete files. The working state cannot be cancelled because interrupting
re-verification or cleanup would weaken the contract.

## Browser-driven interaction critique

The evaluation toolkit’s interaction framing was applied after driving the
surfaces rather than judging screenshots alone.

| Step and action | Finding | Resolution |
| --- | --- | --- |
| Activity · choose Failures | Filters needed observable, reversible feedback | Two failure rows remain immediately and Clear filters restores all five |
| Activity · search for Memos inside Failures | Combined filters could create an unexplained blank | Inline empty copy names filters and retains a one-click reset |
| Activity · expand a failed Open event | Summary alone hid evidence useful for diagnosis | Operation ID, app ID, last stage, and error code expand in place |
| Settings · run Docker check | Generic loading would obscure which checks exist | Both real checks keep their rows, switch to checking, then resolve in place |
| Settings · scan recovery | A destructive-sounding scan could create fear | Copy says read-only before the action; the task opens in a scanning state |
| Recovery · review container cleanup | “Clear” did not communicate preservation | Confirmation explicitly says every setup and data file remains |
| Recovery · review data deletion | A danger color alone was insufficient | Dedicated consequence surface, exact-name confirmation, and disabled action |
| Recovery · confirm deletion | Snapshot-to-action race could remain invisible | Working state says ownership is rechecked under the operation lock before cleanup |
| Recovery · inspect mismatch | A generic retry could expose unsafe cleanup | Both cleanup actions are absent; scan and Settings are the only routes |

No action silently validates, no displayed snapshot is treated as current
authority, and every failure or empty state has a route that does not require a
machine mutation.

## Verification

- 21 deterministic Activity, Settings, and Recovery states are axe-clean at
  1440×900 and 1280×800.
- No state has horizontal viewport overflow at either laptop size.
- Browser interaction verifies combined Activity filters, reset, event detail,
  Doctor feedback, recovery scan, typed destructive confirmation, protected
  working state, success result, and ownership-mismatch refusal.
- The phase checker is `node scripts/check-v2-secondary-surfaces.mjs` while the
  local preview server is running on port 8765.

## Deterministic review URLs

### Activity

- Default: `index.html#activity`
- Live operation: `index.html?state=busy#activity`
- Empty history: `index.html?state=empty#activity`
- Failed history read: `index.html?state=failure#activity`
- Expanded failed event: `index.html?event=op-102#activity`
- Failure filter: `index.html?activity-filter=failed#activity`

### Settings

- Ready: `index.html#settings`
- Checking: `index.html?state=busy#settings`
- Docker failure: `index.html?state=failure#settings`
- No recovery candidates: `index.html?state=empty#settings`

### Recovery

- Verified candidate: `index.html#recovery`
- No matching containers: `index.html?recovery=no-containers#recovery`
- Stop-and-preserve confirmation: `index.html?recovery=confirm-keep#recovery`
- Permanent-delete confirmation: `index.html?recovery=confirm-delete#recovery`
- Protected cleanup: `index.html?recovery=working#recovery`
- Keep-data result: `index.html?recovery=success-keep#recovery`
- Delete-data result: `index.html?recovery=success-delete#recovery`
- Ownership mismatch: `index.html?recovery=mismatch#recovery`
- Scan failure: `index.html?recovery=scan-failure#recovery`

This remains a visual prototype. It does not persist Activity history, invoke
Docker, expose the Rust discard function to the launcher, or delete files.
