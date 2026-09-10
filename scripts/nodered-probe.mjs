// Opt-in Node-RED first-use probe, using the Playwright already in this repo.
//
// Two things a health check cannot tell you about Node-RED: whether the editor
// actually comes up and holds its live connection, and whether the runtime
// executes a flow somebody deployed. Both are what the app is for, so both are
// checked here against a real browser and the real admin API.
import { chromium } from '@playwright/test';
import { readFile, writeFile } from 'node:fs/promises';

const [mode, endpoint, statePath] = process.argv.slice(2);
if (!['deploy', 'verify'].includes(mode) || !statePath) {
  throw new Error('deploy|verify URL STATE required');
}
const target = new URL(endpoint);
if (target.protocol !== 'http:' || !['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname)) {
  throw new Error('loopback only');
}
const base = target.origin;

// A value the flow computes rather than echoes, so a reply proves the function
// node ran instead of proving a static route exists.
const EXPECTED = 'local-store-42';
// Never a real credential: this exists to make Node-RED write an encrypted
// credentials file, and the node it belongs to is wired to nothing.
const SECRET = 'probe-only-not-a-real-secret';

const flows = [
  { id: 'lsprobe', type: 'tab', label: 'local store probe' },
  {
    id: 'lsin', type: 'http in', z: 'lsprobe', url: '/probe', method: 'get',
    wires: [['lsfn']],
  },
  {
    id: 'lsfn', type: 'function', z: 'lsprobe', outputs: 1,
    func: "msg.payload = 'local-store-' + (6 * 7); return msg;",
    wires: [['lsout']],
  },
  { id: 'lsout', type: 'http response', z: 'lsprobe', statusCode: '200' },
  // Carries a credential so the runtime has something to encrypt. Unwired, so
  // it never runs and never opens a connection.
  {
    id: 'lscred', type: 'http request', z: 'lsprobe', method: 'GET',
    url: 'http://127.0.0.1:1/unused', authType: 'basic', wires: [[]],
    credentials: { user: 'probe-user', password: SECRET },
  },
];

async function deploy() {
  const response = await fetch(`${base}/flows`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Node-RED-Deployment-Type': 'full',
    },
    body: JSON.stringify(flows),
  });
  if (!response.ok) {
    throw new Error(`deploy failed: ${response.status} ${await response.text()}`);
  }
}

async function runFlow(what) {
  const response = await fetch(`${base}/probe`);
  if (!response.ok) throw new Error(`${what}: flow endpoint returned ${response.status}`);
  const body = (await response.text()).trim();
  if (body !== EXPECTED) throw new Error(`${what}: flow returned ${JSON.stringify(body)}`);
}

/// The editor, in a real browser: it has to render and hold the websocket it
/// uses for live runtime updates, and it has to show the deployed flow.
async function editor() {
  const browser = await chromium.launch({ headless: true, channel: 'msedge' });
  try {
    const page = await browser.newPage();
    page.setDefaultTimeout(30000);
    const sockets = [];
    page.on('websocket', (socket) => sockets.push(socket));
    await page.goto(base);
    // The workspace is the editor proper, not just a page that answered.
    await page.locator('#red-ui-workspace').waitFor({ state: 'visible' });
    await page.locator('#red-ui-palette').waitFor({ state: 'visible' });
    // The deployed flow's tab is what proves the editor is reading the same
    // runtime the API deployed to.
    await page.locator('#red-ui-workspace-tabs').getByText('local store probe').waitFor();
    // Observed on the canvas rather than read out of the editor's internals:
    // the deployed function node is drawn, so the editor loaded the runtime's
    // flow rather than an empty workspace that merely rendered.
    await page.locator('#red-ui-workspace').getByText('function', { exact: true }).first()
      .waitFor({ state: 'visible' });
    const comms = sockets.filter((socket) => socket.url().includes('/comms'));
    if (!comms.length) throw new Error('the editor opened no /comms websocket');
    if (comms.some((socket) => socket.isClosed())) {
      throw new Error('the editor websocket closed immediately');
    }
  } finally {
    await browser.close();
  }
}

if (mode === 'deploy') {
  await deploy();
  await runFlow('after deploy');
  await editor();
  await writeFile(statePath, JSON.stringify({ expected: EXPECTED, secret: SECRET }));
} else {
  const state = JSON.parse(await readFile(statePath, 'utf8'));
  if (state.expected !== EXPECTED) throw new Error('probe state does not match this script');
  await runFlow('after restart');
  await editor();
}
console.log(`Node-RED ${mode} probe passed`);
