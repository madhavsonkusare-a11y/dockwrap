# Local Store V2 evaluation

Updated: September 11, 2026  
Status: **Phase 14 complete** — every gate passes; two cosmetic findings deferred to Phase 16 with reasons  
Method source: `docs/design/ux-evaluation-toolkit.html`  
Follow-on: Phase 15 laptop visual verification is recorded in `visual-verification.md`

## Result

| Gate | First pass | Full-matrix baseline | Final |
| --- | --- | --- | --- |
| States × viewports | 3 × 2 | 88 × 2 = 176 renders | 176 renders |
| axe, WCAG 2.0/2.1/2.2 A+AA | 0 | 8 | **0** |
| Focus indicator ≥3:1, real keyboard walk, no modal escape | not run | 18 reported, all false positives; 1 real modal escape | **0** across 2,560 focus stops |
| Input boundary ≥3:1 (WCAG 1.4.11) | not run | 102 | **0** |
| Dark-mode legibility (13px, or an 11px letter-spaced label) | 147 nodes on 3 screens | 2,296 nodes per viewport | **0** |
| Token drift outside documented exceptions | 18 (partial scan) | 50 | **0** |
| Undefined `var(--v2-*)` references | not run | 6 | **0** |
| Findings of severity ≥2 | 2 | 14, plus 2 more surfaced while correcting | **0 open** |

**How to run the gates.** Serve the repository root with
`python -m http.server 8765 --bind 127.0.0.1`, then:

```bash
node scripts/check-v2-evaluation-gates.mjs            # hard gates, ~15 minutes
node scripts/check-v2-evaluation-gates.mjs --report   # every finding as JSON
node scripts/check-v2-evaluation-fixes.mjs            # behavioural proof of each correction
```

The five earlier phase suites (`check-v2-prototype`, `-first-run`, `-motion`,
`-secondary-surfaces`, `-grain-telemetry`) still pass after every correction.

## Completed in the bounded first pass

- Explicit WCAG 2.0/2.2 AA axe scan of Overview, Discover, and Install at
  1440×900 and 1280×800.
- Dark-mode small-text inventory using the toolkit's stricter 13px-or-spaced-label
  rule.
- Raw radius and font-size drift inventory.
- Nielsen heuristic pass over the Overview → Discover → Install primary path.
- Krug trunk tests for Overview, Discover, and Install, plus synthetic five-second
  tests for Overview, Discover, First Run, and Install.
- Interaction review of the reviewed-recipe install path.

## Automated gate results — first pass (superseded by the table above)

| Gate | Result | Evidence |
| --- | --- | --- |
| WCAG 2.0/2.2 AA axe | Pass | Zero violations across 3 screens × 2 laptop viewports |
| Dark-mode legibility | Needs work | 39 small-text nodes on Overview, 56 on Discover, and 52 on Install at each viewport |
| Radius consistency | Needs review | 12 raw values remain outside radius tokens, including 3px, 5px, 9px, 11px, 13px, 15px, 17px, and 18px |
| Type consistency | Needs review | Six raw font sizes remain outside semantic type tokens: 10px, 11px, 25px, 27px, 30px, and 31px |

The small-text result is not an axe contradiction. Axe verifies formal contrast and
semantics; the toolkit deliberately adds a stricter dark-mode acuity gate. The
worst repeated examples are the 9px navigation counts, 9px environment version,
10px workspace label, and 11px environment title.

## Nielsen heuristic scorecard — primary path

Severity: 0 = no issue, 1 = cosmetic, 2 = minor, 3 = major, 4 = blocker.

