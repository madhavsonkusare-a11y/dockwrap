import test from 'node:test';
import assert from 'node:assert/strict';
import {createOperations, diagnosticText, canCancel, retryIsUnsafe} from '../src/js/operations.js';

test('IPC settles busy state without event delivery and prevents overlapping requests', () => {
  const state = createOperations();
  const token = state.begin('memos', 'start');
  assert.equal(state.begin('memos', 'stop'), null);
  state.finish('memos', token, {code: 'timed_out', message: 'Timeout'});
  assert.equal(state.get('memos').pending, false);
  assert.match(diagnosticText(state.get('memos').error), /status and logs/);
  const next = state.begin('memos', 'start');
  state.finish('memos', token); // An old request cannot clear its successor.
  assert.equal(state.get('memos').pending, true);
  state.finish('memos', next);
  assert.equal(state.get('memos'), undefined);
});

test('terminal events correlate but never settle a pending IPC request', () => {
  const state = createOperations();
  const token = state.begin('memos', 'start');
  state.receive({app_id: 'memos', kind: 'start', operation_id: 'a', state: 'started'});
  state.receive({app_id: 'memos', kind: 'start', operation_id: 'old', state: 'failed'});
  assert.equal(state.get('memos').finishing, false);
  state.receive({app_id: 'memos', kind: 'start', operation_id: 'a', state: 'succeeded'});
  assert.equal(state.get('memos').pending, true);
  assert.equal(state.get('memos').finishing, true);
  state.finish('memos', token);
  state.receive({app_id: 'memos', kind: 'start', operation_id: 'a', state: 'started'});
  assert.equal(state.get('memos'), undefined);
});


test('install stages correlate and cannot overwrite terminal feedback', () => {
  const state = createOperations();
  state.begin('memos', 'install');
  const event = {app_id:'memos', kind:'install', operation_id:'install-1'};
  state.receive({...event, state:'started'});
  state.receive({...event, state:'progress', stage:'waiting_for_health', operation_id:'other'});
  assert.equal(state.get('memos').stage, undefined);
  state.receive({...event, state:'progress', stage:'waiting_for_health'});
  assert.equal(state.get('memos').stage, 'waiting_for_health');
  assert.equal(state.get('memos').finishing, false);
  state.receive({...event, state:'failed'});
  state.receive({...event, state:'progress', stage:'preparing_files'});
  assert.equal(state.get('memos').stage, 'waiting_for_health');
});


test('a stop control is offered only while a cancellable install is running', () => {
  const state = createOperations();
  const event = {app_id: 'memos', kind: 'install', operation_id: 'install-1'};

  // No operation at all, and an operation whose events have not arrived yet.
  assert.equal(canCancel(undefined), false);
  state.begin('memos', 'install');
  assert.equal(canCancel(state.get('memos')), false, 'no operation id known yet');

  state.receive({...event, state: 'started', cancel_id: 7});
  assert.equal(state.get('memos').cancelId, 7);
  assert.equal(canCancel(state.get('memos')), true);

  state.receive({...event, state: 'progress', stage: 'starting_containers'});
  assert.equal(canCancel(state.get('memos')), true);

  // The registry commit is the backend's cutoff, so the offer stops there.
  state.receive({...event, state: 'progress', stage: 'saving_app'});
  assert.equal(canCancel(state.get('memos')), false, 'offered past the commit cutoff');
});

test('an install with no cancel id is never offered a stop control', () => {
  const state = createOperations();
  state.begin('memos', 'install');
  state.receive({app_id: 'memos', kind: 'install', operation_id: 'install-1', state: 'started'});
  assert.equal(state.get('memos').cancelId, null);
  assert.equal(canCancel(state.get('memos')), false);
});

test('requesting a stop withdraws the offer but leaves IPC to settle the install', () => {
  const state = createOperations();
  const token = state.begin('memos', 'install');
  const event = {app_id: 'memos', kind: 'install', operation_id: 'install-1'};
  state.receive({...event, state: 'started', cancel_id: 12});
  state.cancelRequested('memos');
  assert.equal(canCancel(state.get('memos')), false, 'a second stop was still offered');
  assert.equal(state.get('memos').pending, true, 'the request was settled by the click');
  // Only the IPC result clears it.
  state.finish('memos', token, {code: 'cancelled', message: 'Setup was cancelled.'});
  assert.equal(state.get('memos').pending, false);
  assert.equal(state.get('memos').error.code, 'cancelled');
});

test('a terminal event withdraws the stop offer while IPC settles', () => {
  const state = createOperations();
  state.begin('memos', 'install');
  const event = {app_id: 'memos', kind: 'install', operation_id: 'install-1'};
  state.receive({...event, state: 'started', cancel_id: 3});
  state.receive({...event, state: 'cancelled'});
  assert.equal(state.get('memos').finishing, true);
  assert.equal(canCancel(state.get('memos')), false);
});


test('only a failed cleanup makes a retry unsafe', () => {
  assert.equal(retryIsUnsafe({code: 'rollback_failed'}), true);
  // Everything else rolled back cleanly, so trying again starts from nothing.
  for (const code of ['timed_out', 'port_in_use', 'cancelled', 'prerequisite_unavailable', 'storage_io']) {
    assert.equal(retryIsUnsafe({code}), false, code);
  }
  assert.equal(retryIsUnsafe(undefined), false);
  assert.equal(retryIsUnsafe(null), false);
});

test('recovery guidance covers the outcomes a user has to act on', () => {
  assert.match(diagnosticText({code: 'rollback_failed', message: 'Setup failed.'}), /containers or files may remain/);
  assert.match(diagnosticText({code: 'cancelled', message: 'Setup was cancelled.'}), /Nothing was added/);
  assert.match(diagnosticText({code: 'already_exists', message: 'Memos is already in My Apps.'}), /Open My Apps/);
  // An unknown code still shows the message rather than swallowing it.
  assert.equal(diagnosticText({code: 'something_new', message: 'Odd failure.'}), 'Odd failure.');
});
