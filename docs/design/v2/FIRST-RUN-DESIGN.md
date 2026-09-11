# First Run design — Phase 11

First Run is a short introduction, not a setup gate. It explains the product,
runs the real two-check Docker preflight in parallel, and routes people into the
same Install, Connect, and Discover surfaces used everywhere else.

## Product promise

> Run and reach your self-hosted apps from one calm place.

The sentence names the audience, the job, and the benefit without requiring
Docker vocabulary. Supporting copy explains reviewed installs and linked apps;
the promise does not attempt to carry those implementation details.

## Backend truth used

- Docker Doctor performs exactly two checks: Docker Engine and Docker Compose.
- Three reviewed recipes exist: Memos 0.30.0 on port 5230, n8n 2.37.10 on port
  5678, and Uptime Kuma 2.5.3 on port 3001.
- The published host port is editable from 1024–65535; the container port is
  fixed by the recipe.
- Install, Connect, Discover, My Apps, and Settings already provide the routes
  needed to complete or re-enter the introduction.
- First-run completion is not persisted by the current backend. Skip state and
  automatic launch are therefore labeled implementation concepts rather than
  presented as existing product truth.
- The backend exposes no image size, elapsed-time estimate, or percentage, so
  the introduction does not show them.

## Design changes

| Before | After | Why |
| --- | --- | --- |
| Generic three-button welcome | One product promise, one primary **Get started** action, and a quiet Skip route | The first frame now has a single obvious job and passes a five-second comprehension check |
| Passing preflight looked like a setup step | Doctor runs in a compact footer while the welcome is being read | Successful system plumbing should not lengthen onboarding |
| Docker readiness appeared only as a green result | A typed Docker-missing branch explains the consequence, lists the two real checks, and provides a retry loop | Failure needs a recovery path without implying the whole product is blocked |
| Install, Connect, and Browse had equal visual weight | Reviewed install is the recommended path; Connect and Browse remain visible text routes | Choice remains available without producing a wall of competing calls to action |
| One hard-coded starter button | Three real reviewed recipes with local icons, exact versions, licenses, default ports, and editable host port | The decision is concrete and implementation-ready before the full review |
| First Run duplicated setup concepts | Starter selection hands off to the shared Phase 10 Install review; Connect opens the shared contextual form | Shared flows keep consequences and recovery behavior consistent |
| Skip silently left the flow | Skip opens Overview, confirms that nothing changed, and points to Settings for re-entry | Leaving is safe and reversible |
| Completion was a generic success banner | Ready identifies the app, address, version, data policy, and offers Open or Enter workspace | The final state closes the task and gives a useful next action |

## Flow and hierarchy

1. **Welcome.** The product promise and three concrete benefits are readable
   while Doctor runs below the primary action.
2. **Choose.** Memos is recommended, but n8n and Uptime Kuma are selectable.
   Selecting a recipe updates its real default port; editing the published port
   never changes the container port.
3. **Install.** The existing dedicated Install task receives the selected recipe
   and port. Back returns to the retained starter choice.
4. **Ready.** A successful install returns to a small completion surface with
   Open and Enter workspace. Connect and Browse deliberately complete in their
   normal shared destinations rather than creating parallel onboarding copies.

A failed preflight inserts one recovery state between Welcome and Choose.
**Check again** reruns the two checks; Connect and Browse remain usable without
Docker. A passing preflight does not create another screen.

## Five-second test

The toolkit’s five-second questions were applied to the rendered 1440×900
welcome frame before reviewing lower-level copy. This is an expert heuristic
test, not a claim of representative-user research; the latter remains a Phase
14 gate.

| Question | Expected recall from the frame | Result |
| --- | --- | --- |
| What does this product do? | Runs and reaches self-hosted apps from one workspace | Pass — stated in the dominant heading |
| Who is it for? | Someone who hosts or runs their own apps | Pass — “self-hosted” is in the heading and “what you host” is the eyebrow |
| What should I do next? | Get started | Pass — it is the only filled action in the main content |
| What should not dominate first impression? | Docker implementation detail | Pass — preflight is a compact status below the primary action |

## Browser-driven cognitive walkthrough

| Step and action | Will the user know what to do? | Feedback and recovery |
| --- | --- | --- |
| Welcome · select **Get started** | Yes; it is the only primary action | Choice appears immediately and the progress rail marks Welcome complete |
| Welcome · select **Skip for now** | Yes; the route is visible but secondary | Overview opens with confirmation that saved apps were unchanged and Settings can restart the introduction |
| Docker missing · select **Check again** | Yes; the recovery instructions precede the action | A passing result advances to Choose and confirms both checks; Connect and Browse are still available if it fails |
| Choose · select n8n | Yes; all three cards use familiar app identity and reviewed facts | Selection and checkmark move, and the port updates from 5230 to the real n8n default 5678 |
| Choose · edit port to 5680 | Yes; loopback prefix and fixed internal port disambiguate the field | Invalid range stays inline; valid input is preserved into Install |
| Choose · select **Review n8n install** | Yes; the verb promises review, not immediate execution | Shared Install opens with n8n and port 5680; no Docker change occurs yet |
| Install · use Back | Yes; the task-level route names starter choices | Selected recipe and port remain available |
| Install success · select **Finish setup** | Yes; success has already confirmed the result | Ready identifies n8n, its local address, and the next useful actions |
| Settings · select **Run the introduction again** | Yes; the row explains that saved apps are untouched | Welcome reopens with a fresh background preflight |

The browser check also verifies the full selection/install handoff, Docker retry,
Skip, and Settings re-entry paths rather than reviewing screenshots alone.

## Verification

- Six deterministic states are axe-clean at 1440×900 and 1280×800.
- No state has horizontal viewport overflow at either laptop size.
- Browser interaction verifies three recipe choices, n8n’s default port, edited
  port retention into Install, return to Ready, Docker recovery, Skip, and later
  re-entry.
- All recipe and brand icons are local; there are no remote runtime assets.
- The reusable checker is `node scripts/check-v2-first-run.mjs` while the local
  preview server is running on port 8765.

## Deterministic review URLs

- Welcome while checking: `index.html#first-run`
- Welcome after passing preflight: `index.html?preflight=ready#first-run`
- Starter choice: `index.html?step=choose&preflight=ready#first-run`
- Selected n8n: `index.html?step=choose&preflight=ready&starter=n8n#first-run`
- Starter recipe failure: `index.html?state=empty&step=choose#first-run`
- Docker missing: `index.html?preflight=missing#first-run`
- Ready: `index.html?step=ready&starter=memos#first-run`

This remains a visual prototype. It does not persist onboarding completion,
start Docker, run a container, open an external app, or mutate saved app data.