| Heuristic | Severity | Evidence and correction |
| --- | ---: | --- |
| Visibility of system status | 0 | Docker readiness, app attention, install stage, and non-live concept data are explicitly labeled. |
| Match with the real world | 0 | “Managed,” “linked,” ports, storage, health, and rollback use concrete self-hosting language. |
| User control and freedom | 0 | Install review has a safe exit; drawers and dialogs support explicit close and Escape; selection and filters persist. |
| Consistency and standards | 1 | “Install preview,” “Review install,” and “Reviewed recipe” describe one capability. Standardize on “Reviewed install.” |
| Error prevention | 0 | Ports validate before execution; machine changes and rollback are disclosed; destructive actions are separated. |
| Recognition rather than recall | 0 | Current location, selected app, exact recipe, and next action stay visible. |
| Flexibility and efficiency | 2 | The global “Search anything” control advertises Ctrl+K, but only reveals that search is conceptual after activation. Label the unavailable capability before activation or implement the palette contract. |
| Aesthetic and minimalist design | 3 | Hierarchy and identity are strong, but the repeated 9–11px text is too dependent on high visual acuity in a dark UI. Promote operational copy to at least 13px; retain smaller sizes only for genuinely secondary, letter-spaced labels. |
| Help users recover from errors | 0 | Install failure and recovery paths preserve context, explain what changed, and provide specific next actions. |
| Help and documentation | 1 | Inline guidance is strong, but recipe evidence does not yet link to a durable explanation of ownership and rollback terminology. Add contextual documentation in production handoff. |

No blocker was found. The highest-priority correction is typography legibility,
followed by truthful treatment of the unimplemented global search affordance.
Both are resolved — see F-01 and F-13 in the register.

## Krug trunk test

| Screen | Where am I? | What can I do? | One level up | Result |
| --- | --- | --- | --- | --- |
| Overview | Selected rail item, breadcrumb, and “Workspace health” heading | Add an app or review the three attention items | Workspace navigation rail | Pass |
| Discover | Selected rail item, breadcrumb, offline-catalog label | Search/filter, inspect a project, connect, or review an install | Workspace navigation rail | Pass |
| Install | “Review install” task heading and recipe identity | Edit the port, inspect evidence, install, or cancel | Explicit back/cancel route to Discover | Pass |

## Synthetic five-second test

| Screen | Likely first impression | Primary action clarity | Result |
| --- | --- | --- | --- |
| Overview | A local self-hosted app workspace showing health and attention | “Review 3 items” is dominant and specific | Pass |
| Discover | An offline self-hosted catalog with three trusted install recipes | Recipe cards and Connect are understandable; the page intentionally supports more than one task | Pass |
| First Run | A private home for self-hosted apps that preserves user control | “Get started” is unmistakable; “Skip for now” remains available | Pass |
| Install | A safety review before running a container | “Install Memos” is singular, visible, and preceded by consequences | Pass |

## Interaction-only findings

The reviewed-recipe path gives feedback within the same event task, rejects an
invalid port at the field, discloses machine changes before commitment, provides
stage-based progress without a false percentage, and offers recovery after failure.
Returning to Discover preserves the prior search/filter state. No silent validation,
hidden point-of-no-return consequence, or dead end was found in this path.

## Pass 2 — full-matrix baseline (September 11, 2026)

The first pass covered 3 screens × 2 viewports. The gate script was extended to
every screen × all six conditions plus every focused substate the prototype's URL
contract exposes: dialogs, drawers, log substates, all 11 recovery modes, all 7
install stages, all 5 install failures, and every First Run branch — **88 states
× 2 laptop viewports = 176 renders**. Results before any correction:

| Gate | Result | What it found |
| --- | --- | --- |
| axe (WCAG 2.0/2.1/2.2 A+AA) | **8 violations** | `aria-prohibited-attr` on the Overview and Discover skeletons and the logs skeleton (an `aria-label` on a role-less `div`); `color-contrast` on two prototype-panel lines set in `--v2-text-quiet` (3.72:1) |
| Input boundaries (WCAG 1.4.11) | **102 failures** | Every text input and select has a 1.26–1.51:1 boundary against its surface. Axe cannot detect this: its contrast rule is text-only |
| Dark-mode legibility | **2,296 nodes per viewport** | Four sizes: 12px (1,036, the `--v2-type-caption` token), 9px (585), 10px (524), 11px (151) |
| Token drift | **50 raw values** | 12 radii, 6 font sizes, 17 colours, 15 spacing values outside the scale |

