// Penpot, checked as a person would use it — including by an AI agent.
//
// First the App Store standard: the address opens something to act on, and
// the same page after a restart and a reinstall. Then the part only Penpot
// has: with `enable-mcp`, its frontend proxies an MCP server at
// `/mcp/stream`, and an MCP client has to be able to open a session there and
// list the tools it would use. A container that merely starts proves nothing
// about that; a session that answers does.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const [mode, endpoint, statePath] = process.argv.slice(2);
if (!['first-use', 'verify'].includes(mode) || !statePath) {
  throw new Error('first-use|verify URL STATE required');
}
const target = new URL(endpoint);
if (target.protocol !== 'http:' || !['127.0.0.1', 'localhost', '[::1]'].includes(target.hostname)) {
  throw new Error('loopback only');
}

const here = path.dirname(fileURLToPath(import.meta.url));
const standard = spawnSync(process.execPath, [path.join(here, 'standard-probe.mjs'), mode, endpoint, statePath], {
  encoding: 'utf8',
});
if (standard.status !== 0) {
  process.stderr.write(standard.stderr);
  throw new Error('the standard first-use check failed');
}

const stream = new URL('/mcp/stream', endpoint);
const headers = { 'content-type': 'application/json', accept: 'application/json, text/event-stream' };

// MCP answers either as plain JSON or as a server-sent event; read both.
async function call(body, session) {
  const response = await fetch(stream, {
    method: 'POST',
    headers: session ? { ...headers, 'mcp-session-id': session } : headers,
    body: JSON.stringify(body),
    signal: AbortSignal.timeout(30000),
  });
  const text = await response.text();
  if (!response.ok) {
    throw new Error(`MCP answered ${response.status} at ${stream.pathname}: ${text.slice(0, 120)}`);
  }
  const payload = text.trim().startsWith('{')
    ? text
    : text.split('\n').filter((line) => line.startsWith('data:')).map((line) => line.slice(5)).join('');
  return { message: JSON.parse(payload), session: response.headers.get('mcp-session-id') };
}

// The MCP server can come up a little after the page does.
let opened;
const deadline = Date.now() + 60000;
for (;;) {
  try {
    opened = await call({
      jsonrpc: '2.0', id: 1, method: 'initialize',
      params: { protocolVersion: '2025-03-26', capabilities: {}, clientInfo: { name: 'local-store-probe', version: '1' } },
    });
    break;
  } catch (error) {
    if (Date.now() > deadline) throw error;
    await new Promise((resolve) => setTimeout(resolve, 2000));
  }
}
const server = opened.message.result?.serverInfo?.name;
if (!server) {
  throw new Error(`MCP initialize returned no server: ${JSON.stringify(opened.message).slice(0, 160)}`);
}

await fetch(stream, {
  method: 'POST',
  headers: opened.session ? { ...headers, 'mcp-session-id': opened.session } : headers,
  body: JSON.stringify({ jsonrpc: '2.0', method: 'notifications/initialized' }),
});
const listed = await call({ jsonrpc: '2.0', id: 2, method: 'tools/list' }, opened.session);
const tools = listed.message.result?.tools ?? [];
if (tools.length === 0) {
  throw new Error('the MCP server listed no tools');
}
console.log(`penpot ${mode} probe passed: ${server} offers ${tools.length} MCP tool(s) at ${stream.pathname}`);
