# Install design — Phase 10

Install is a dedicated, conventional trust surface. It keeps recipe review,
machine consequences, progress, cancellation, and recovery inside one spatial
frame so users do not have to reconstruct what happened across dialogs.

## Backend truth used

The design is grounded in the current recipe, setup-review, operation, and
error contracts:

- Memos uses the pinned image `neosmemo/memos:0.30.0`, publishes loopback port
  5230 to container port 5230, and stores data in its Local Store managed folder.
- The host port is editable today. The backend accepts 1024–65535, rewrites the
  launch and health addresses, and leaves the container port fixed.
- Doctor performs exactly two checks: Docker Engine and Docker Compose.
- The happy path emits `checking_system`, `preparing_files`,
  `validating_recipe`, `starting_containers`, `waiting_for_health`, and
  `saving_app`. `rolling_back` is the explicit failure path.
- Cancel is truthful only before `saving_app`; both `saving_app` and
  `rolling_back` ignore or prohibit cancellation to avoid invisible containers
  or interrupted cleanup.
- Progress has no bytes, duration, download size, or percentage contract, so the
  prototype shows none.

## Design changes

| Before | After | Why |
| --- | --- | --- |
| A short list of five facts | Configuration, disclosed machine changes, compact Doctor status, and optional evidence | The review now answers what, where, and under whose control before execution |
| Port displayed as a fixed machine fact | Inline host-port editor with 1024–65535 validation and an immutable container-port cue | Matches the implemented backend and prevents users from trying to edit the wrong side |
| Generic busy banner | Ordered backend stage list with completed, active, waiting, failed, and rollback states | Stage truth is more useful than invented percentage progress |
| Cancel always implied | Cancel is removed at registry commit and rollback, with an explicit reason | A disabled or ineffective promise is worse than a visible boundary |
| Generic failure messaging | Error-specific consequence and recovery surfaces | Port conflict, Docker absence, invalid input, health timeout, and cleanup failure need different next actions |
| Retry available after any error | Cleanup failure routes to Recovery and offers no retry | Retrying over unknown containers or files is unsafe |
| Success was only a banner | Result surface confirms health, registration, data policy, and routes to Open or selected My Apps | The final state closes the loop and provides a useful next action |

## Review state

The first frame says **Nothing has run yet** and keeps the primary Install action
beside a concise consequence summary. The visible review includes:

- app identity and reviewed-recipe status;
- editable loopback host port and fixed container port;
- current setup requirements (none for reviewed Memos);
- one container, one published loopback address, and one managed data folder;
- Docker and Compose readiness;
- collapsed recipe evidence containing exact image/version, health address,
  restart policy, license, verification date, compatible image platforms, image
  digest, risk notes, and rollback behavior.

The setup-field variant is labeled **Component study** because the current
reviewed Memos recipe has no fields. It exercises the existing `SetupReview`
shape: ordinary defaults are visible and retained, sensitive inputs use a
password control, sensitive defaults are never exposed, and generated
credentials are counted rather than displayed.

## Progress and cancellation

Each stage retains its label and one short consequence. Completed steps remain
visible, which lets a user understand the active step without interpreting a
spinner. A small orbit communicates indeterminate activity; reduced motion
removes the rotation.

At `saving_app`, the UI explains that the healthy container is being committed
to My Apps so it cannot become invisible. At `rolling_back`, the failed health
step remains visibly failed and cleanup becomes the active, uncancellable step.
The original failure is not replaced by a generic loading state.

## Failure and recovery matrix

| Error | Machine consequence shown | Primary recovery |
| --- | --- | --- |
| `port_in_use` | No files or containers created | Edit the retained port, then return to review |
| `prerequisite_unavailable` | No files or containers created | Open the real two-check system diagnostic |
| `invalid_input` | No container started; incomplete files rolled back if needed | Return to retained configuration |
| `timed_out` | Incomplete setup rolled back | Review and retry after checking Docker |
| `rollback_failed` | Containers or files may remain | Review retained setup in Recovery; retry omitted |
| `cancelled` | Nothing added to My Apps; cleanup requested | Return to review with the valid port retained |

