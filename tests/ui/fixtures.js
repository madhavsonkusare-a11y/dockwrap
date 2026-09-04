export const catalog = [
 { name:'Actual Budget', source_url:'https://actualbudget.org', description:'A privacy-focused app for managing your finances.', category:'Money, Budgeting & Management', license:'MIT', icon:null, warning:false, capability:'connect' },
 { name:'Immich', source_url:'https://immich.app', description:'Self-hosted photo and video management solution.', category:'Photo Galleries', license:'AGPL-3.0', icon:null, warning:false, capability:'connect' },
 { name:'Memos', source_url:'https://usememos.com', description:'A lightweight, self-hosted memo hub.', category:'Note-taking & Editors', license:'MIT', icon:null, warning:false, capability:'verified_install', recipe_id:'memos' },
 { name:'Uptime Kuma', source_url:'https://github.com/louislam/uptime-kuma', description:'A friendly monitoring tool.', category:'Status / Uptime pages', license:'MIT', icon:null, warning:true, capability:'connect' }
];
export const apps = [{ id:'studio-notes', display_name:'Studio notes', launch_url:'http://localhost:5230', icon_path:null, runtime:{kind:'external'}, status:'connected', created_at_unix:1, updated_at_unix:1, catalog_id:null }];
const recipe = { id:'memos', display_name:'Memos', version:'0.30.0', image:'neosmemo/memos:0.30.0', launch_url:'http://localhost:5230', risk_notes:['Creates a Docker container named local-store-memos.','Binds port 5230 on this computer.','Stores app data in the Local Store managed directory.'] };
export function installAdapter(page, options = {}) {
 const fixtureCatalog = options.catalog ?? catalog, fixtureApps = options.apps ?? apps;
 return page.addInitScript(({catalog, apps, failure, recipe}) => {
   let current = structuredClone(apps);
   window.__calls = [];
   window.__TAURI__ = { core: { invoke: async (command, args = {}) => {
     window.__calls.push({ command, args }); if (failure === command) throw new Error(`Could not ${command.replaceAll('_',' ')}`);
     if (command === 'search_catalog') {
       const query = args.query.toLowerCase(), filtered = catalog.filter(a => (!args.category || a.category === args.category) && (`${a.name} ${a.description}`).toLowerCase().includes(query));
       return { entries: filtered.slice(args.offset, args.offset + args.limit), total: filtered.length, catalog_total: 1257, offset: args.offset, limit: args.limit, categories: [...new Set(catalog.map(a=>a.category))].sort() };
     }
     if (command === 'list_apps') return structuredClone(current);
     if (command === 'add_app') { current.push({ id:args.name.toLowerCase().replaceAll(' ','-'), display_name:args.name, launch_url:args.url, icon_path:null, runtime:{kind:'external'}, status:'connected', catalog_id:null, created_at_unix:1, updated_at_unix:1 }); return; }
     if (command === 'remove_app_cmd' || command === 'uninstall_app') { current = current.filter(a => a.id !== args.id); return; }
     if (command === 'recipe_details') return recipe;
     if (command === 'doctor') return { ready:true, checks:[{id:'docker',label:'Docker engine',ok:true,detail:'28.0.1'},{id:'compose',label:'Docker Compose',ok:true,detail:'2.35.1'}] };
     if (command === 'install_app') { current.push({ id:'memos', display_name:'Memos', launch_url:'http://localhost:5230', icon_path:null, runtime:{kind:'compose'}, status:'running', catalog_id:'Memos', created_at_unix:1, updated_at_unix:1 }); return; }
     if (command === 'app_logs') return 'memos  | server started on port 5230';
     if (command === 'start_app') { current.find(a=>a.id===args.id).status='running'; return; }
     if (command === 'stop_app') { current.find(a=>a.id===args.id).status='stopped'; return; }
   }}};
 }, { catalog: fixtureCatalog, apps: fixtureApps, failure: options.failure, recipe });
}
