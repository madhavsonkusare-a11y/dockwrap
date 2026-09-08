# CapRover setup primitive compatibility

Pinned revision: `7f1f329125dde0cb915f47c8b33293eca45f7d0e`; 356 definitions.

Reproduce: `python scripts/caprover_setup_report.py` (cached archive required).
Runs the actual Rust parsers on normalized upstream variables. No installs,
credential generation, source-expression execution or catalog promotion.

| Measurement | Count |
| --- | --- |
| apps with refused primitives | 75 |
| patterns refused | 87 |
| patterns supported | 1377 |
| secret declarations refused | 52 |
| secret declarations supported | 222 |
| variables mapped | 2387 |
| variables refused | 190 |

Supported means only that the declaration can be represented by these primitives.
Variable mapping checks literal defaults and rules together; the separate primitive
counts do not. Neither checks variable substitution, service topology, platform
requirements, image support or successful installation. ASCII-only pattern fields
are intentionally narrower than JavaScript regexes. A full adapter must report
these limitations and preserve every other upstream constraint.

## Setup-variable mapping refusals

First refusal per variable; categories can hide additional limitations.

| Reason | Variables |
| --- | --- |
| Fractional default cannot be read back exactly | 8 |
| Invalid CapRover variable identifier | 17 |
| Platform variable cannot be redeclared | 1 |
| Unsupported CapRover validation pattern | 78 |
| Unsupported generated-secret expression or length | 61 |
| Unsupported setup variable property | 7 |
| Upstream default does not satisfy its setup rule | 18 |

## Apps needing primitive follow-up

Counts only; upstream values are not included in diagnostics.

| App | Refused patterns | Refused secret declarations |
| --- | --- | --- |
| ackee | 0 | 1 |
| affine | 1 | 0 |
| btcpayserver | 0 | 1 |
| chaskiq | 4 | 3 |
| chatwoot | 3 | 0 |
| claper | 1 | 0 |
| collabora-online | 0 | 1 |
| commento | 1 | 0 |
| directus | 4 | 1 |
| directus-mysql-redis | 5 | 0 |
| docmost | 2 | 0 |
| flagsmith | 1 | 0 |
| forge_minecraft | 0 | 2 |
| formance-ledger | 0 | 1 |
| formbricks | 1 | 0 |
| freshrss | 2 | 3 |
| ghost | 2 | 2 |
| gitea | 0 | 1 |
| gogost | 1 | 0 |
| hasura | 1 | 0 |
| hasura-only | 2 | 0 |
| healthchecks | 1 | 0 |
| hedgedoc | 2 | 0 |
| homepage | 1 | 0 |
| imgproxy | 1 | 0 |
| influxdb2 | 0 | 1 |
| invoiceninja | 4 | 3 |
| iredmail | 0 | 1 |
| kanboard-sqlite | 1 | 0 |
| leantime | 1 | 0 |
| maildev | 0 | 1 |
| mastodon | 4 | 2 |
| matrix-conduit | 2 | 0 |
| matrix-synapse | 2 | 0 |
| matrix-synapse-only | 2 | 0 |
| mattermost-ee | 1 | 0 |
| mattermost-team | 1 | 0 |
| mcp-context-forge | 0 | 1 |
| miniflux | 0 | 2 |
| minio | 2 | 0 |
| mumble | 0 | 1 |
| neo4j | 0 | 1 |
| netbox | 0 | 3 |
| nextcloud | 2 | 1 |
| onlyoffice-documentserver | 0 | 1 |
| paperless-ng | 0 | 1 |
| paperless-ngx | 0 | 1 |
| peertube | 0 | 1 |
| penpot | 4 | 0 |
| percona | 0 | 1 |
| pgadmin4 | 0 | 1 |
| pigallery2 | 1 | 0 |
| pihole | 0 | 1 |
| portainer | 1 | 0 |
| posthog | 0 | 1 |
| prisma | 1 | 0 |
| prometheus | 1 | 0 |
| rallly | 2 | 0 |
| rclone | 0 | 1 |
| remark42 | 0 | 1 |
| rocketchat | 4 | 0 |
| rudder-stack | 1 | 0 |
| rustfs | 2 | 0 |
| seafile | 0 | 2 |
| seafile-nomemcached | 0 | 2 |
| serpbear | 3 | 0 |
| silex-platform | 2 | 0 |
| squidex | 2 | 0 |
| supertokens | 1 | 0 |
| teslamate | 0 | 1 |
| valheim | 0 | 1 |
| vitodeploy | 0 | 1 |
| weblate | 1 | 0 |
| yagpdb | 1 | 1 |
| zammad | 2 | 1 |
