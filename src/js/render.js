export const escapeHtml = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export const icon = name => `<img src="assets/icons/${name}.svg" alt="">`;
export function plainDescription(value) {
  return String(value ?? '').replace(/\[([^\]]+)\]\(https?:[^)]+\)/g, '$1').replace(/[`*_]/g, '').trim();
}
export function avatar(name, url) {
  const initials = name.trim().split(/\s+/).slice(0, 2).map(word => [...word][0] || '').join('').toUpperCase();
  const localIcons = { memos: 'memos', n8n: 'n8n', 'uptime kuma': 'uptime-kuma', immich: 'immich', 'actual budget': 'actual-budget' };
  const localIcon = Object.hasOwn(localIcons, name.toLowerCase()) && localIcons[name.toLowerCase()];
  const source = localIcon ? `assets/apps/${localIcon}.svg` : url && /^https:\/\//i.test(url) ? url : null;
  return `<span class="app-avatar" aria-hidden="true">${escapeHtml(initials)}${source ? `<img src="${escapeHtml(source)}" alt="" loading="lazy" referrerpolicy="no-referrer">` : ''}</span>`;
}
export function discoveryCard(app, index) {
  const verified = app.capability === 'verified_install';
  return `<article class="app-card${verified ? ' verified-card' : ''}"><div class="card-top">${avatar(app.name, app.icon)}<div class="card-heading"><h3>${escapeHtml(app.name)}</h3><span class="card-category">${escapeHtml(app.category)}</span></div></div><p class="card-description">${escapeHtml(plainDescription(app.description))}</p><div class="card-bottom"><span class="capability ${verified ? 'verified' : ''}">${icon(verified ? 'hard-drive' : 'link')}${verified ? 'Verified install' : 'Connect existing'}</span><button class="card-detail" data-detail="${index}" aria-label="View ${escapeHtml(app.name)} details">View app <span aria-hidden="true">↗</span></button></div>${app.warning ? '<p class="warning-label">Catalog caution — review project details</p>' : ''}</article>`;
}
export function installedRow(app, index) {
  const managed = app.runtime?.kind === 'compose';
  const running = app.status === 'running';
  const status = managed ? app.status : 'connected';
  const controls = managed
    ? `${running ? `<button class="primary" data-open="${index}">Open ${icon('arrow-up-right')}</button><button class="secondary" data-stop="${index}">Stop</button>` : `<button class="primary" data-start="${index}">Start</button>`}<button class="secondary" data-logs="${index}">Logs</button><button class="icon-button" data-uninstall="${index}" aria-label="Uninstall ${escapeHtml(app.display_name)}">${icon('ellipsis')}</button>`
    : `<button class="secondary" data-shortcut="${index}">Shortcut</button><button class="primary" data-open="${index}">Open ${icon('arrow-up-right')}</button><button class="icon-button" data-remove="${index}" aria-label="Remove ${escapeHtml(app.display_name)}">${icon('ellipsis')}</button>`;
  return `<article class="installed-app">${avatar(app.display_name, app.icon_path)}<div class="installed-info"><div class="installed-title"><h3>${escapeHtml(app.display_name)}</h3><span class="status status-${escapeHtml(status)}">${escapeHtml(status)}</span></div><p>${escapeHtml(app.launch_url)}</p><div class="installed-type">${managed ? 'Managed by Local Store' : 'Connected instance'}</div><p class="inline-error" id="app-error-${index}" role="alert"></p></div><div class="installed-actions">${controls}</div></article>`;
}
export function emptyState(title, text, action, label, glyph = 'layout-grid') {
  return `<div class="empty-state">${icon(glyph)}<h3>${escapeHtml(title)}</h3><p>${escapeHtml(text)}</p><button class="primary" data-action="${action}">${escapeHtml(label)}</button></div>`;
}
export function detail(app) {
  const verified = app.capability === 'verified_install';
  return `<div class="modal-top"><p class="eyebrow">FROM THE COLLECTION</p><button class="icon-button" data-close="detail-dialog" aria-label="Close app details">${icon('x')}</button></div><div class="detail-heading">${avatar(app.name, app.icon)}<div><h2 id="detail-title">${escapeHtml(app.name)}</h2><span class="card-category">${escapeHtml(app.category)}</span></div></div><p class="detail-description">${escapeHtml(plainDescription(app.description))}</p><dl class="detail-meta"><dt>Available action</dt><dd>${verified ? 'Verified local install' : 'Connect existing instance'}</dd><dt>License / tags</dt><dd>${escapeHtml(app.license || 'See project')}</dd><dt>Project</dt><dd><button class="text-button" id="open-source">Visit project website ${icon('external-link')}</button></dd></dl>${app.warning ? '<p class="warning-label">This project carries a caution in the source catalog. Review its documentation before using it.</p>' : ''}<div class="form-note">${icon(verified ? 'hard-drive' : 'unplug')}<span>${verified ? 'Local Store reviewed and pinned this recipe. You can inspect every setting before Docker starts.' : 'Already host this app? Connect its address. Automatic installation is available only for reviewed recipes.'}</span></div><p id="detail-error" class="form-error" role="alert"></p><div class="modal-actions"><button class="secondary" data-close="detail-dialog">Back to collection</button><button class="primary" id="detail-primary">${verified ? 'Review install' : `Connect ${escapeHtml(app.name)}`}</button></div>`;
}
export function recipeView(recipe, report) {
  const checks = report.checks.map(check => `<li class="doctor-check ${check.ok ? 'check-ok' : 'check-fail'}"><strong>${escapeHtml(check.label)}</strong><span>${escapeHtml(check.ok ? check.detail || 'Ready' : check.detail)}</span></li>`).join('');
  return `<div class="recipe-summary"><div><span>Version</span><strong>${escapeHtml(recipe.version)}</strong></div><div><span>Container</span><strong>${escapeHtml(recipe.image)}</strong></div><div><span>Address</span><strong>${escapeHtml(recipe.launch_url)}</strong></div><div><span>Data storage</span><strong>${escapeHtml(recipe.data_storage)}</strong></div></div><h3 class="section-label">System check</h3><ul class="doctor-list">${checks}</ul><h3 class="section-label">What this changes</h3><ul class="risk-list">${recipe.risk_notes.map(note => `<li>${escapeHtml(note)}</li>`).join('')}</ul>`;
}
