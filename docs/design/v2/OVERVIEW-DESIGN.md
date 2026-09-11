# V2 Overview design

Status: **Phase 7 complete**  
Primary viewport: 1440×900  
Minimum viewport: 1280×800

## Product job

Overview answers one question: **does anything in my local workspace need me?**

When attention exists, the primary action is **Review _n_ items** and routes to
My Apps. When everything is healthy, the primary action becomes **Open My
Apps**. Adding another app remains secondary. Routine lifecycle controls stay
in My Apps so Overview does not become a duplicate app list.

## State model

| Prototype condition | Designed meaning | Primary response |
| --- | --- | --- |
| Default | Mixed managed and linked apps; one stopped and one unreachable | Review the two affected apps |
| Success | Docker is ready and all managed/linked apps are healthy | Open My Apps; no celebratory decoration |
| Busy | Docker Doctor is running while the last useful summary remains visible | Wait or inspect the retained app summary |
| Failure | Docker is unavailable; linked apps remain usable; retained setup is present | View diagnostics or review recovery |
| Empty | No apps or recovery items exist | Connect an app, preview a starter install, or browse |
| Loading | Workspace records and Doctor facts are not available yet | Preserve layout with restrained skeletons |

The default fixture deliberately includes both runtime models. Copy always
names them as **managed app** or **linked app** when that distinction changes
the meaning of a status.

## Backend truth boundaries

| Surface | Truth class | Contract or rule |
| --- | --- | --- |
| Saved apps, runtime type, status, readiness | Available | `list_apps` / `AppView` |
| Docker Engine and Compose results and versions | Available | The two real Doctor checks only |
| Managed, linked, and attention counts | Derivable | Computed from current app records |
| Recent registry changes | Derivable | Creation/update facts; never called “last active” |
| Recent operation rows | Available in the current session | Live `OperationEvent` facts; not presented as durable history |
| Resource, storage, and availability summaries | Concept | Optional design-lab study, off by default and visibly labeled |

The resource study can be enabled with `?concepts=1#overview`. Removing it
does not move or weaken the health question, Doctor result, counts, attention,
or recent activity.

## Design review

| Before | After | Why |
| --- | --- | --- |
| “Workspace is healthy” appeared beside two warnings | “Two apps need a quick look” with Docker readiness stated separately | Environment readiness and app health are different signals |
| Discover was the strongest action | Review attention is primary; Add app is secondary | The action now follows the page's operational question |
| One static layout represented every condition | Calm, checking, Docker-failure, empty, and loading compositions | State changes now alter meaning, not just banner color |
| Four counts mixed runtime and status semantics | Saved, managed, linked, and attention counts use explicit sources | Each number can be explained from current data |
| “Recent changes” could imply durable history | Recent activity labels session operations versus registry changes | The UI does not overstate backend persistence |
| Concept telemetry sat near production facts by implication | Future signals are off by default and carry a Concept Data label | Speculative features cannot be mistaken for available data |
| Empty Overview had one generic catalog CTA | Connect, reviewed starter install, and browse are distinct routes | Each supported way to begin remains discoverable |

## Evaluation-toolkit checks

### Five-second test

The captured default frame was evaluated with the toolkit's three recall
questions, without using implementation notes:

1. **What does this product do?** Keeps locally managed and linked self-hosted
   apps in one workspace and surfaces operational health.
2. **Who is it for?** A self-hoster managing apps on a Windows laptop.
3. **What is the primary action?** Review the two items that need attention.

All three answers are present in the first viewport at both target sizes. This
is a structured design proxy; the Phase 14 gate still calls for validation with
representative users before final approval.

### Automated and visual checks

- `@axe-core/playwright` with `wcag2aa` and `wcag22aa`: zero violations inside
  `#screen` for default, loading, empty, busy, success, and failure states.
- Default state captured and inspected at 1280×800 and 1440×900.
- Success, empty, failure, and optional concept study captured at 1440×900.
- The 1280×800 layout keeps the screen question, primary action, Doctor facts,
  summary, and entry points to both lower panels visible without horizontal
  clipping. Lower rows remain available by ordinary vertical scrolling.
- The failure fixture updates both the page and persistent environment card;
  it never shows “Docker ready” and “Docker unavailable” simultaneously.

## Deterministic preview URLs

- `index.html#overview` — mixed default state
- `index.html?state=success#overview` — all healthy
- `index.html?state=busy#overview` — Doctor checking
- `index.html?state=failure#overview` — Docker and recovery attention
- `index.html?state=empty#overview` — no apps
- `index.html?state=loading#overview` — loading
- `index.html?concepts=1#overview` — optional future-signal study
