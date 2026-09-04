import { invoke } from './api.js';
import { discoveryCard, installedRow, emptyState, detail } from './render.js';
const $ = id => document.getElementById(id);
const state = { view: 'discover', query: '', category: '', offset: 0, limit: 12, entries: [], apps: [], visibleApps: [], total: 0 };
let request = 0, searchTimer, toastTimer, selectedApp, removeApp, refreshError;
const message = error => error?.message || String(error);
function toast(text) {
  clearTimeout(toastTimer); $('toast').textContent = text; $('toast').hidden = false;
  toastTimer = setTimeout(() => { $('toast').hidden = true; }, 6000);
}
function loadState(loading) { $('content').setAttribute('aria-busy', String(loading)); }
async function refreshApps() {
  try { state.apps = await invoke('list_apps'); refreshError = null; $('app-count').textContent = state.apps.length; }
  catch (error) { refreshError = error; $('app-count').textContent = '–'; }
}
async function render() {
  const token = ++request;
  $('pagination').hidden = true;
  if (state.view === 'apps') {
    loadState(false);
    if (refreshError) { showError(refreshError); return; }
    state.visibleApps = state.apps.filter(a => `${a.name} ${a.url}`.toLowerCase().includes(state.query.toLowerCase()));
    $('results-count').textContent = `${state.visibleApps.length} connections`;
    $('content').innerHTML = state.visibleApps.length ? `<div class="installed-list">${state.visibleApps.map(installedRow).join('')}</div>` : emptyState(state.query ? 'No matching connections' : 'Your apps belong here.', state.query ? 'Try another name or address.' : 'Connect an app you already run. It will be waiting here the next time you need it.', state.query ? 'clear' : 'connect', state.query ? 'Clear search' : 'Connect your first app');
    return;
  }
  loadState(true);
  $('content').innerHTML = '<div class="loading" role="status">Finding your next app…</div>';
  $('results-count').textContent = '';
  try {
    const page = await invoke('search_catalog', { query: state.query, category: state.category, offset: state.offset, limit: state.limit });
    if (token !== request) return;
    state.entries = page.entries; state.total = page.total;
    const select = $('category');
    if (select.options.length === 1) page.categories.forEach(category => select.add(new Option(category, category)));
    select.value = state.category;
    $('results-count').textContent = `${page.total.toLocaleString()} ${page.total === 1 ? 'project' : 'projects'}`;
    $('catalog-note').textContent = `${page.catalog_total.toLocaleString()} projects to discover · Connect your own instance`;
    $('content').innerHTML = page.entries.length ? `<div class="app-grid">${page.entries.map(discoveryCard).join('')}</div>` : emptyState('Nothing here just yet.', 'Try a different search or category. You can also connect an app that isn’t in this collection.', 'clear', 'Clear filters', 'search');
    $('pagination').hidden = page.total <= state.limit;
    $('page-label').textContent = `${state.offset + 1}–${Math.min(state.offset + state.limit, page.total)} of ${page.total.toLocaleString()} projects`;
    $('previous').disabled = state.offset === 0;
    $('next').disabled = state.offset + state.limit >= page.total;
  } catch (error) { if (token === request) showError(error); }
  finally { if (token === request) loadState(false); }
}
function showError(error) {
  $('content').innerHTML = emptyState('We couldn’t load your workspace.', message(error), 'retry', 'Try again', 'circle-alert');
}
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
  $('intro').textContent = discover ? 'Find your next self-hosted app. Give the ones you run a place on your desktop.' : 'Your connected apps, ready to open in their own desktop windows.';
  $('results-title').textContent = discover ? 'Explore the collection' : 'Your connections';
  $('search').placeholder = discover ? 'Search apps, ideas, and tools…' : 'Search your apps…';
  $('discovery-note').hidden = !discover; $('category-wrap').hidden = !discover;
  $('catalog-note').textContent = discover ? 'Independent software. A little closer to home.' : 'Your connections are saved on this computer.';
  render();
}
function openConnect(name = '') {
  $('connect-form').reset(); $('connect-name').value = name; $('connect-error').textContent = '';
  $('connect-dialog').showModal(); (name ? $('connect-url') : $('connect-name')).focus();
}
$('nav-discover').onclick = () => navigate('discover');
$('nav-apps').onclick = async () => { await refreshApps(); navigate('apps'); };
document.querySelector('.brand').onclick = event => { event.preventDefault(); navigate('discover'); };
$('connect-top').onclick = $('connect-note').onclick = () => openConnect();
$('about').onclick = () => $('about-dialog').showModal();
$('search').addEventListener('input', () => {
  clearTimeout(searchTimer); ++request;
  state.query = $('search').value; state.offset = 0;
  searchTimer = setTimeout(render, 180);
});
$('category').onchange = () => { clearTimeout(searchTimer); state.category = $('category').value; state.offset = 0; render(); };
$('previous').onclick = () => { state.offset = Math.max(0, state.offset - state.limit); render(); };
$('next').onclick = () => { state.offset += state.limit; render(); };
document.addEventListener('keydown', event => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'k' && !document.querySelector('dialog[open]')) {
    event.preventDefault(); $('search').focus();
  }
});
document.addEventListener('error', event => { if (event.target.matches?.('.app-avatar img')) event.target.remove(); }, true);
document.addEventListener('click', async event => {
  const button = event.target.closest('button'); if (!button) return;
  if (button.dataset.close) { $(button.dataset.close).close(); return; }
  if (button.dataset.detail !== undefined) {
    selectedApp = state.entries[Number(button.dataset.detail)];
    $('detail-content').innerHTML = detail(selectedApp); $('detail-dialog').showModal();
    $('connect-detail').onclick = () => { $('detail-dialog').close(); openConnect(selectedApp.name); };
    $('open-source').onclick = async () => { try { await invoke('open_project', { url: selectedApp.source_url }); } catch (error) { $('detail-error').textContent = message(error); } };
    return;
  }
  if (button.dataset.action === 'connect') openConnect();
  if (button.dataset.action === 'clear') { $('search').value = ''; $('category').value = ''; state.query = ''; state.category = ''; state.offset = 0; render(); }
  if (button.dataset.action === 'retry') { await refreshApps(); render(); }
  if (button.dataset.remove !== undefined) {
    removeApp = state.visibleApps[Number(button.dataset.remove)];
    $('remove-description').textContent = `${removeApp.name} will be removed from My Apps.`;
    $('remove-error').textContent = ''; $('remove-dialog').showModal();
  }
  const key = button.dataset.open !== undefined ? 'open' : button.dataset.shortcut !== undefined ? 'shortcut' : null;
  if (key) {
    const index = Number(button.dataset[key]), app = state.visibleApps[index];
    const oldText = button.innerHTML; button.disabled = true; button.textContent = key === 'open' ? 'Opening…' : 'Creating…';
    $(`app-error-${index}`).textContent = '';
    try {
      await invoke(key === 'open' ? 'open_app' : 'create_shortcut', { name: app.name });
      toast(key === 'open' ? `${app.name} opened.` : `Shortcut created for ${app.name}.`);
    } catch (error) { const target = $(`app-error-${index}`); if (target) target.textContent = message(error); }
    finally { button.disabled = false; button.innerHTML = oldText; }
  }
});
$('connect-form').onsubmit = async event => {
  event.preventDefault();
  const name = $('connect-name').value.trim(), rawUrl = $('connect-url').value.trim();
  try {
    const url = new URL(rawUrl);
    if (!/^https?:\/\//i.test(rawUrl) || !['http:', 'https:'].includes(url.protocol) || !url.hostname || url.username || url.password || /\s/.test(rawUrl)) throw new Error('Enter an http:// or https:// address without embedded credentials.');
    if (!name) throw new Error('Enter a name for this connection.');
    $('connect-submit').disabled = true; $('connect-submit').textContent = 'Saving…';
    $('connect-error').textContent = '';
    await invoke('add_app', { name, url: rawUrl });
    $('connect-dialog').close(); await refreshApps(); navigate('apps'); toast(`${name} added to My Apps.`);
  } catch (error) { $('connect-error').textContent = message(error); }
  finally { $('connect-submit').disabled = false; $('connect-submit').textContent = 'Add to My Apps →'; }
};
$('remove-confirm').onclick = async () => {
  $('remove-confirm').disabled = true;
  try { await invoke('remove_app_cmd', { name: removeApp.name }); $('remove-dialog').close(); await refreshApps(); render(); toast('Connection removed. App data is unchanged.'); }
  catch (error) { $('remove-error').textContent = message(error); }
  finally { $('remove-confirm').disabled = false; }
};
await refreshApps();
render();
