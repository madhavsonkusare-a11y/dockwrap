# CapRover deployment adapter: next bounded batch

Status: **done.** The adapter is implemented in `src/importers/caprover.rs`,
measured over all 356 pinned definitions by
`python scripts/caprover_import_report.py` (117 expressible, 239 blocked), and
proven end to end against real Docker by `tests/caprover_lifecycle.rs`, which
installs the pinned CodiMD definition and takes it through health, restart,
preserved-data reinstall and deletion.

Kept for the record of what was asked for and why. The remaining measured wins
are listed in the handoff, and the largest is the generated-secret-with-a-rule
question this checklist deliberately left refused.

Original brief follows.
Use the pinned archive in `catalog/import-audit-sources.json` and the existing
`DeploymentPlan`, `PlanTemplate`, runtime transaction and Runtipi report pattern.
Do not create a second install engine or promote imported apps automatically.

## Implement

1. Normalize checksum-verified YAML to JSON with existing PyYAML. Bound member
   size, never extract paths from the archive or execute source expressions.
2. Accept captainVersion 4 explicitly. Allowlist root/service properties; report
   unknown deployment behavior rather than ignoring it. Metadata may be retained
   separately. Preserve Apache-2.0 attribution when bundling source definitions.
3. Map variables using `caprover::setup_variable`. Detect duplicate IDs and
   uppercase-normalization collisions. Fields currently default to sensitive
   because upstream lacks reliable credential annotations. Expose the ASCII
   restriction in any future setup UI. Generated secrets with regex constraints
   remain refused until compatibility can be proven for all generated values.
4. Map service names and `srv-captain--` references consistently. Reject unresolved
   platform variables, cycles and references to absent services. Do not execute
   `$$` expressions. Replace declared references only, avoiding prefix collisions.
5. Resolve image-version defaults explicitly and validate the final pinned image.
   Do not present an editable setup field if changing it cannot change the image.
   Floating images need review; successful parsing does not establish platform support.
6. Preserve named-volume identity using `PlanMount::Volume`. Refuse host paths,
   Docker socket mounts, privileged behavior and unsupported mount options.
7. Respect `notExposeAsWebApp`; an absent containerHttpPort defaults to 80 in
   CapRover, but do not invent multiple endpoints. Reuse the existing safe host-port
   mapping for privileged container ports. Keep database ports unpublished.
8. Treat HTTPS, root-domain routing, proxy/auth and websocket requirements as
   explicit deployment requirements. A localhost port does not replace all of them.
9. Return a complete validated template or named limitations. No partially working
   plans, dropped health checks, ignored commands or automatic recipe promotion.

## Validate before widening support

- One simple web service with a named volume and fixed version default.
- Web plus database: service DNS substitution, generated hex credential shared by
  both services, unpublished database, dependency order, volume identity.
- Refusals: unresolved expressions, generated secret with incompatible rule,
  duplicate/colliding variables, unknown properties, host paths and platform routing.
- Reproducible report over every pinned definition using the actual Rust adapter.
  Count parseable templates separately from reviewed and Docker-proven recipes.
- One reviewed imported app lifecycle in an isolated project/private registry:
  install, health, commit, restart, retained data, reinstall with original secret,
  explicit cleanup and unrelated-container preservation. Reuse existing proof helpers.

The existing Adminer/Postgres test proves the shared plan transaction, not any
CapRover application definition. Catalog expansion should follow adapter evidence.
