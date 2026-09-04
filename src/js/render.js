export const escapeHtml = value => String(value ?? '').replace(/[&<>"']/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
export const icon = name => `<img src="assets/icons/${name}.svg" alt="">`;
export function plainDescription(value) {
  return String(value ?? '').replace(/\[([^\]]+)\]\(https?:[^)]+\)/g, '$1').replace(/[`*_]/g, '').trim();
}
export function avatar(name, url) {
  const initials = name.trim().split(/\s+/).slice(0, 2).map(word => [...word][0] || '').join('').toUpperCase();
  // Local file paths and non-HTTPS schemes are not sent to an image element.
  const source = url && /^https:\/\//i.test(url) ? url : null;
  return `<span class="app-avatar" aria-hidden="true">${escapeHtml(initials)}${source ? `<img src="${escapeHtml(source)}" alt="" loading="lazy" referrerpolicy="no-referrer">` : ''}</span>`;
}
export function discoveryCard(app, index) {
  return `<article class="app-card"><div class="card-top">${avatar(app.name, app.icon)}<div class="card-heading"><h3>${escapeHtml(app.name)}</h3><span class="card-category">${escapeHtml(app.category)}</span></div></div><p class="card-description">${escapeHtml(plainDescription(app.description))}</p><div class="card-bottom"><span class="capability">${icon('link')}Connect existing</span><button class="card-detail" data-detail="${index}" aria-label="View ${escapeHtml(app.name)} details">View app <span aria-hidden="true">↗</span></button></div>${app.warning ? '<p class="warning-label">Catalog caution — review project details</p>' : ''}</article>`;
}
export function installedRow(app, index) {
  return `<article class="installed-app">${avatar(app.name, app.icon)}<div class="installed-info"><h3>${escapeHtml(app.name)}</h3><p>${escapeHtml(app.url)}</p><div class="installed-type">${app.compose ? 'Existing Compose connection' : 'Connected instance'}</div><p class="inline-error" id="app-error-${index}" role="alert"></p></div><div class="installed-actions"><button class="secondary" data-shortcut="${index}">Shortcut</button><button class="primary" data-open="${index}">Open ${icon('arrow-up-right')}</button><button class="icon-button" data-remove="${index}" aria-label="Remove ${escapeHtml(app.name)}">${icon('ellipsis')}</button></div></article>`;
}
export function emptyState(title, text, action, label, glyph = 'layout-grid') {
  return `<div class="empty-state">${icon(glyph)}<h3>${escapeHtml(title)}</h3><p>${escapeHtml(text)}</p><button class="primary" data-action="${action}">${escapeHtml(label)}</button></div>`;
}
export function detail(app) {
  return `<div class="modal-top"><p class="eyebrow">FROM THE COLLECTION</p><button class="icon-button" data-close="detail-dialog" aria-label="Close app details">${icon('x')}</button></div><div class="detail-heading">${avatar(app.name, app.icon)}<div><h2 id="detail-title">${escapeHtml(app.name)}</h2><span class="card-category">${escapeHtml(app.category)}</span></div></div><p class="detail-description">${escapeHtml(plainDescription(app.description))}</p><dl class="detail-meta"><dt>Available action</dt><dd>Connect existing instance</dd><dt>License / tags</dt><dd>${escapeHtml(app.license || 'See project')}</dd><dt>Project</dt><dd><button class="text-button" id="open-source">Visit project website ${icon('external-link')}</button></dd></dl>${app.warning ? '<p class="warning-label">This project carries a caution in the source catalog. Review its documentation before using it.</p>' : ''}<div class="form-note">${icon('unplug')}<span>Already host this app? Connect its address. Automatic installation isn’t available in this release.</span></div><p id="detail-error" class="form-error" role="alert"></p><div class="modal-actions"><button class="secondary" data-close="detail-dialog">Back to collection</button><button class="primary" id="connect-detail">Connect ${escapeHtml(app.name)}</button></div>`;
}
