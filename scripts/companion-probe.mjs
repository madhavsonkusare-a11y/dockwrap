// The standard probe, plus the second addresses an app's own pages call.
//
// Sim's browser keeps a socket open to its realtime server, Maxun's frontend
// calls its backend, and LobeHub uploads straight to its object store. Each
// is published on a second loopback port, and the standard probe would pass
// with every one of them broken: the first page loads either way. This finds
// the app's other published ports from Docker and asks each one to answer.
//
//     node scripts/companion-probe.mjs first-use|verify URL STATE [--inside VAR]
//
// `--inside VAR` also checks, from inside the app's own container, the
// address that environment variable names. LobeHub reads files back from the
// same S3 address the browser uploads to, which only works inside because
// its start script forwards it.
import { execFileSync, spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const [mode, endpoint, statePath, ...rest] = process.argv.slice(2);
const insideVar = rest[0] === '--inside' ? rest[1] : null;

const here = path.dirname(fileURLToPath(import.meta.url));
const standard = spawnSync(
  process.execPath,
  [path.join(here, 'standard-probe.mjs'), mode, endpoint, statePath],
  { stdio: 'inherit' },
);
if (standard.status !== 0) process.exit(standard.status ?? 1);

const docker = (...args) => execFileSync('docker', args, { encoding: 'utf8' }).trim();
const mainPort = new URL(endpoint).port;
const project = docker(
  'ps', '--filter', `publish=${mainPort}`,
  '--format', '{{.Label "com.docker.compose.project"}}',
).split('\n')[0];
if (!project) throw new Error(`no container publishes port ${mainPort}`);

const rows = docker(
  'ps', '--filter', `label=com.docker.compose.project=${project}`,
  '--format', '{{.Names}}|{{.Ports}}',
).split('\n');
const seconds = [];
let mainContainer = null;
for (const row of rows) {
  const [name, ports = ''] = row.split('|');
  for (const match of ports.matchAll(/127\.0\.0\.1:(\d+)->(\d+)\/tcp/g)) {
    if (match[1] === mainPort) mainContainer = name;
    else seconds.push({ name, port: match[1] });
  }
}
if (seconds.length === 0) throw new Error(`${project} publishes no second address`);

for (const { name, port } of seconds) {
  // A second address may come up after the app's own page does, so this
  // waits rather than judging the first refusal.
  let status;
  let failure;
  const deadline = Date.now() + 90000;
  do {
    try {
      const response = await fetch(`http://127.0.0.1:${port}/`, {
        signal: AbortSignal.timeout(15000),
        redirect: 'manual',
      });
      status = response.status;
      break;
    } catch (error) {
      failure = error;
      await new Promise((wait) => setTimeout(wait, 3000));
    }
  } while (Date.now() < deadline);
  if (status === undefined) {
    throw new Error(`${name}'s second address, port ${port}, did not answer: ${failure.message}`);
  }
  // Any answer from the service itself will do: a socket server has nothing
  // at `/`, and an object store refuses an unsigned listing. A 5xx is a
  // proxy or the service failing, which is not an answer.
  if (status >= 500) throw new Error(`${name}'s second address, port ${port}, answered ${status}`);
  console.log(`${name} answers on its second address, port ${port} (${status})`);
}

if (insideVar) {
  if (!mainContainer) throw new Error('could not find the container behind the main address');
  const script =
    `const u=process.env[${JSON.stringify(insideVar)}];` +
    `if(!u){console.error('unset');process.exit(2)}` +
    `fetch(new URL('/health',u),{signal:AbortSignal.timeout(15000)})` +
    `.then(r=>{console.log(r.status);process.exit(r.status<500?0:1)})` +
    `.catch(e=>{console.error(e.message);process.exit(1)})`;
  const inside = spawnSync('docker', ['exec', mainContainer, 'node', '-e', script], {
    encoding: 'utf8',
  });
  if (inside.status !== 0) {
    throw new Error(
      `${insideVar} does not answer from inside ${mainContainer}: ${(inside.stderr || inside.stdout).trim()}`,
    );
  }
  console.log(`${insideVar} answers from inside ${mainContainer} (${inside.stdout.trim()})`);
}
console.log(`second-address ${mode} probe passed`);
