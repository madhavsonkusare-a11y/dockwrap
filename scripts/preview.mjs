import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
const root = decodeURIComponent(new URL('../src/', import.meta.url).pathname).slice(process.platform === 'win32' ? 1 : 0);
const types = {'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.png':'image/png','.woff2':'font/woff2'};
const server = createServer(async (req,res) => {
  try {
    const raw = req.url.split('?')[0]; const relative = decodeURIComponent(raw === '/' ? 'index.html' : (raw.startsWith('/') ? raw.slice(1) : raw));
    const path = normalize(join(root, relative));
    if (!path.startsWith(normalize(root)) || !(await stat(path)).isFile()) throw new Error();
    res.writeHead(200, {'content-type': types[extname(path)] || 'application/octet-stream'}); res.end(await readFile(path));
  } catch { res.writeHead(404); res.end('Not found'); }
});
// Playwright tears this server down at the end of a run. An http.Server stays
// alive while any socket is open, and a browser holds keep-alive connections
// well past the last request, so without this the process can outlive the run
// and leave the port bound.
server.keepAliveTimeout = 1000;
server.listen(4173, '127.0.0.1', () => console.log('Preview http://127.0.0.1:4173'));
let stopping = false;
function shutdown() {
  if (stopping) return;
  stopping = true;
  server.close(() => process.exit(0));
  server.closeAllConnections?.();
  // Never let cleanup itself become the hang.
  setTimeout(() => process.exit(0), 2000).unref();
}
for (const signal of ['SIGINT', 'SIGTERM', 'SIGHUP', 'SIGBREAK']) {
  try { process.on(signal, shutdown); } catch { /* not every signal exists on every platform */ }
}
// Windows cannot deliver a catchable signal to a detached child, but the parent
// closing our stdin is a reliable end-of-run marker there.
process.stdin.on('close', shutdown);
process.stdin.resume();


