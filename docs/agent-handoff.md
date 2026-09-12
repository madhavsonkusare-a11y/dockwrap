# Agent handoff

Updated September 12, 2026. Start with [V1_TASKS.md](V1_TASKS.md); it is the
only release ledger. [Documentation index](README.md) explains the rest.

## Current work and checkout

Phase 0 (R01–R03) is complete. Main contains the integrated code and the
consolidated V1 documentation plus the frozen 100-app planning roster.

- PRs #3–#6 are merged. Main integration commit `7d9e0d3` has exactly the
  tested tree from PR #6 head `720ce23d46a8bcfd7b401b5ef94f840296f74749`.
  Quality, minimum-Rust, Windows, macOS and Linux passed
  [run 34695407242](https://github.com/madhavsonkusare-a11y/local-store/actions/runs/34695407242).
- Merged feature/design branches were removed after ancestry checks. The
  temporary documentation integration branch can be removed after publication.
- Keep the stash `pre-consolidation-local-documents-2026-09-12`: its 335 V2
  files match integrated content; its three extra drafts are preserved in
  research/screening documents. Keep unrelated detached worktrees intact.
- The unmerged Claude cleanup commit remains preserved by the pushed tag
  `archive/claude-cleanup-2026-09-12`. Do not apply it merely because archived.
- [Frozen roster](v1-app-roster.md): 52 existing offerings and 48 expansion
  candidates, each with a useful task and explicit acceptance gaps. Live public
  repository checks excluded archived File Browser and Pingvin Share. Selection
  does not promote candidates, refresh old image pins or establish agent access.
- Regenerate the matrix with `python scripts/build-v1-roster.py`; use `--check`
  to detect input drift. Membership changes require an explained selection diff.

## Owner decisions to carry forward

V1 requires 100 distinct tested/verified offerings (existing target), a bundled
engine, owner-led V3 frontend, signed Windows delivery, an automatic updater,
and agents managing the store and accessing every offered app. The new signing
requirement supersedes its earlier deferral. V2 is reference material; do not
start its production integration while V3 is being designed.

Source-available apps remain allowed with accurate notices. The eight Umbrel
icons remain under the owner's existing retain-with-NOASSERTION decision;
that does not settle broad distribution rights. Shrimply remains excluded.

## Next bounded implementation batch

Next: E01/E02 and Q01/Q02 from the master ledger. Verify a maintained
Moby/Compose WSL packaging approach, define the engine seam and evidence
identity, then add all-service health checks. Define A01's app-access matrix in
that batch so app qualification and agent capability evidence can share identity.
Do not start another large app batch against Docker Desktop before planning
how those results will be re-proven on the managed engine.

V3 approval and owner/provider signing credentials are external inputs.
Backend work can proceed independently. No keys, accounts, spending, release
publication or third-party messaging is authorized by these documents.

## Baseline and limitations

52 offerings (3 recipes, 49 approved templates), 1,678 discovery entries with
local icons. Generic proof establishes startup/actionable page/persistence;
it does not demonstrate a useful task in every app. No bundled engine, agent
gateway, V3 production UI or updater exists yet. See the master ledger for
remaining work rather than old app counts in historical evidence.

PR integration fixed OCI metadata CI, filled image caches, verified Tautulli's
original digest after its tag moved, filled five missing icon records, fetched
full history for catalog provenance, and corrected Compose digest validation.
The default Rust suite, strict Clippy and all remote checks passed at the head
above. Documentation changes after that head are not covered by that CI run.

## Working and verification

Reuse `src/importers`, `src/plan.rs`, `src/setup`, `src/runtime`,
`src/qualification.rs`, `src/offerings.rs` and existing frontend state modules.
Read [lessons.md](lessons.md) before Docker work. Run only one qualification
at a time, use private roots/project labels and never prune unrelated resources.
Choose the source explicitly where alternate definitions differ. Rebuild the
CLI before proving changed manifests. Pulling by digest does not restore a tag.

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
python scripts/test_catalog.py
python scripts/test_template_platforms.py
python scripts/test_validate_recipes.py
python scripts/check-template-platforms.py
python scripts/catalog_pipeline.py --check
python scripts/cache-catalog-icons.py --check
python scripts/validate-recipes.py
node scripts/generate-brand.mjs --check
npm test -- --ignore-snapshots
```

Select relevant checks for each bounded change; do not repeat Docker proofs for
text-only edits. Record task IDs, actual result and next action before stopping.