**The first pass's drift scan had a blind spot.** It matched only `font-size:`.
Fifty-one `font:` shorthand declarations set sizes of 9, 10 and 11px, all of them
IBM Plex Mono — so the machine-fact layer, whose job is to make verifiable facts
look verifiable, is the smallest text in the product. That is the root cause of
the severity-3 legibility finding, not the caption token alone. The gate now reads
shorthand sizes too.

**Focus-indicator results needed correcting before they could be trusted.** The
first probe called `element.focus()`, which does not reliably match
`:focus-visible`; it reported 18 controls with no indicator. A real keyboard walk
showed every one of them — the skip link, the panel close button, the six
condition buttons, the concept checkbox — renders a 2px `#ff9b5c` ring. All 18
were false positives. The gate now walks focus with real Tab presses.

That walk surfaced one real defect instead: the prototype panel declares
`aria-modal="true"`, but Tab leaves it and lands on `body` and then the skip link
behind it. See the findings register below.

**Tooling caveat worth keeping.** During exploratory testing, the Playwright MCP
browser reported a single Tab press changing the route twice. A clean Playwright
run reproduced nothing: fourteen Tab presses, no navigation, a logical order. The
MCP session was the artifact. Confirm any keyboard finding in a clean run before
recording it.

## Nielsen heuristics — remaining screens

Scored the same way as the primary path. Only non-zero heuristics are listed; every
unlisted heuristic scored 0 for that screen. Finding IDs refer to the register.

| Screen | Heuristic | Sev | Finding |
| --- | --- | ---: | --- |
| My Apps | Consistency and standards | 2 | An `error` app renders in the same warning yellow as a `stopped` app. Stopped is a state the user chose; error is a fault. `statusClass()` maps anything unrecognised to warning. **F-10** |
| My Apps | Consistency and standards | 2 | Two ember-filled actions compete: the header's "Discover" and the detail's "Open app". The system allows one primary per view. **F-11** |
| My Apps | Aesthetic and minimalist | 2 | Three levels of bordered containers — panel, detail, then separate status, fact-grid and disclosure cards inside it. **F-14** |
| My Apps | Match with the real world | 1 | "Object type", "Registry updated" and "Catalog association" are internal vocabulary. **F-16** |
| My Apps | Aesthetic and minimalist | 1 | Row subtitles repeat the status the row badge already shows ("Managed app · Running" beside RUNNING). **F-24** |
| Activity | Match with the real world | 1 | The screen description reads "an implementation model for durable operation history" — design-process language in product copy. **F-19** |
| Settings | Aesthetic and minimalist | 2 | "Scan again" wraps onto two lines inside its pill at 1440×900. It reads as broken. **F-12** |
| Settings | Consistency and standards | 1 | The retained-setup row offers a "Review" chip and an "Open →" link for one destination. |
| Recovery | — | 0 | No findings. Ownership language, itemised consequences and typed confirmation are exemplary. |
| First Run | Match with the real world | 1 | "Docker Doctor is running while you read" exposes an internal feature name to a first-time user. **F-18** |
| First Run | Aesthetic and minimalist | 1 | The three promise rows sit directly under the lede and directly above the actions, with no separation. **F-23** |
| Install | Help users recover from errors | 2 | After `port_in_use`, the retry field is pre-filled with the port that just failed and "Review new port" accepts it unchanged. **F-09** |
| Install | Consistency and standards | 1 | The same screen says `localhost:5230` in the summary and `127.0.0.1:` in the port control. **F-17** |
| Global | Flexibility and efficiency | 2 | Carried from pass 1: "Search anything" advertises Ctrl K and only reveals that it is a concept after activation. **F-13** |
| Global | Consistency and standards | 1 | Carried from pass 1: "Install preview", "Review install" and "Reviewed recipe" name one capability. **F-15** |

## Krug trunk test — remaining screens

