import test from 'node:test';
import assert from 'node:assert/strict';
import { invoke, CommandError, listenOperations, listenActivationFailures } from '../src/js/api.js';

test('IPC preserves structured error codes and readable messages', async () => {
  globalThis.window = { __TAURI__: { core: { invoke: async () => {
    throw { code: 'timed_out', message: 'Docker timed out.' };
  } } } };
  await assert.rejects(invoke('doctor'), error => error instanceof CommandError &&
    error.code === 'timed_out' && error.message === 'Docker timed out.');
});

test('IPC accepts legacy failures and catches synchronous adapter failures', async () => {
  for (const failure of ['Legacy failure', new Error('Legacy failure')]) {
    globalThis.window = { __TAURI__: { core: { invoke: () => { throw failure; } } } };
    await assert.rejects(invoke('start_app'), error => error.code === 'unknown' && error.message === 'Legacy failure');
  }
});

test('IPC returns successful payloads unchanged and explains browser-only access', async () => {
  const payload = { ready: true };
  globalThis.window = { __TAURI__: { core: { invoke: async (command, args) => {
    assert.equal(command, 'doctor'); assert.deepEqual(args, { id: 'memos' }); return payload;
  } } } };
  assert.equal(await invoke('doctor', { id: 'memos' }), payload);
  globalThis.window = {};
  await assert.rejects(invoke('doctor'), error => error.code === 'desktop_unavailable');
});

test('operation subscription uses the dedicated channel and can be removed', async () => {
  let callback, removed = false;
  globalThis.window = { __TAURI__: { event: { listen: async (name, handler) => {
    assert.equal(name, 'local-store://operation'); callback = handler;
    return () => { removed = true; };
  } } } };
  const received = [];
  const unlisten = await listenOperations(event => received.push(event));
  callback({ payload: { kind: 'unexpected' } });
  const payload = { operation_id: '1-2', app_id: 'memos', kind: 'start', state: 'failed',
    error: { code: 'timed_out', message: 'Docker timed out.' } };
  callback({ payload });
  assert.deepEqual(received, [payload]);
  unlisten(); assert.equal(removed, true);
  globalThis.window = {};
  assert.equal(typeof await listenOperations(() => {}), 'function');
});


test('native failures queued before startup are drained once and later wakeups reuse the queue', async () => {
  let wake, removed = false;
  let pending = [{code:'not_found', message:'App not found.'}];
  globalThis.window = {__TAURI__:{core:{invoke: async name => {
    assert.equal(name, 'take_activation_errors'); const values = pending; pending = []; return values;
  }}, event:{listen: async (name, handler) => {
    assert.equal(name, 'local-store://activation-failure'); wake = handler; return () => { removed = true; };
  }}}};
  const errors = [];
  const stop = await listenActivationFailures(error => errors.push(error.message));
  assert.deepEqual(errors, ['App not found.']);
  await wake({}); assert.equal(errors.length, 1);
  pending.push({code:'process_failed', message:'Docker failed.'});
  await wake({}); assert.deepEqual(errors, ['App not found.', 'Docker failed.']);
  stop(); assert.equal(removed, true);
});
