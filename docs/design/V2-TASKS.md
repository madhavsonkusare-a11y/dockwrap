# Local Store V2 design tasks

Status: **in progress — phases 1–15 complete**  
Scope: complete the laptop-first V2 visual design, interactive prototype,
evaluation, and implementation handoff. Production integration under `src/`
starts only after this ledger is complete and approved.

## Definition of done

V2 design is complete when:

- One coherent, interactive HTML prototype covers the primary product flows.
- The prototype is designed and verified at 1280×800 and 1440×900.
- Overview, Discover, My Apps, Install, Activity, First Run, Settings, and
  Recovery have approved default, empty, loading, success, and relevant failure
  states.
- Every app shown in the prototype has a correct icon; the production icon
  import/fallback plan can guarantee an icon for every catalog entry.
- The visual system, grain treatment, typography, motion, and component rules
  are documented and internally consistent.
- Real backend capabilities and speculative future concepts are clearly
  distinguished in the prototype and handoff.
- The evaluation toolkit gates pass, or remaining exceptions are explicitly
  documented and approved.
- The final design can be implemented without inventing missing layouts,
  states, or interaction rules.

## Phase 0 — Preserve context and establish constraints

- [ ] Confirm laptop-only scope for V2; do not design mobile layouts.
- [ ] Set the minimum supported design viewport to 1280×800.
- [ ] Set the primary design viewport to 1440×900.
- [ ] Record Windows as the current shipping platform.
- [ ] Treat the existing Ember work as exploration, not a pixel-perfect mandate.
- [ ] Keep all V2 work isolated under `docs/design/v2/` until approval.
- [ ] Do not modify the production frontend under `src/` during the design phase.
- [ ] Add a visible prototype label for data or features that do not yet exist.
- [ ] Reconcile outdated statements in `docs/design/README.md`, including the
  already-implemented editable port support.

## Phase 1 — Product and backend capability inventory

- [x] Build a feature matrix covering backend support, current UI exposure,
  proposed V2 exposure, and missing backend work.
- [x] Inventory installed-app fields: identity, catalog link, URL, icon, runtime
  type, creation time, and update time.
- [x] Inventory runtime states: running, stopped, error, ready, unreachable, and
  unknown/not-yet-checked.
- [x] Inventory Docker Doctor results and Docker/Compose version details.
- [x] Inventory install stages, operation IDs, cancellation checkpoints, and the
  uncancellable commit boundary.
- [x] Inventory recipe details: image, version, port, storage, health check,
  restart behavior, requirements, setup fields, and risk notes.
- [x] Inventory lifecycle actions: open, start, stop, logs, shortcut, remove,
  uninstall, preserve data, and delete data.
- [x] Inventory recovery candidates, ownership verification, preserved data,
  and available recovery actions.
- [x] Inventory catalog data: description, category, tags, capability, license,
  architecture, warnings, maintenance state, provenance, and source dates.
- [x] Inventory connection address validation and readiness behavior.
- [x] Identify attractive future concepts that need new backend data, such as
  resource usage, uptime history, storage totals, and an activity ledger.
- [x] Mark each prototype feature as `available`, `derivable`, or `concept`.

## Phase 2 — Information architecture

- [x] Decide the final top-level navigation: Overview, Discover, My Apps,
  Activity, and Settings placement.
- [x] Define what belongs on Overview versus My Apps.
- [x] Define the relationship between catalog details, Connect, and Install.
- [x] Define managed-app and linked-app mental models and terminology.
- [x] Define the master/detail behavior for My Apps.
- [x] Define where logs appear: detail panel, drawer, or dedicated view.
- [x] Define where recovery appears and when it becomes prominent.
- [x] Define First Run entry, completion, skip, and later re-entry behavior.
- [x] Define global search scope and keyboard behavior.
- [x] Run a Krug trunk test on the proposed information architecture.

## Phase 3 — Visual system

- [x] Finalize warm neutral primitives and semantic color tokens.
- [x] Finalize ember accent, hover, pressed, focus, success, warning, and error
  colors with measured contrast.