Success makes **Open Memos** primary and **View in My Apps** secondary. The My
Apps route selects Memos instead of dropping the user at an unrelated list row.

## Interaction-level critique

The evaluation toolkit’s UXBench framing was applied after browser interaction.

| Step and action | Finding | Resolution |
| --- | --- | --- |
| Review · press Install with port 80 | Validation could have failed without moving attention | Inline error appears and focus remains on the invalid port |
| Review · press Install with port 8080 | Immediate acknowledgement had to precede backend events | The screen switches synchronously to `checking_system`; measured prototype feedback was 44ms |
| Installing · press Cancel | Cancellation needed a terminal explanation | Cancelled result states that nothing entered My Apps and retains port 8080 |
| Commit · inspect `saving_app` | Removing Cancel without explanation would hide a consequence | A commit-boundary message explains why the action is unavailable |
| Port failure · enter 5231 and recover | A recoverable error could discard corrected input | Review reopens with 5231 intact |
| Cleanup failure · inspect actions | A generic retry would be actively unsafe | Retry is absent; Recovery is the primary route |
| Success · choose View in My Apps | A generic route could lose task continuity | My Apps opens with Memos selected |

No remaining interaction finding in this phase has severity above 0: feedback
is immediate, validation is visible, consequences precede the point of no
return, every terminal state has a safe route, and valid configuration survives
recoverable branches.

## Cognitive walkthrough

| User question | Result |
| --- | --- |
| Do I know what I am about to install? | App identity, reviewed status, exact image, port, and local data consequence are visible before the action. |
| Can I tell what will change? | Three machine-change cards name the container, loopback binding, and managed folder. |
| Will I understand that Install worked? | Feedback moves to the first real backend stage immediately and keeps the task title visible. |
| Can I stop safely? | Cancel is present only while the backend can honor it; later stages explain the boundary. |
| Can I recover from likely errors? | Every typed failure has one safe primary route and retains valid choices. |
| Can I finish the job? | Success offers Open and selected My Apps; unsafe cleanup goes directly to Recovery. |

## Verification

- Rendered at 1440×900 and 1280×800 with no horizontal overflow.
- Axe-clean for review, setup-field study, expanded evidence, loading,
  unavailable recipe, every progress/cancellation boundary, success, cancelled,
  and all five typed failures at both laptop viewports.
- Browser-driven checks cover invalid/valid ports, sub-100ms acknowledgement,
  cancellation, commit cutoff, retained port recovery, unsafe retry omission,
  selected-app success routing, and masked secret controls.
- The foundation checker verifies the exact stage order, error fixtures, recipe
  fields, cancellation cutoff, and offline icon.

## Deterministic review URLs

- Review: `index.html#install`
- Setup-field study: `index.html?setup=fields#install`
- Expanded evidence: `index.html?evidence=1#install`
- Installing: `index.html?state=busy&stage=starting_containers#install`
- Commit cutoff: `index.html?state=busy&stage=saving_app#install`
- Rollback: `index.html?state=busy&stage=rolling_back#install`
- Success: `index.html?state=success#install`
- Cancelled: `index.html?outcome=cancelled#install`
- Port conflict: `index.html?state=failure&failure=port_in_use#install`
- Docker unavailable: `index.html?state=failure&failure=prerequisite_unavailable#install`
- Invalid configuration: `index.html?state=failure&failure=invalid_input#install`
- Health timeout: `index.html?state=failure&failure=timed_out#install`
- Unsafe cleanup failure: `index.html?state=failure&failure=rollback_failed#install`

This remains a visual prototype. No Docker command, file write, app registration,
or backend mutation is triggered by the install controls.
