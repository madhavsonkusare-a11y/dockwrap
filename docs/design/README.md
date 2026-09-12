# Local Store — visual design handoff

Status: **proposal.** Nothing here is wired into the app. No file under `src/` was
modified to produce it. These are self-contained HTML mockups plus the reasoning
and tokens behind them, so another agent can pick the work up cold.

Everything opens in a browser with no build step and no network:

```bash
start docs/design/ember-screens.html          # Windows
xdg-open docs/design/ember-screens.html       # Linux
```

## What is here

| File | What it is |
|---|---|
| `ember-screens.html` | Six app screens at 1280×800: Overview, Discover, My Apps, Install review, Activity, First run. Static mockups. |
| `setup-flow.html` | The setup wizard as a **clickable prototype**. Walk it end to end, or use the "Jump" chips. Two failure paths are toggleable. |
| `ux-evaluation-toolkit.html` | 22 prompts, agent skills, runnable tools and benchmarks for evaluating UI/UX, ranked for this repo. Read this before reviewing the design. |
| `tokens.css` | The design tokens as a real stylesheet — primitive → semantic → component, with measured contrast in the comments. |

The mockups are single files. App logos are inlined as data URIs, extracted from
this repo's own `src/assets/catalog/`, so they render offline. The grainy orange
gradients are generated on a `<canvas>` at runtime, not image assets.

## The direction, in one paragraph

Warm near-black surfaces, a single saturated ember orange spent once per screen,
pill-shaped controls, generous type, real app logos, and grainy orange gradient
blooms as the signature. Type is Instrument Sans with IBM Plex Mono reserved for
anything the machine asserts — image tags, ports, paths, digests, uptimes, exit
codes. That mono layer is deliberate: it makes verifiable facts *look* verifiable
on the screen that has to be believed.

## Why it looks like this

The direction was chosen against evidence, not taste. Full write-ups are linked
below; the short version:

- **The audience is hobbyist engineers, not consumers.** 2025 self-hosting surveys:
  31.3% self-host for fun/hobby; privacy + independence + data control + learning
  total 58.3%; cost is last at 7.2%. 98.3% use containers, 87.6% use Compose.
  They are not shopping — they are building, and they want the mechanism visible.
- **Prototypicality is category-relative.** The comparison set in a self-hoster's
  head is Proxmox, Portainer, Grafana, Unraid — not the App Store. The shipped
  build is a competent consumer app store, which makes it prototypical of a
  category its users are not thinking about.
- **Visual design is a credibility argument.** In Fogg's Stanford study (2,684
  participants), "design look" was the most-cited credibility factor at 46.1%.
  Local Store asks permission to run containers on someone's machine, so the
  install screen is a trust surface and stays the most conventional screen in the
  product on purpose.
- **Complexity is not one thing.** Text-based complexity correlates *positively*
  with trust; image-based complexity correlates negatively. Keep the data density,
  keep decoration to a signature.
- **Dark-only has a cost.** Dark text on light beats light on dark for acuity and
  proofreading, and the gap widens as type shrinks. Dark-only is a fine product
  decision — it just means type must be **larger and heavier**, not smaller. The
  shipped build puts 75% of its `font-size` declarations at ≤12px.

An earlier, much more brutalist direction was scored against six candidates on
seven weighted criteria and lost: it ties with the current build at 2.85/5, from
the opposite direction. The chosen direction scored 4.50.

## Diagnosis of the shipped UI (evidence, not opinion)

Measured from `src/styles/app.css` at the time of writing:

| Finding | Evidence |
|---|---|
| The orange is declared but never used | `--signal: #ff623e` is defined and applied to nothing; the UI paints `--accent: #ffac92`, the same hue lifted to **79% lightness** |
| Radius is decorative, not structural | **11 distinct values** across 28 declarations: 4, 5, 9, 10, 12, 14, 15, 16, 19, 20, 23px |
| Every surface is the same object | 11 gradients, all `linear-gradient(145deg, …)` plus an inset white highlight, on cards, rows, dialogs and avatars alike |
| Type never gets loud or quiet | 52 of 69 `font-size` declarations are ≤12px; the largest is a single 33px heading |
| Machine facts wear marketing clothes | `.recipe-summary` renders image, port and volume in Inter at 11px — same face and size as the shelf caption "Catch a thought" |

## Token mapping

`tokens.css` is the full set. The names below already exist in
`src/styles/app.css`, which is most of why this is cheap to try:

| Token | Current | Proposed | Effect |
|---|---|---|---|
| `--canvas` | `#111214` | `#0C0A09` | Ground drops below panels so hairlines register |
| `--surface` | `#1b1d20` | `#161211` | Warm-biased neutral instead of blue-grey |
| `--border` | `#303236` | `#FFFFFF0F` | Hairline becomes structural |
| `--accent` | `#ffac92` | `#F26419` | The single largest visible change |
| `--signal` | `#ff623e` *(unused)* | `#FF8A3C` | Promoted to the secondary step, actually used |
| `--ink` / `--muted` | `#f8f7f3` / `#a3a5ac` | `#F7F1E9` / `#8B7F76` | Neutrals biased warm to sit with the accent |
| `--success` | `#9ad8b6` | `#62C79A` | Running state readable at a glance, 12.87:1 |
| `border-radius` | 4 … 23px | `20px` cards, `999px` controls | Eleven values collapse to two |
| fonts | Inter only | + Instrument Sans, IBM Plex Mono | Two woff2 files into `src/fonts/` |

**Contrast rule that falls out of the numbers:** near-black on `#F26419` measures
6.41:1; white on the same orange measures 3.09:1 and fails AA for body text. Every
orange fill takes `#2A0F03` text, never white.

## The setup flow

`setup-flow.html` is interactive. Four steps — Welcome, Choose, Install, Ready —
and a fifth that only appears when Docker is missing.

It is built against the **real backend**, not invented:

- The preflight runs the two checks `doctor_with()` actually runs in
  `src/runtime/mod.rs`: `docker version --format {{.Server.Version}}` and
  `docker compose version --short`. There is no third check.
- The install step shows the six stages the backend emits, in order, with the
  labels from `src/js/operations.js`: `checking_system`, `preparing_files`,
  `validating_recipe`, `starting_containers`, `waiting_for_health`, `saving_app`.
- Cancel disappears at `saving_app`, because `UNCANCELLABLE_STAGES` in that same
  file means the backend refuses a cancel from there. Offering one would be a lie.
- The port-conflict failure uses the real `port_in_use` code and its real hint
  string, and rolls back before reporting — which is what the backend does.
- The v1 → v2 registry migration notice reflects `load_or_migrate_registry_at()`
  in `src/storage.rs`, including the `migration-v1-backup.json` backup.

Two toggles under the frame demo the failure paths: **Docker missing** and
**Port in use**.

### Backend changes it would need

Reconciled on September 12, 2026 against the shipped launcher.

1. ~~**Editable port before install.**~~ **Already implemented.** `install_app` takes
   a host port and `Recipe::with_host_port` applies it; `tests/ui/operations.spec.js`
   covers choosing a port, the pinned default, and refusing an unusable one. The flow
   here matches what ships.
2. **A first-run flag.** Still missing. Nothing records that setup has been completed,
   so First Run cannot open by itself.
3. **Preflight on launch.** Still missing as a flow. `doctor` exists and is reachable
   from Settings; running it on the welcome screen is a wiring change, not a new
   command.

The V2 design supersedes these mockups. Its full backend dependency list is
`docs/design/v2/BACKEND-DEPENDENCIES.md`.

## Known gaps

- **Not implemented.** No `src/` file was changed. These are mockups.
- **Screenshot baselines will break.** `tests/ui/*-snapshots/` compares at
  1280×800, 800×600 and 400×860 with a 1% diff ratio. Adopting any of this means
  regenerating every baseline in the same commit. Under the approved V2 design the
  800×600 and 400×860 baselines are retired: nothing below 1180px wide is designed
  (`docs/design/v2/HANDOFF.md`).
- **No user testing.** The style scoring is a weighted decision model applied
  consistently, not measurement. `ux-evaluation-toolkit.html` includes a
  three-test protocol (50 ms first impression, install-screen credibility, timed
  find-the-app) that would replace judgment with data in about a week.
- **Logo data is duplicated** between the two mockups, ~42KB each. Fine for a
  handoff; deduplicate if these ever ship.
- Memos ships only a raster logo in this repo, so it is a downscaled PNG while
  every other logo is vector.

## Picking this up

Read in this order:

1. `ux-evaluation-toolkit.html` — how to evaluate what you are about to change
2. This file — the direction and the constraints
3. `setup-flow.html` — click it; the flow decisions are not visible in a screenshot
4. `ember-screens.html` — the surface language
5. `tokens.css` — the values

Then the cheapest real first step is the toolkit's 30-minute version: install
Playwright MCP, add a design-review subagent pointed at `node scripts/preview.mjs`
on `127.0.0.1:4173`, and put the anti-generic audit prompt in `CLAUDE.md` so every
future UI change is checked for the failure this whole exercise started from.

## Superseded, kept for context

Two earlier directions were explored and rejected. They are not in this folder;
the reasoning that killed them is summarised above.

- **Signal** — minimalist brutalism with ordered Bayer dithering. Too atypical for
  a trust-critical screen.
- **Ember I** — the first pass at this direction. Superseded by `ember-screens.html`
  (finer grain, linear-light gradient interpolation, real logos, warmer neutrals).