- [x] Ensure all orange-filled controls use a dark foreground with AA contrast.
- [x] Finalize Instrument Sans usage and bundle it locally for offline preview.
- [x] Finalize IBM Plex Mono usage for machine-asserted facts only.
- [x] Define the complete laptop type scale and minimum text sizes.
- [x] Define spacing, grid, content width, navigation width, and detail-panel width.
- [x] Reduce card and control radii to a small, intentional token set.
- [x] Define borders, elevation, overlays, and surface hierarchy.
- [x] Define icon container shapes without forcing every logo into the same card.
- [x] Define density rules for browse surfaces versus trust/confirmation surfaces.
- [x] Define when grain and ember blooms are allowed and when they are prohibited.
- [x] Produce a component/token specimen page.
- [x] Run the anti-generic/fingerprint audit and revise the system.
- [x] Run the token-consistency audit and remove unexplained raw values.

## Phase 4 — Brand mark and grain treatment

- [x] Create at least three grain treatments for the orange square in the logo.
- [x] Keep the cream/white part of the mark crisp and visually dominant.
- [x] Compare flat, subtle-grain, and stronger ember-depth variants.
- [x] Test the mark on base, raised, orange, and light surfaces.
- [x] Test at 16, 24, 32, 48, 64, and 128 pixels.
- [x] Create a simplified small-size variant if grain becomes muddy.
- [x] Verify deterministic offline rendering in the Tauri webview.
- [x] Check SVG filter performance and prepare a baked-texture alternative.
- [x] Select and document the approved logo treatment.

## Phase 5 — Catalog icon coverage

- [x] Measure current catalog coverage and list every entry with a missing icon.
- [x] Create a normalized mapping strategy using catalog IDs and aliases.
- [x] Audit the official Umbrel App Store repository for matching applications.
- [x] Record the exact pinned Umbrel revision used by the importer.
- [x] Review Umbrel asset provenance and applicable licensing/trademark notices.
- [x] Import matching Umbrel icons through a reproducible script, not manually.
- [x] Preserve original aspect ratio and avoid destructive cropping.
- [x] Prefer vector assets when trustworthy; validate and sanitize SVG content.
- [x] Enforce file-size, dimensions, MIME, extension, and checksum limits.
- [x] Fall back to existing source-catalog or verified upstream icons where
  Umbrel has no match.
- [x] Generate a branded deterministic monogram for any remaining entry.
- [x] Produce an icon provenance manifest with source URL, revision, checksum,
  and license/notice fields.
- [x] Detect duplicate, suspicious, blank, corrupt, and incorrectly matched icons.
- [x] Render an icon contact sheet for human review.
- [x] Guarantee that every catalog entry resolves to an icon or approved fallback.
- [x] Add the final icon pipeline requirements to the implementation handoff.

## Phase 6 — Interactive prototype foundation

- [x] Create `docs/design/v2/index.html` as the prototype entry point.
- [x] Create a shared V2 token/component stylesheet.
- [x] Create a small prototype controller with no framework or build step.
- [x] Create realistic fixture data derived from current backend contracts.
- [x] Label speculative data in fixtures separately from real contract fields.
- [x] Support navigation between all primary screens without page reloads.
- [x] Add a prototype control panel for jumping to screens and states.
- [x] Add toggles for loading, empty, busy, success, and failure conditions.
- [x] Keep the prototype fully usable offline, including fonts and icons.
- [x] Ensure semantic HTML and keyboard operation from the first iteration.

## Phase 7 — Overview design

- [x] Establish the screen's primary question and primary action.
- [x] Design a normal state with mixed managed and linked apps.
- [x] Design the all-healthy state without excessive status decoration.
- [x] Design a needs-attention state for stopped, unreachable, or recoverable apps.
- [x] Design the no-apps state and route users to Connect, Install, or Discover.
- [x] Show Docker/Compose readiness using real Doctor concepts.
- [x] Show recent operations using real or clearly labeled conceptual data.
- [x] Explore resource/storage summaries as labeled future concepts.
- [x] Ensure the page remains useful when conceptual telemetry is removed.
- [x] Run a five-second test on Overview.

## Phase 8 — Discover design

- [x] Design a catalog view that makes six or more projects scannable at once.
- [x] Give every displayed app a correct icon.
- [x] Make Install Preview, Connect, and Explore Project visually distinct.
- [x] Integrate search, capability filters, categories, licenses, architectures,
  and maintenance warnings without producing toolbar clutter.
- [x] Decide whether featured recipes need a hero, compact strip, or pinned cards.
- [x] Use the grainy ember treatment once as a signature, not on every card.
- [x] Design catalog loading, no-results, error, and pagination states.
- [x] Design project detail with provenance and honest capability language.
- [x] Design the Connect path from a catalog entry and from a global action.
- [x] Run a timed find-the-app test.
- [x] Run a five-second test on Discover.

