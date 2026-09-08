import {createServer} from 'node:http';
const port = Number(process.argv[2]);
if (!Number.isInteger(port) || port < 1024) throw new Error('Provide a local test port');
const probes = [];
const page = `<!doctype html><meta charset="utf-8"><title>Native smoke fixture</title><h1>Native smoke notes</h1>
<script>
(async () => {
 const invoke = window.__TAURI__?.core?.invoke || window.__TAURI_INTERNALS__?.invoke;
 const results = [];
 for (const [command,args] of [['list_apps',{}],['cancel_app_setup',{id:'native-smoke',operationId:1}],['app_readiness',{id:'native-smoke'}],['take_activation_errors',{}]]) {
  if (!invoke) { results.push({command, denied:true, reason:'bridge unavailable'}); continue; }
  try { await invoke(command,args); results.push({command,denied:false}); }
  catch (error) { results.push({command,denied:true,reason:String(error?.message || error)}); }
 }
 await fetch('/probe',{method:'POST',body:JSON.stringify({results})});
})();
</script>`;
createServer(async (req,res) => {
 if (req.method === 'POST' && req.url === '/probe') {
  let data = ''; for await (const chunk of req) { data += chunk; if (data.length > 16384) {res.writeHead(413);res.end();return;} }
  probes.push(JSON.parse(data)); res.end('ok'); return;
 }
 if (req.url === '/results') {res.setHeader('content-type','application/json');res.end(JSON.stringify(probes));return;}
 res.setHeader('content-type','text/html');res.end(page);
}).listen(port,'127.0.0.1');
