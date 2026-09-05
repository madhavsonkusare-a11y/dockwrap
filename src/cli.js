const fs = require('fs');
const path = require('path');

// Minimal CLI helper to register an app (writes apps.json)
// Usage: node cli.js register <name> --url <url> [--icon <path>]
const appdata = path.join(process.env.APPDATA, 'local-store', 'apps.json');

function loadApps() {
  try {
    const raw = fs.readFileSync(appdata, 'utf8');
    return JSON.parse(raw);
  } catch {
    return {};
  }
}

function saveApps(map) {
  fs.mkdirSync(path.dirname(appdata), { recursive: true });
  fs.writeFileSync(appdata, JSON.stringify(map, null, 2));
}

const [,, ...args] = process.argv;
const cmd = args[0];

if (cmd === 'register') {
  const name = args[1];
  const urlIdx = args.indexOf('--url');
  const url = urlIdx > -1 ? args[urlIdx + 1] : null;
  const iconIdx = args.indexOf('--icon');
  const icon = iconIdx > -1 ? args[iconIdx + 1] : null;
  if (!name || !url) {
    console.error('Usage: node cli.js register <name> --url <url> [--icon <path>]');
    process.exit(1);
  }
  const map = loadApps();
  map[name] = { name, url, icon: icon || undefined };
  saveApps(map);
  console.log(`Registered "${name}" -> ${url}`);
} else if (cmd === 'list') {
  console.log(JSON.stringify(loadApps(), null, 2));
} else {
  console.error('Usage: node cli.js <register|list>');
  process.exit(1);
}
