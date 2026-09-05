// The only IPC boundary. Browser tests inject a controlled __TAURI__ adapter.
export function invoke(command, args = {}) {
  if (!window.__TAURI__?.core?.invoke) {
    return Promise.reject(new Error('Open Local Store on your desktop to use this workspace.'));
  }
  return window.__TAURI__.core.invoke(command, args);
}
