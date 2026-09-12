// LobeHub's probe: the second-address check, and its S3 address from inside
// LobeHub's own container, where Local Store's start script forwards it.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const here = path.dirname(fileURLToPath(import.meta.url));
const result = spawnSync(
  process.execPath,
  [path.join(here, 'companion-probe.mjs'), ...process.argv.slice(2), '--inside', 'S3_ENDPOINT'],
  { stdio: 'inherit' },
);
process.exit(result.status ?? 1);
