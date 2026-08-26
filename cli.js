// v0.2: replace with a Rust CLI.
// Registers an app in %APPDATA%/dockwrap/apps.json.
//   node cli.js register <name> --url <url> [--icon <path>]
//   node cli.js list
const fs = require('fs');
const path = require('path');

const appdata = process.env.APPDATA || path.join(process.env.HOME, '.config');
const cfg = path.join(appdata, 'dockwrap', 'apps.json');

function load() {
  try { return JSON.parse(fs.readFileSync(cfg, 'utf8')); } catch { return []; }
}
function save(map) {
  fs.mkdirSync(path.dirname(cfg), { recursive: true });
  fs.writeFileSync(cfg, JSON.stringify(map, null, 2));
}

const [,, ...args] = process.argv;
const cmd = args[0];

if (cmd === 'register') {
  const name = args[1];
  const u = args.indexOf('--url');
  const url = u > -1 ? args[u + 1] : null;
  const i = args.indexOf('--icon');
  const icon = i > -1 ? args[i + 1] : undefined;
  if (!name || !url) {
    console.error('Usage: node cli.js register <name> --url <url> [--icon <path>]');
    process.exit(1);
  }
  const map = load().filter(a => a.name !== name);
  map.push({ name, url, icon });
  save(map);
  console.log(`Registered "${name}" -> ${url}`);
} else if (cmd === 'list') {
  console.log(JSON.stringify(load(), null, 2));
} else {
  console.error('Usage: node cli.js <register|list>');
  process.exit(1);
}