| Screen | Where am I? | What can I do? | One level up | Result |
| --- | --- | --- | --- | --- |
| My Apps | Selected rail item, breadcrumb, "My Apps" heading, selected row | Filter, select, open, start/stop, logs, manage, connect | Rail | Pass |
| Activity | Selected rail item, breadcrumb, heading | Search, filter by type or app, expand an event | Rail | Pass |
| Settings | Selected rail item, breadcrumb, heading | Run the Docker check, review recovery, re-run the introduction | Rail | Pass |
| Recovery | Heading and back link — **but no rail item is selected** | Review cleanup or deletion | "Back to Diagnostics & Recovery" | Pass with finding |
| First Run | Own frame; step rail shows position | Get started, skip, or take an alternate route | Skip returns to the workspace | Pass |

Focused tasks — Install and Recovery — clear the rail selection entirely, and the
breadcrumb names only the task ("Local workspace / Install"), not its parent. The
back link carries the whole "one level up" answer on its own. Highlight the parent
section in the rail and add it to the breadcrumb. **F-20**, severity 1.

## Cognitive walkthroughs

Four questions per step: will the user try to achieve this effect; will they notice
the correct action; will they associate it with the effect; will they see progress?

### First Run

| Step | Try | Notice | Associate | Progress | Notes |
| --- | --- | --- | --- | --- | --- |
| Welcome | Yes — the promise is one sentence | "Get started" is the only filled control | Yes | Preflight row resolves in place | "Docker Doctor" is unexplained (F-18) |
| Docker missing | Yes — the heading is the instruction | "Check again" is primary; Connect and Browse are clearly secondary | Yes | Engine and Compose rows change state; Compose says it is waiting on Engine | Strong. "Nothing is broken in your workspace" answers the first worry |
| Choose | Yes | Memos is preselected and marked | Yes — button names the recipe | Port is editable with an explicit "a full review appears before Docker runs" | Pass |
| Install | See Install below | | | | |
| Ready | Yes | "Open Memos" primary, "Enter workspace" secondary | Yes | Success state names the address | Pass |

### Connect — from a catalog entry and from the global action

| Step | Try | Notice | Associate | Progress | Notes |
| --- | --- | --- | --- | --- | --- |
| Open drawer | Yes | "Connect" on the card, or "Connect app" globally | Yes | Drawer title names the app | Consequences are disclosed up front: it does not install, start, stop, update or back up |
| Enter address | **At risk** | The field already contains `http://immich.home` in value styling | — | — | **F-02, severity 3.** The address is fabricated from the catalog ID and rendered as a real value. The drawer's own copy says saving succeeds even when the server is offline, so one click on the primary button saves an address the user never typed |
| Enter name | Yes | Name is pre-filled from the catalog | Yes | — | **F-07, severity 2.** Clear the name and save: the field gets `aria-invalid` and focus, and nothing visible says why |
| Save | Yes | "Save linked app" is primary | Yes | Toast, then My Apps | **F-08, severity 2.** From the global action there is no catalog ID, so the prototype selects Memos — the user lands looking at an app they did not just add |

### Install

The primary path passed in pass 1. The failure branches were walked here.

| Branch | Result | Notes |
| --- | --- | --- |
| `prerequisite_unavailable` | Pass | Routes to the system check; review is preserved |
| `invalid_input` | Pass | Returns to review with values kept |
| `timed_out` / health | Pass | Explains the rollback, offers review and retry |
| `rollback_failed` | Pass | Retry is deliberately withheld and the user is routed to Recovery — the correct refusal |
| `port_in_use` | **Fail** | **F-09, severity 2.** The retry field shows 5230, the port that just failed. "Review new port" accepts it unchanged and returns to a review that no longer mentions the conflict. Pressing Install reproduces the identical failure |
| Cancel | Pass | Cancel disappears at `saving_app`; "Nothing was added to My Apps" is stated |

### Recovery