## Phase 9 — My Apps design

- [x] Design the default master/detail layout for 1440×900.
- [x] Verify the layout remains usable at 1280×800.
- [x] Visually distinguish managed Compose apps from linked external apps.
- [x] Design running, stopped, busy, unreachable, and error rows.
- [x] Show URL, runtime type, created/updated dates, and catalog association where
  useful rather than exposing fields indiscriminately.
- [x] Design detail actions for Open, Start, Stop, Logs, Shortcut, and management.
- [x] Design the logs surface with readable mono typography and empty/error states.
- [x] Design uninstall-with-data-preserved as the safe default.
- [x] Design the irreversible delete-data confirmation.
- [x] Design list loading, empty, refresh-failed, and filtered-empty states.
- [x] Compare dark-raised and warm-light detail panels and select one.
- [x] Verify keyboard selection and focus movement in the master/detail layout.

## Phase 10 — Install design

- [x] Design recipe review before any machine change occurs.
- [x] Show exact image/version, published port, data location, restart policy,
  health check, and rollback behavior.
- [x] Design inline port editing and validation.
- [x] Design optional setup fields and secret-field handling.
- [x] Show Docker and Compose checks without inventing additional checks.
- [x] Design all six real installation stages in backend order.
- [x] Design immediate feedback for the first 100ms after Install is pressed.
- [x] Show Cancel only while cancellation is truthful.
- [x] Design successful completion and the transition into My Apps.
- [x] Design Docker-missing, port-in-use, validation, health-check, cleanup, and
  unsafe-retry failure paths.
- [x] Preserve user selections and valid inputs after recoverable failures.
- [x] Run the complete UXBench-style interaction critique.
- [x] Run a cognitive walkthrough of every install branch.

## Phase 11 — First Run design

- [x] Define the product promise in one immediately understandable sentence.
- [x] Run preflight while the welcome content is being read.
- [x] Keep passing preflight invisible as a separate step.
- [x] Design the Docker-missing branch with a clear recovery loop.
- [x] Offer Install, Connect, and Browse without making all three equal-weight CTAs.
- [x] Design recipe selection and editable port configuration.
- [x] Design Skip and later re-entry behavior.
- [x] Design completion with a clear next useful action.
- [x] Run a five-second test on the welcome screen.
- [x] Run a cognitive walkthrough of the entire first-run flow.

## Phase 12 — Activity, Settings, and Recovery

- [x] Define a useful Activity model and clearly mark fields requiring backend work.
- [x] Avoid charts that do not answer an operational question.
- [x] Design operation/event filtering and an empty state.
- [x] Design Settings around real controls rather than descriptive filler.
- [x] Design Docker diagnostics and readable Doctor results.
- [x] Design recovery scanning, candidate verification, safe cleanup, and errors.
- [x] Make destructive data deletion visually and verbally distinct.
- [x] Define About/build/catalog information placement.
- [x] Verify these secondary surfaces use the same component language as the core.
- [x] Reintroduce the approved grainy ember material as a selective signature surface.
- [x] Add a useful memory graph study without presenting unavailable telemetry as live data.
- [x] Route all three editorial recipe cards into their exact install reviews.

## Phase 13 — Interaction and motion specification

- [x] Inventory every transition and decide whether it should animate at all.
- [x] Keep keyboard-triggered and frequently repeated actions effectively instant.
- [x] Add 100–160ms press feedback to pressable controls.
- [x] Use short ease-out transitions for dialogs, panels, and contextual surfaces.
- [x] Animate only transform and opacity unless a documented exception is needed.
- [x] Make panel and popover origins spatially coherent with their triggers.
- [x] Define progress motion that communicates state without implying false percent.
- [x] Ensure every operation provides visible feedback within 100ms.
- [x] Implement and verify reduced-motion behavior.
- [x] Document durations, easing curves, interruption, and exit behavior.

## Phase 14 — Evaluation toolkit gates

- [x] Run Nielsen's ten heuristics against every primary screen.
- [x] Assign severity 0–4 and a concrete correction to every finding.
- [x] Run the Krug trunk test on every navigable screen.
- [x] Run five-second tests on Overview, Discover, and First Run.
- [x] Run cognitive walkthroughs on First Run, Connect, Install, and Recovery.
- [x] Drive interactive flows and report missing feedback, silent validation,
  hidden consequences, weak recovery, and state lost on Back.
