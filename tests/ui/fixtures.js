export const catalog = [
 { name:'Actual Budget', source_url:'https://actualbudget.org', description:'A privacy-focused app for managing your finances.', category:'Money, Budgeting & Management', license:'MIT', icon:null, warning:false, capability:'connect' },
 { name:'Immich', source_url:'https://immich.app', description:'Self-hosted photo and video management solution.', category:'Photo Galleries', license:'AGPL-3.0', icon:null, warning:false, capability:'connect' },
 { name:'Memos', source_url:'https://usememos.com', description:'A lightweight, self-hosted memo hub.', category:'Note-taking & Editors', license:'MIT', icon:null, warning:false, capability:'connect' },
 { name:'Uptime Kuma', source_url:'https://github.com/louislam/uptime-kuma', description:'A friendly monitoring tool.', category:'Status / Uptime pages', license:'MIT', icon:null, warning:true, capability:'connect' }
];
export const apps = [{ name:'Studio notes', url:'http://localhost:5230', icon:null, compose:null, health:null }];
export function installAdapter(page, options = {}) {
 const fixtureCatalog = options.catalog ?? catalog, fixtureApps = options.apps ?? apps;
 return page.addInitScript(({catalog, apps, failure}) => {
   let current = structuredClone(apps);
   window.__calls = [];
   window.__TAURI__ = { core: { invoke: async (command, args = {}) => {
     window.__calls.push({ command, args }); if (failure === command) throw new Error(`Could not ${command.replaceAll('_',' ')}`);
     if (command === 'search_catalog') {
       const query = args.query.toLowerCase(), filtered = catalog.filter(a => (!args.category || a.category === args.category) && (`${a.name} ${a.description}`).toLowerCase().includes(query));
       return { entries: filtered.slice(args.offset, args.offset + args.limit), total: filtered.length, catalog_total: 1257, offset: args.offset, limit: args.limit, categories: [...new Set(catalog.map(a=>a.category))].sort() };
     }
     if (command === 'list_apps') return structuredClone(current);
     if (command === 'add_app') { current.push({ name: args.name, url: args.url, icon:null, compose:null, health:null }); return; }
     if (command === 'remove_app_cmd') { current = current.filter(a => a.name !== args.name); return; }
   }}};
 }, { catalog: fixtureCatalog, apps: fixtureApps, failure: options.failure });
}