| Step | Try | Notice | Associate | Progress | Notes |
| --- | --- | --- | --- | --- | --- |
| Candidate | Yes | Keep and delete are separate rows; delete is red | Yes | "Ownership snapshot verified" | "Evidence to review, not permission to act" sets the right expectation |
| Confirm delete | Yes | Typed confirmation gates the button | Yes | Consequences itemised before commitment | Exemplary |
| Mismatch / scan failure | Yes | Cleanup is refused with the reason | Yes | — | Refusal is explained, not hidden |

## Interaction-only findings — every flow

Using the toolkit's UXBench classes, which only show up by driving the interface:

| Class | Found | Where |
| --- | --- | --- |
| Missing feedback within 100ms | None | Every action gives same-task feedback; Phase 13 verified the timings |
| Silent validation | **1** | Connect name field (F-07) |
| Hidden consequence | **2** | Fabricated connect address (F-02); global connect lands on the wrong app (F-08) |
| Weak recovery path | **1** | Port-conflict retry (F-09) |
| State lost on Back | None | Discover filters, My Apps selection, install values and First Run choices all survive Back |

## Anti-generic audit

Toolkit pick 02: could each element appear unchanged in any other product?

| Element | Verdict | Evidence |
| --- | --- | --- |
| Typeface | Distinctive | Instrument Sans with a mono layer reserved for machine-asserted facts |
| Accent | Grounded | Ember sits in the infra-tool neighbourhood (Hacker News, Cloudflare, Ubuntu) and pairs with the grain mark. "Dark with one warm pop" is a known AI default; the grain and warm neutrals are what keep it from reading as one |
| Card treatment | **Generic** | Rounded, hairline-bordered cards nested three deep on My Apps is the "container nesting soup" fingerprint (F-14) |
| Radius discipline | **Drifting** | 12 raw radii outside four tokens |
| Icons | Distinctive | Real catalog logos throughout; custom stroked sprite |
| Layout rhythm | Partly generic | Eyebrow → title → description repeats on every screen. Where the eyebrow restates the navigation ("Your workspace", "System & support") it is filler (F-21) |
| Selection marker | Acceptable | The left accent bar is a listed fingerprint, but here it marks only the selected row and nav item — functional, not decorative |
| Copy voice | Distinctive | "This snapshot is evidence to review, not permission to act." No other product says that |

**The detail only this product has:** the port row — `127.0.0.1: [5230] → 5230` — and
an install list that mirrors the backend's six stage IDs in order. Both would be
meaningless anywhere else.

## Three-perspective critique

Three voices reviewed the same screens: an **Operator** (a self-hoster running
twenty containers), a **Brand designer**, and an **Accessibility specialist**.

| Question | Operator | Brand | Accessibility | Synthesis |
| --- | --- | --- | --- | --- |
| Type size | Smaller mono fits more facts | Larger type reads as calmer | 13px floor in a dark UI | **13px floor for reading text and values; 11px only for letter-spaced tags.** The facts that matter most are the ones that were smallest |
| Ember grain | Noise on operational screens | Wants it on more surfaces | Never behind text | Signature surfaces only — Discover feature, First Run, empty Overview. The prototype already complies |
| Prototype truth labels | Values the honesty | Clutter | Neutral | Keep in the prototype, strip in production. They must share one annotation style so implementers can find every one (F-22) |
| `error` status colour | Must be red | Too much red alarms | Colour plus text | `error` → danger, `stopped` stays neutral (F-10) |
| Input borders | Indifferent | Visible grey borders look heavier | 3:1 is required | **WCAG 1.4.11 is not negotiable.** Use the warm `neutral-450` so the border sits with the palette (F-03) |
| Primary actions | One obvious next step | One ember fill per view | One focus target | Unanimous: one primary (F-11) |

The only real disagreement was type size, and it resolved in the accessibility
direction on evidence rather than preference: the polarity research says the
penalty grows as text shrinks, and the text that was smallest was the text the
product most needs to be believed.

## Findings register

Severity: 0 none · 1 cosmetic · 2 minor · 3 major · 4 blocker. Every resolved
finding with behaviour is asserted in `scripts/check-v2-evaluation-fixes.mjs`.

