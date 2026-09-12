# V2 Discover design

Status: **Phase 8 complete**  
Primary viewport: 1440×900  
Minimum viewport: 1280×800

## Product job

Discover answers **what can I run, connect, or learn about?** It is a dense
offline catalog, not a promotional app store. Six representative projects are
visible together at both target widths, with identity, category, short
description, license, architecture, and capability action remaining scannable.

Capability language is fixed and deliberately unequal:

- **Review install** uses the ember primary control only when a reviewed Local
  Store recipe exists.
- **Connect** uses a neutral contained control when a web UI can be linked but
  the server is not managed.
- **Explore** uses a quiet outlined action with an external-link mark when Local
  Store only provides discovery facts.

## Featured recipe decision

Reviewed recipes use one compact signature strip above the catalog. A large
hero would push the second card row below the fold; pinned cards would repeat
capability information already present in the grid. The strip is the only
Discover surface that uses the grainy Local Store mark and ember-depth bloom.
Catalog cards remain neutral so project artwork stays dominant.

## Search and filters

The primary toolbar contains search, four capability tabs, and one advanced
Filters trigger. Category, license, architecture, and maintenance-warning
controls live in an origin-aware popover instead of expanding the toolbar.
Filter combinations update the visible result count and produce an inline
no-results recovery state without losing the query.

Paging is bounded and explicit: the default shows the visible range, total,
and page number; the busy fixture retains all current results and disables the
next-page action while loading. No endless-scroll behavior is implied.

## Project and Connect drawers

A right contextual drawer preserves the catalog behind it. Project detail
shows category, capability, warning language, license, architecture,
maintenance status, source date, catalog provenance, and upstream location.
It exposes exactly one primary capability action, with the reviewed-install
exception allowing a secondary “connect a running app” route.

Connect can start globally or from a catalog entry. A catalog entry prefills
the name and a sample local address. The form states before submission that a
linked app receives a shortcut and advisory readiness check, while Local Store
does not install, start, stop, update, or back up its server. Invalid addresses
retain both fields, receive inline feedback, and focus the address field.

## State model

| Prototype condition | Designed meaning | Recovery |
| --- | --- | --- |
| Default | Six mixed-capability projects and page controls | Search, filter, inspect, connect, or review install |
| Loading | Catalog index is not yet available | Stable catalog skeleton |
| Empty | A real query/filter combination has no matches | Clear search and filters |
| Busy | Next bounded page is loading | Current six results remain usable |
| Success | Local catalog snapshot loaded successfully | Continue browsing |
| Failure | Local snapshot could not be read | Retry locally or inspect snapshot details in Settings |

The failure state explicitly says saved apps are unaffected and that there is
no network fallback.

## Backend truth boundaries

All default project facts map to current catalog fields: stable ID, name,
description, category, tags, licenses, architectures, source URLs, upstream
URL, updated date, warning/maintenance state, and web-UI capability. Capability
labels are derived from the current `preview_install`, `connect`, and
`discover` model. Ratings, popularity, download counts, and reviews are absent
because no backend source exists for them.

## Design review

| Before | After | Why |
| --- | --- | --- |
| Action meaning appeared as a footer badge | Install, Connect, and Explore have explicit action controls | Capability is understandable without guessing badge semantics |
| Three decorative filter chips | Search, capability tabs, and working advanced facets | Real backend filters are exposed without toolbar clutter |
| Entire cards acted as invisible buttons | Cards contain explicit Details and capability actions | Multiple outcomes remain keyboard- and screen-reader-readable |
| Project detail produced a toast | Contextual drawer preserves catalog state and shows provenance | Trust evidence needs depth without losing browse position |
| Connect was a placeholder message | Prefilled form, advisory check, validation, consequence copy, and success route | The complete supported path can now be evaluated |
| No maintenance caution in the grid | Warning mark plus full drawer explanation | Risk is discoverable without turning every card into an alert |
| No pagination contract | Visible range, page number, bounded next action, and busy state | The UI matches backend paging and avoids invented infinite scroll |
| Grain could spread across cards | One branded reviewed-recipe strip | The signature stays recognizable and project icons remain primary |

## Evaluation-toolkit checks

### Timed find-the-app test

The scripted task “find Memos and identify how to install it” started on the
default 1440×900 frame. Typing `memos` reduced the six rendered fixtures to the
correct card in **0.20 ms of in-page filter work**. The visible primary action
was `Review install`; opening it routes to the dedicated install review. This
measures prototype response and control clarity, not user reading speed.

### Five-second test

The default capture was evaluated with the toolkit's recall questions:

1. **What does this screen do?** Browse an offline catalog of self-hosted
   projects.
2. **What distinctions matter?** Reviewed install, connect an existing web UI,
   or explore the upstream project.
3. **What is the primary next action?** Search the catalog or review the
   featured Memos setup.

All answers are present in the first viewport at 1280×800 and 1440×900. A
representative-user five-second test remains part of the final Phase 14 gate.

### Automated and interaction checks

- Nine surfaces—six conditions, filters, project detail, and Connect—pass
  `wcag2aa` and `wcag22aa` axe scans inside the prototype screen at both target
  viewports.
- Default, empty, error, pagination-busy, filter, project-detail, and Connect
  screenshots were inspected for clipping and hierarchy.
- Search, quick filters, combined advanced filters, inline no-results recovery,
  drawer Escape/return behavior, invalid-address preservation, advisory address
  feedback, and the Connect success route were driven end to end.
- Every displayed fixture resolves to its local catalog icon.

## Deterministic preview URLs

- `index.html#discover` — default catalog
- `index.html?state=empty#discover` — no results
- `index.html?state=busy#discover` — pagination busy
- `index.html?state=failure#discover` — catalog read failure
- `index.html?filters=1#discover` — advanced filters open
- `index.html?project=nextcloud&drawer=detail#discover` — warning detail
- `index.html?project=immich&drawer=connect#discover` — prefilled Connect
