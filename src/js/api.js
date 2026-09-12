// The only IPC boundary. Browser tests inject a controlled __TAURI__ adapter.
export class CommandError extends Error {
  constructor(error) {
    super(typeof error?.message === 'string' ? error.message :
      typeof error === 'string' ? error : 'The operation could not be completed.');
    this.name = 'CommandError';
    this.code = typeof error?.code === 'string' ? error.code : 'unknown';
  }
}

export async function invoke(command, args = {}) {
  if (!window.__TAURI__?.core?.invoke) {
    throw new CommandError({code: 'desktop_unavailable', message: 'Open Local Store on your desktop to use this workspace.'});
  }
  try {
    return await window.__TAURI__.core.invoke(command, args);
  } catch (error) {
    throw new CommandError(error);
  }
}

// Subscribe before invoking a lifecycle command. The returned function removes
// the listener; IPC completion remains authoritative if event delivery fails.
export async function listenOperations(onOperation) {
  if (!window.__TAURI__?.event?.listen) return () => {};
  try {
    return await window.__TAURI__.event.listen('local-store://operation', event => {
      const operation = event?.payload;
      if (typeof operation?.operation_id !== 'string' || typeof operation.app_id !== 'string' ||
          !['install', 'start', 'stop', 'uninstall', 'open'].includes(operation.kind) ||
          !['started', 'progress', 'succeeded', 'failed', 'cancelled'].includes(operation.state)) return;
      onOperation(operation);
    });
  } catch (error) {
    throw new CommandError(error);
  }
}
// A link that could not be handed to the browser fails inside an app window,
// which owns none of our UI. The launcher reports it instead.
export async function listenBrowserFailures(onFailure) {
  if (!window.__TAURI__?.event?.listen) return () => {};
  try {
    return await window.__TAURI__.event.listen('local-store://browser-failure', event => {
      const error = event?.payload;
      if (typeof error?.message !== 'string') return;
      onFailure(error);
    });
  } catch (error) {
    throw new CommandError(error);
  }
}

// Wakeups and startup both drain the same queue; each error is shown once.
export async function listenActivationFailures(onFailure) {
  if (!window.__TAURI__?.event?.listen) return () => {};
  const drain = async () => {
    const errors = await invoke('take_activation_errors');
    if (Array.isArray(errors)) errors.forEach(error => { if (typeof error?.message === 'string') onFailure(error); });
  };
  const unlisten = await window.__TAURI__.event.listen('local-store://activation-failure', () => drain().catch(() => {}));
  try { await drain(); } catch (error) { unlisten(); throw error; }
  return unlisten;
}