| ID | Sev | Where | Finding | Resolution | Status |
| --- | ---: | --- | --- | --- | --- |
| F-01 | 3 | Global | Mono machine-fact layer at 9–11px via 51 `font:` shorthands; caption token at 12px | `--v2-type-caption` and `--v2-type-mono` raised to 13px. New `--v2-type-mono-label` (11px) is used only by 17 short uppercase tags and always pairs with `--v2-tracking-label`, which moved from 0.075em to the documented 0.08em | Resolved |
| F-02 | 3 | Connect | Fabricated `http://<id>.home` pre-filled as a real value | Field starts empty; the guess becomes an `e.g.` placeholder; the address field takes focus | Resolved |
| F-03 | 3 | All forms | Input and select boundaries 1.26–1.51:1 | New `--v2-border-control` (`#7d7068`): 4.13 on canvas, 3.89 raised, 3.43 on hover | Resolved |
| F-04 | 2 | Loading | `aria-label` on role-less skeleton containers | `role="status"` | Resolved |
| F-05 | 2 | Panel | Panel copy in `--v2-text-quiet`, 3.31:1 | `--v2-text-tertiary`, 5.32:1 | Resolved |
| F-06 | 2 | Modals | Panel and Discover drawer let focus leave; the panel could reach the skip link | One Tab trap for every `aria-modal` surface; skip link inert while the panel is open | Resolved |
| F-07 | 2 | Connect | Empty name rejected silently | Visible message: "Give this app a name you will recognise in My Apps." | Resolved |
| F-08 | 2 | Connect | Global connect landed with Memos selected | Existing catalog apps are selected; otherwise a banner names what was saved and states the production behaviour (select the new row) | Resolved — production requirement recorded |
| F-09 | 2 | Install | Port retry pre-filled with the failing port; review forgot the conflict | The refused port is remembered, marked invalid with "Port 5230 was refused", and an unchanged retry is refused | Resolved |
| F-10 | 2 | My Apps | `error` shared warning colour with `stopped` | `error` maps to danger | Resolved |
| F-11 | 2 | My Apps | Two ember-filled actions | Header "Discover" is ghost | Resolved |
| F-12 | 2 | Settings | "Scan again" wrapped inside its pill | `.button` never wraps | Resolved |
| F-13 | 2 | Global | Ctrl K search revealed as a concept only on use | "Concept" tag inside the control, also in its accessible name | Resolved |
| F-14 | 2 | My Apps | Three levels of bordered containers | Facts and paths are divided, not boxed; only the status card is lifted | Resolved |
| F-15 | 1 | Global | Three names for one capability | "Reviewed install" in the filter and the recipe badge. "Review install" stays as the button verb | Resolved |
| F-16 | 1 | My Apps | Internal field labels | "Runs as", "From catalog", "Last changed" | Resolved |
| F-17 | 1 | Install | `localhost` and `127.0.0.1` on one screen | **Withdrawn.** Both are real and distinct: every recipe publishes on `127.0.0.1:PORT` (the compose file in `src/recipes/*.json`) and launches at `http://localhost:PORT` (`launch_url`). The port control shows the bind address; the summary shows where the window opens. A correction was briefly applied and reverted — see "What went wrong" | Withdrawn |
| F-18 | 1 | First Run | "Docker Doctor" | "Checking Docker and Compose while you read." | Resolved |
| F-19 | 1 | Activity | Design-process copy | "Every install, start, stop and failure, newest first." | Resolved |
| F-20 | 1 | Focused tasks | Rail lost "you are here" in Install and Recovery | Parent marked with `aria-current="location"`; breadcrumb reads "Discover / Install" and "Settings / Review recovery" | Resolved |
| F-21 | 1 | Global | Eyebrows that restate the navigation | Deferred to Phase 16 copy rules: removing them changes the header rhythm on every screen, which is a copy-system decision rather than an evaluation fix | Deferred — Phase 16 |
| F-22 | 1 | Prototype | Truth labels look like product badges | Deferred to the Phase 16 handoff, which must list every prototype annotation so implementation removes all of them | Deferred — Phase 16 |
| F-23 | 1 | First Run | Promise rows crowded the actions | Root cause was F-26 | Resolved |
| F-24 | 1 | My Apps | Row subtitle repeated the status badge | Subtitle carries the app type only | Resolved |
| F-25 | 2 | Rail | The Docker status card showed a chevron but was a `div` with no behaviour | Now a link to Settings, with hover and focus states | Resolved — found during correction |
| F-26 | 2 | System | Six declarations used `--v2-space-7`, which did not exist. An undefined custom property voids its whole declaration, so each of those margins silently rendered as 0 | `--v2-space-7: 28px` completes the 4px scale. The gate now fails on any undefined `var(--v2-*)` | Resolved — found during correction |

