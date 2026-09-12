import {invoke} from './api.js';

const button = document.getElementById('scan-recovery');
const output = document.getElementById('recovery-output');
const error = document.getElementById('recovery-error');
const dialog = document.getElementById('settings-dialog');
let request = 0;
const reset = () => { button.disabled = false; button.textContent = 'Scan for setups'; };
dialog.addEventListener('close', () => { request++; reset(); });
button.addEventListener('click', async () => {
  const token = ++request;
  button.disabled = true; button.textContent = 'Scanning…';
  output.replaceChildren(); error.textContent = '';
  try {
    const candidates = await invoke('inspect_recovery');
    if (token !== request || !dialog.open) return;
    if (!candidates.length) {
      output.textContent = 'No retained setups found for supported apps.';
      return;
    }
    for (const candidate of candidates) {
      const row = document.createElement('div'); row.className = 'settings-row';
      const content = document.createElement('div');
      const title = document.createElement('h3'); title.textContent = candidate.display_name;
      const description = document.createElement('p');
      description.textContent = candidate.ownership_status === 'verified'
        ? 'Matching Docker setup found. Review the retained setup files before manual recovery. Automatic repair is not yet supported.'
        : candidate.ownership_status === 'no_containers'
          ? 'Retained setup files found, with no matching container. These may be data you chose to keep.'
          : 'This setup could not be verified. Leave it unchanged until its ownership is confirmed.';
      const details = document.createElement('details');
      const summary = document.createElement('summary'); summary.textContent = 'Setup location';
      const path = document.createElement('p'); path.textContent = candidate.compose_file;
      path.style.overflowWrap = 'anywhere';
      details.append(summary, path); content.append(title, description, details);
      row.append(content); output.append(row);
    }
  } catch (failure) {
    if (token === request && dialog.open) error.textContent = failure.message || 'Could not inspect setups. Check Docker and try again.';
  } finally { if (token === request) reset(); }
});
