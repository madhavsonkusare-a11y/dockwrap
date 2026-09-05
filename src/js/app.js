import { invoke } from './api.js';
import { discoveryCard, installedRow, emptyState, detail, recipeView } from './render.js';
import { showDialog, closeDialog, setDialogBusy, revealToast } from './motion.js';

const $ = id => document.getElementById(id);
const state = { view: 'discover', query: '', category: '', offset: 0, limit: 12, entries: [], apps: [], visibleApps: [], total: 0 };
let request = 0, recipeRequest = 0, searchTimer, toastTimer, pendingApp, activeRecipe, refreshError;
const message = error => error?.message || String(error);
function toast(text) {
  clearTimeout(toastTimer);
  $('toast').textContent = text;
  $('toast').hidden = false;
  revealToast($('toast'));
  toastTimer = setTimeout(() => { $('toast').hidden = true; }, 6000);
}
function loadState(loading) { $('content').setAttribute('aria-busy', String(loading)); }
async function refreshApps() {
  try {
    state.apps = await invoke('list_apps');
    refreshError = null;
    $('app-count').textContent = state.apps.length;
  } catch (error) { refreshError = error; $('app-count').textContent = '–'; }
}
async function render() {
  const token = ++request;
  $('pagination').hidden = true;
  if (state.view === 'apps') {
    loadState(false);
    if (refreshError) { showError(refreshError); return; }
    const query = state.query.toLowerCase();
    state.visibleApps = state.apps.filter(app => `${app.display_name} ${app.launch_url}`.toLowerCase().includes(query));
    $('results-count').textContent = `${state.visibleApps.length} ${state.visibleApps.length === 1 ? 'app' : 'apps'}`;
    $('content').innerHTML = state.visibleApps.length
      ? `<div class="installed-list">${state.visibleApps.map(installedRow).join('')}</div>`
      : emptyState(state.query ? 'No matching apps' : 'Your apps belong here.', state.query ? 'Try another name or address.' : 'Install a reviewed recipe or connect an app you already run.', state.query ? 'clear' : 'discover', state.query ? 'Clear search' : 'Discover apps');
    return;
  }
  loadState(true);
  // Keep the previous results in place during search; avoid a loading flash on every key.
  if (!$('content').children.length) $('content').innerHTML = '<div class="loading" role="status">Finding your next app…</div>';
  try {
    const page = await invoke('search_catalog', { query: state.query, category: state.category, offset: state.offset, limit: state.limit });
    if (token !== request) return;
    state.entries = page.entries;
    state.total = page.total;
    const select = $('category');
    if (select.options.length === 1) page.categories.forEach(category => select.add(new Option(category, category)));
    select.value = state.category;
    $('results-count').textContent = `${page.total.toLocaleString()} ${page.total === 1 ? 'project' : 'projects'}`;
    $('catalog-note').textContent = `${page.catalog_total.toLocaleString()} projects to discover · Reviewed installs are clearly marked`;
    $('content').innerHTML = page.entries.length
      ? `<div class="app-grid">${page.entries.map(discoveryCard).join('')}</div>`
      : emptyState('Nothing here just yet.', 'Try a different search or category. You can also connect an app that isn’t in this collection.', 'clear', 'Clear filters', 'search');
    $('pagination').hidden = page.total <= state.limit;
    $('page-label').textContent = `${state.offset + 1}–${Math.min(state.offset + state.limit, page.total)} of ${page.total.toLocaleString()} projects`;
    $('previous').disabled = state.offset === 0;
    $('next').disabled = state.offset + state.limit >= page.total;
  } catch (error) { if (token === request) showError(error); }
  finally { if (token === request) loadState(false); }
}
function showError(error) { $('content').innerHTML = emptyState('We couldn’t load your workspace.', message(error), 'retry', 'Try again', 'circle-alert'); }
function navigate(view) {
  clearTimeout(searchTimer);
  state.view = view; state.query = ''; state.category = ''; state.offset = 0;
  $('search').value = ''; $('category').value = '';
  const discover = view === 'discover';
  for (const [id, active] of [['nav-discover', discover], ['nav-apps', !discover]]) {
    $(id).classList.toggle('selected', active);
    if (active) $(id).setAttribute('aria-current', 'page'); else $(id).removeAttribute('aria-current');
  }
  $('breadcrumb').textContent = discover ? 'Discover' : 'My Apps';
  $('eyebrow').textContent = discover ? 'THE SELF-HOSTED COLLECTION' : 'YOUR PERSONAL WORKSPACE';
  $('page-title').textContent = discover ? 'Good software. Your space.' : 'Right where you left them.';
  $('intro').textContent = discover ? 'Discover independent apps. Bring your favorites closer to home.' : 'Your apps, their own windows. All within reach.';
  $('results-title').textContent = discover ? 'Explore the collection' : 'Your apps';
  $('search').placeholder = discover ? 'Search apps, ideas, and tools…' : 'Search your apps…';
  for (const id of ['discovery-note', 'category-wrap', 'featured-shelf']) $(id).hidden = !discover;
  $('catalog-note').textContent = discover ? 'Independent software. A little closer to home.' : 'Your app registry is saved on this computer.';
  $('content').replaceChildren();
  window.scrollTo({ top: 0, behavior: 'instant' });
  render();
}
function openConnect(name = '') {
  $('connect-form').reset();
  $('connect-name').value = name;
  $('connect-error').textContent = '';
  showDialog($('connect-dialog'));
  (name ? $('connect-url') : $('connect-name')).focus();
}
async function reviewInstall(recipeId) {
  const token = ++recipeRequest;
  activeRecipe = null;
  $('install-title').textContent = 'Review installation';
  $('install-content').innerHTML = '<div class="loading" role="status">Checking Docker and recipe…</div>';
  $('install-error').textContent = '';
  $('install-confirm').textContent = 'Checking system…';
  $('install-confirm').disabled = true;
  showDialog($('install-dialog'));
  try {
    const [recipe, report] = await Promise.all([invoke('recipe_details', { id: recipeId }), invoke('doctor')]);
    if (token !== recipeRequest || !$('install-dialog').open) return;
    activeRecipe = recipe;
    $('install-title').textContent = `Review ${recipe.display_name} installation`;
    $('install-confirm').textContent = `Install ${recipe.display_name}`;
    $('install-confirm').disabled = !report.ready;
    $('install-content').innerHTML = recipeView(recipe, report);
    if (!report.ready) $('install-error').textContent = 'Start Docker Desktop and make sure Docker Compose is available, then reopen this review.';
  } catch (error) {
    if (token !== recipeRequest || !$('install-dialog').open) return;
    $('install-content').innerHTML = '';
    $('install-error').textContent = message(error);
    $('install-confirm').textContent = 'Install unavailable';
  }
}
function showDetail(app) {
  $('detail-content').innerHTML = detail(app);
  if (app.capability === 'verified_install') {
    const connect = document.createElement('button');
    connect.id = 'detail-connect';
    connect.className = 'text-button detail-connect';
    connect.textContent = 'Already running it? Connect an instance';
    $('detail-content').append(connect);
  }
  showDialog($('detail-dialog'));
  $('detail-primary').onclick = async () => {
    await closeDialog($('detail-dialog'));
    app.capability === 'verified_install' ? reviewInstall(app.recipe_id) : openConnect(app.name);
  };
  $('detail-connect')?.addEventListener('click', async () => { await closeDialog($('detail-dialog')); openConnect(app.name); });
  $('open-source').onclick = async () => {
    try { await invoke('open_project', { url: app.source_url }); }
    catch (error) { $('detail-error').textContent = message(error); }
  };
}
async function runAppAction(button, command, app, success) {
  const old = button.innerHTML, target = $(`app-error-${state.visibleApps.indexOf(app)}`);
  button.disabled = true;
  button.textContent = command === 'start_app' ? 'Starting…' : 'Stopping…';
  if (target) target.textContent = '';
  try { await invoke(command, { id: app.id }); await refreshApps(); await render(); toast(success); }
  catch (error) { if (target) target.textContent = message(error); }
  finally { if (button.isConnected) { button.disabled = false; button.innerHTML = old; } }
}
$('nav-discover').onclick = () => navigate('discover');
$('nav-apps').onclick = async () => { await refreshApps(); navigate('apps'); };
document.querySelector('.brand').onclick = event => { event.preventDefault(); navigate('discover'); };
$('connect-top').onclick = $('connect-note').onclick = () => openConnect();
$('about').onclick = () => showDialog($('about-dialog'));
$('search').addEventListener('input', () => {
  clearTimeout(searchTimer); ++request; state.query = $('search').value; state.offset = 0;
  searchTimer = setTimeout(render, 180);
});
$('category').onchange = () => { clearTimeout(searchTimer); state.category = $('category').value; state.offset = 0; render(); };
$('previous').onclick = () => { state.offset = Math.max(0, state.offset - state.limit); render(); };
$('next').onclick = () => { state.offset += state.limit; render(); };
document.addEventListener('keydown', event => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k' && !document.querySelector('dialog[open]')) { event.preventDefault(); $('search').focus(); }
});
document.addEventListener('error', event => { if (event.target.matches?.('.app-avatar img')) event.target.remove(); }, true);
document.addEventListener('click', async event => {
  const button = event.target.closest('button');
  if (!button) return;
  if (button.dataset.close) { await closeDialog($(button.dataset.close)); return; }
  if (button.dataset.featured) { reviewInstall(button.dataset.featured); return; }
  if (button.dataset.detail !== undefined) { showDetail(state.entries[Number(button.dataset.detail)]); return; }
  if (button.dataset.action === 'connect') openConnect();
  if (button.dataset.action === 'discover') navigate('discover');
  if (button.dataset.action === 'clear') {
    clearTimeout(searchTimer);
    $('search').value = ''; $('category').value = ''; state.query = ''; state.category = ''; state.offset = 0; render();
  }
  if (button.dataset.action === 'retry') { await refreshApps(); render(); }
  const keyed = ['open', 'shortcut', 'start', 'stop', 'logs', 'remove', 'uninstall'].find(key => button.dataset[key] !== undefined);
  if (!keyed) return;
  const app = state.visibleApps[Number(button.dataset[keyed])];
  if (keyed === 'remove') {
    pendingApp = app; $('remove-description').textContent = `${app.display_name} will be removed from My Apps.`;
    $('remove-error').textContent = ''; showDialog($('remove-dialog')); return;
  }
  if (keyed === 'uninstall') {
    pendingApp = app; $('delete-data').checked = false; $('uninstall-confirm').textContent = 'Uninstall, keep data';
    $('uninstall-description').textContent = `${app.display_name} and its container will be removed. Its data is preserved by default.`;
    $('uninstall-error').textContent = ''; showDialog($('uninstall-dialog')); return;
  }
  if (keyed === 'logs') {
    $('logs-title').textContent = `${app.display_name} logs`; $('logs-content').textContent = 'Loading…'; $('logs-error').textContent = '';
    showDialog($('logs-dialog'));
    try { $('logs-content').textContent = await invoke('app_logs', { id: app.id }) || 'No recent output.'; }
    catch (error) { $('logs-content').textContent = ''; $('logs-error').textContent = message(error); }
    return;
  }
  if (keyed === 'start' || keyed === 'stop') { await runAppAction(button, keyed === 'start' ? 'start_app' : 'stop_app', app, `${app.display_name} ${keyed === 'start' ? 'started' : 'stopped'}.`); return; }
  const old = button.innerHTML;
  button.disabled = true; button.textContent = keyed === 'open' ? 'Opening…' : 'Creating…';
  try { await invoke(keyed === 'open' ? 'open_app' : 'create_shortcut', { id: app.id }); toast(keyed === 'open' ? `${app.display_name} opened.` : `Shortcut created for ${app.display_name}.`); }
  catch (error) { $(`app-error-${state.visibleApps.indexOf(app)}`).textContent = message(error); }
  finally { button.disabled = false; button.innerHTML = old; }
});
$('connect-form').onsubmit = async event => {
  event.preventDefault();
  const name = $('connect-name').value.trim(), rawUrl = $('connect-url').value.trim();
  try {
    const url = new URL(rawUrl);
    if (!/^https?:\/\//i.test(rawUrl) || !['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password || /\s/.test(rawUrl)) throw new Error('Enter an http:// or https:// address without embedded credentials.');
    if (!name) throw new Error('Enter a name for this connection.');
    $('connect-submit').disabled = true; $('connect-submit').textContent = 'Saving…'; $('connect-error').textContent = '';
    setDialogBusy($('connect-dialog'), true);
    await invoke('add_app', { name, url: rawUrl });
    setDialogBusy($('connect-dialog'), false);
    await closeDialog($('connect-dialog'));
    await refreshApps(); navigate('apps'); toast(`${name} added to My Apps.`);
  } catch (error) { $('connect-error').textContent = message(error); }
  finally { setDialogBusy($('connect-dialog'), false); $('connect-submit').disabled = false; $('connect-submit').textContent = 'Add to My Apps →'; }
};
$('install-confirm').onclick = async () => {
  if (!activeRecipe) return;
  const recipe = activeRecipe;
  $('install-confirm').disabled = true; $('install-confirm').textContent = 'Installing…'; $('install-error').textContent = '';
  setDialogBusy($('install-dialog'), true);
  try {
    await invoke('install_app', { recipeId: recipe.id });
    setDialogBusy($('install-dialog'), false);
    await closeDialog($('install-dialog'));
    await refreshApps(); navigate('apps'); toast(`${recipe.display_name} installed and ready.`);
  } catch (error) { $('install-error').textContent = message(error); }
  finally { setDialogBusy($('install-dialog'), false); $('install-confirm').disabled = false; $('install-confirm').textContent = `Install ${recipe.display_name}`; }
};
$('remove-confirm').onclick = async () => {
  const app = pendingApp;
  $('remove-confirm').disabled = true; setDialogBusy($('remove-dialog'), true);
  try {
    await invoke('remove_app_cmd', { id: app.id });
    setDialogBusy($('remove-dialog'), false); await closeDialog($('remove-dialog'));
    await refreshApps(); render(); toast('Connection removed. App data is unchanged.');
  } catch (error) { $('remove-error').textContent = message(error); }
  finally { $('remove-confirm').disabled = false; setDialogBusy($('remove-dialog'), false); }
};
$('delete-data').onchange = () => { $('uninstall-confirm').textContent = $('delete-data').checked ? 'Uninstall and delete data' : 'Uninstall, keep data'; };
$('uninstall-confirm').onclick = async () => {
  const app = pendingApp, deleteData = $('delete-data').checked;
  if (deleteData && !window.confirm('Permanently delete this app’s managed data? This cannot be undone.')) return;
  $('uninstall-confirm').disabled = true; setDialogBusy($('uninstall-dialog'), true);
  try {
    await invoke('uninstall_app', { id: app.id, deleteData });
    setDialogBusy($('uninstall-dialog'), false); await closeDialog($('uninstall-dialog'));
    await refreshApps(); render(); toast(deleteData ? 'App and managed data deleted.' : 'App uninstalled. Managed data was preserved.');
  } catch (error) { $('uninstall-error').textContent = message(error); }
  finally { $('uninstall-confirm').disabled = false; setDialogBusy($('uninstall-dialog'), false); }
};
await refreshApps();
render();