No blocker was found. No severity ≥2 finding remains open.

## Token drift — documented exceptions

`docs/design/v2/evaluation-exceptions.json` is the gate's allow-list. It holds five
values, each deliberate:

| Value | Where | Why it is not a token |
| --- | --- | --- |
| `border-radius: 50%` | Status dots, spinners, orbit rings | True circles: a proportion of the element, not a step on a scale |
| `1px` | Divider grids (`gap: 1px`) | The gap *is* the hairline; the grid background shows through it |
| `2px`, `3px` | Optical nudges under titles and icons | Baseline corrections below the 4px grid, by design |
| `54px` | `.activity-event-detail` left margin | Aligns detail text under the event title: a 38px icon plus `--v2-space-4` |

Everything else moved into tokens. The corrections added:

| Token | Value | Replaces |
| --- | --- | --- |
| `--v2-radius-xs` | 4px | 3, 4 and 5px on log skeletons and key caps |
| `--v2-radius-icon-sm/md/lg` | 10 / 13 / 18px | Nine proportional icon-container radii. Icon boxes use roughly 0.3 × their size; the scale quantises that to three steps |
| `--v2-type-heading-lg` | 30px | 30 and 31px focused-task result headings |
| `--v2-space-7` | 28px | Six void declarations (F-26) |
| `--v2-field-icon-inset` | 42px | Identical text-clearance padding in search fields and selects |
| `--v2-material-ember-1…6`, `--v2-on-material`, `--v2-on-material-strong` | — | The approved grain material and the cream text on it. Eight near-identical creams collapsed to two |
| `--v2-surface-log`, `--v2-text-log` | `#181512`, `#d8d0c9` | Log viewer, 12.1:1 |
| `--v2-surface-accent-selected` | `#32120b` | Selected starter card |
| `--v2-border-control`, `--v2-neutral-450` | `#7d7068` | Form boundaries (F-03) |
| `--v2-type-mono-label` | 11px | Short letter-spaced tags only (F-01) |

## Regressions introduced by the corrections, and fixed

The 13px floor made three values truncate at 1280×800, caught in the visual pass:
"Actual Budget" in the saved-apps list, "Engine 29.3.1" in the rail's Docker card,
and the version and address on the Discover recipe cards. Each was fixed spatially
rather than by shrinking type again: the saved-apps list keeps a 340px width at the
narrow breakpoint instead of dropping to 310px, the rail card tightens its own
padding below 1320px, and the recipe cards put version and address on separate
lines. The behavioural suite asserts that nothing in the list or rail card
truncates at 1280.

## What went wrong, for the record

- **F-17 was a wrong finding.** It read `localhost` and `127.0.0.1` on one screen as
  an inconsistency and replaced six `localhost` references without checking the
  backend, which uses both for different things. The change was reverted from a
  pre-work snapshot, and a regression suite that had correctly expected
  `localhost:5678` was restored. Lesson: verify any copy that states a machine
  fact against `src/` before correcting it.
- **The first focus probe produced 18 false positives** by calling `.focus()`, which
  does not reliably trigger `:focus-visible`. A real keyboard walk replaced it.
- **The Playwright MCP browser reported a keyboard bug that did not exist.** A clean
  Playwright run showed no navigation on Tab.