- [x] Run the anti-generic aesthetic audit.
- [x] Run a three-perspective design critique and synthesize disagreements.
- [x] Run automated axe checks against every prototype state.
- [x] Verify text and non-text contrast programmatically and visually.
- [x] Run a token/consistency audit for color, spacing, radius, and type drift.
- [x] Record findings and resolutions in `docs/design/v2/evaluation.md`.

Phase 14 outcome: all gates pass across 88 states × 2 viewports (axe, keyboard
focus, input boundaries, 13px legibility, token drift, undefined tokens). 26
findings: 23 resolved, 1 withdrawn, 2 cosmetic deferred to Phase 16 below. Run
`scripts/check-v2-evaluation-gates.mjs` and `scripts/check-v2-evaluation-fixes.mjs`.

## Phase 15 — Laptop visual verification

- [x] Capture deterministic screenshots at 1280×800.
- [x] Capture deterministic screenshots at 1440×900.
- [x] Check every screen for clipping, overflow, awkward empty space, and obscured
  primary actions.
- [x] Verify long app names, long URLs, long image tags, and long error messages.
- [x] Verify catalog density with real descriptions and mixed icon aspect ratios.
- [x] Verify dialogs and panels fit at 1280×800 without hiding irreversible actions.
- [x] Verify focus rings, hover, active, selected, disabled, and busy states.
- [x] Review the prototype at 100%, 125%, and 150% Windows display scaling.
- [x] Produce a final screenshot/contact sheet for approval.

Phase 15 outcome: 88 states × 2 viewports captured byte-deterministically, plus the
same matrix with worst-case real content, 4 display-scaling panels, and 57
interaction-state checks. Every hard check passes (overflow, unreadable truncation,
glyph clipping, overlap and logo spill, covered primaries, unfit dialogs,
determinism, states). 16 findings: 15 resolved, and V-16, the launcher window size
versus the design range, carried into Phase 16 below. Record:
`docs/design/v2/visual-verification.md`. Approval set: `docs/design/v2/screenshots/phase15/`.
Run `scripts/check-v2-visual.mjs`, `scripts/check-v2-visual-fixes.mjs`, and
`scripts/check-v2-real-windows.mjs`.

## Phase 16 — Final design handoff

- [ ] Freeze the approved V2 tokens.
- [ ] Freeze the approved navigation and screen inventory.
- [ ] Freeze component anatomy and interaction-state specifications.
- [ ] Document copy rules, terminology, and machine-fact formatting.
  Includes deferred F-21: keep screen eyebrows only where they carry state.
  Includes Phase 15: give the catalog's "See project" licence placeholder a product
  phrase (for example "Licence not stated"); it reads as a fact but isn't one.
- [ ] Document real versus future backend dependencies per component.
  Includes deferred F-22: list every prototype annotation (truth labels, the
  search "Concept" tag, backend-work banners) so implementation removes all of them.
- [ ] Document icon import and attribution requirements.
  Includes Phase 15: prefer a square symbol variant for wordmark-only logos wider
  than 3:1 (Appsmith, Neon, Martin), which are legible but small at 44px.
- [ ] Document production asset requirements, including bundled fonts and logo files.
- [ ] Map prototype components to current `src/index.html`, CSS, and JS modules.
- [ ] Produce an implementation sequence that preserves working functionality.
- [ ] Reconcile the launcher window with the design range (Phase 15 V-16).
  `src/main.rs` opens at 1180×760 with a minimum of 800×600, and a maximised
  1920×1080 laptop at 150% gives 1280×640. Decide the default and minimum size, and
  pin focused-task commits in a sticky footer below 760px of height.
- [ ] Identify which existing Playwright baselines will be replaced.
- [ ] Define production acceptance tests before implementation starts.
- [ ] Obtain explicit approval of the final prototype and handoff.

## V2 design completion record

- [ ] All phases above are complete or have explicitly approved exceptions.
- [ ] Final prototype approved at 1280×800.
- [ ] Final prototype approved at 1440×900.
- [ ] Evaluation findings resolved or accepted.
- [ ] Icon coverage and provenance plan approved.
- [ ] Production implementation plan approved.
- [ ] V2 design marked complete with completion date and approver.
