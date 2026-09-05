import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';
const root = decodeURIComponent(new URL('../src/', import.meta.url).pathname).slice(process.platform === 'win32' ? 1 : 0);
const types = {'.html':'text/html','.js':'text/javascript','.css':'text/css','.svg':'image/svg+xml','.woff2':'font/woff2'};
createServer(async (req,res) => {
  try {
    const raw = req.url.split('?')[0]; const relative = decodeURIComponent(raw === '/' ? 'index.html' : (raw.startsWith('/') ? raw.slice(1) : raw));
    const path = normalize(join(root, relative));
    if (!path.startsWith(normalize(root)) || !(await stat(path)).isFile()) throw new Error();
    res.writeHead(200, {'content-type': types[extname(path)] || 'application/octet-stream'}); res.end(await readFile(path));
  } catch { res.writeHead(404); res.end('Not found'); }
}).listen(4173, '127.0.0.1', () => console.log('Preview http://127.0.0.1:4173'));


