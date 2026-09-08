import test from 'node:test';
import assert from 'node:assert/strict';
import {createReadinessMonitor} from '../src/js/readiness.js';

const app = (id = 'notes', extra = {}) => ({id, launch_url:`http://localhost/${id}`,
  status:'running', runtime:{kind:'compose'}, ...extra});
const flush = () => new Promise(resolve => setImmediate(resolve));
function harness(probe, changed = () => {}) {
  const timers = new Map();
  let id = 0;
  const monitor = createReadinessMonitor({probe, changed,
    schedule(callback, delay) { assert.equal(delay, 15000); timers.set(++id, callback); return id; },
    cancel: id => timers.delete(id),
  });
  return {monitor, timers, async tick() {
    assert.equal(timers.size, 1);
    const [id, callback] = timers.entries().next().value;
    timers.delete(id); callback(); await flush();
  }};
}

test('readiness refreshes quietly, retaining only known answers', async () => {
  let answer = 'ready', changes = 0;
  const {monitor, tick} = harness(async () => answer, () => changes++);
  monitor.update([app()]); await flush();
  assert.equal(monitor.get('notes'), 'ready');
  await tick(); assert.equal(changes, 1); // Identical replies never repaint.
  answer = 'unreachable'; await tick();
  assert.equal(monitor.get('notes'), 'unreachable');
  answer = 'unknown'; await tick();
  assert.equal(monitor.get('notes'), undefined);
  monitor.dispose();
});

test('one failed probe clears only its own readiness and keeps polling', async () => {
  let fail = false;
  const {monitor, tick} = harness(async selected => {
    if (fail && selected.id === 'notes') throw new Error('IPC unavailable');
    return 'ready';
  });
  monitor.update([app(), app('other')]); await flush();
  fail = true; await tick();
  assert.equal(monitor.get('notes'), undefined);
  assert.equal(monitor.get('other'), 'ready');
  fail = false; await tick();
  assert.equal(monitor.get('notes'), 'ready');
  monitor.dispose();
});

test('stopped and busy managed apps are skipped, external connections are checked', async () => {
  const seen = [];
  const {monitor} = harness(async selected => { seen.push(selected.id); return 'ready'; });
  monitor.update([app('stopped', {status:'stopped'}), app('busy', {busy:true}),
    app('external', {status:'connected', runtime:{kind:'external'}})]);
  await flush(); assert.deepEqual(seen, ['external']);
  monitor.dispose();
});

test('a stopped operation or removed app cannot receive a stale ready result', async () => {
  for (const next of [[], [app('notes', {busy:true})], [app('notes', {status:'stopped'})]]) {
    let resolve;
    const {monitor, timers} = harness(() => new Promise(done => { resolve = done; }));
    monitor.update([app()]);
    monitor.update(next);
    resolve('ready'); await flush();
    assert.equal(monitor.get('notes'), undefined);
    assert.equal(timers.size, 0);
    monitor.dispose();
  }
});

test('hidden workspaces stop polling and ignore late replies until reopened', async () => {
  const pending = [];
  const {monitor, timers} = harness(() => new Promise(done => pending.push(done)));
  monitor.update([app()]);
  monitor.update([app()], false);
  pending.shift()('unreachable'); await flush();
  assert.equal(monitor.get('notes'), undefined);
  assert.equal(timers.size, 0);
  monitor.update([app()], true);
  pending.shift()('ready'); await flush();
  assert.equal(monitor.get('notes'), 'ready');
  monitor.dispose(); assert.equal(timers.size, 0);
  monitor.update([app()]); assert.equal(pending.length, 0);
});

test('a changed address cannot inherit the previous endpoint answer', async () => {
  const pending = [];
  const {monitor} = harness(selected => new Promise(done => pending.push({selected, done})));
  monitor.update([app()]);
  monitor.update([app('notes', {launch_url:'http://localhost/new'})]);
  pending.shift().done('ready'); await flush();
  assert.equal(monitor.get('notes'), undefined);
  assert.equal(pending[0].selected.launch_url, 'http://localhost/new');
  pending.shift().done('unreachable'); await flush();
  assert.equal(monitor.get('notes'), 'unreachable');
  monitor.dispose();
});

test('large workspaces cap concurrent probes and never overlap polling rounds', async () => {
  const pending = [];
  let started = 0;
  const {monitor, timers} = harness(() => {
    started++; return new Promise(done => pending.push(done));
  });
  const apps = Array.from({length:9}, (_, i) => app(`app-${i}`));
  monitor.update(apps);
  monitor.update(apps); // Repainting the same rows must not restart probes.
  assert.equal(started, 4); assert.equal(timers.size, 0);
  pending.shift()('ready'); await flush();
  assert.equal(started, 5); assert.equal(pending.length, 4);
  monitor.dispose();
  pending.splice(0).forEach(done => done('ready')); await flush();
  assert.equal(started, 5); assert.equal(timers.size, 0);
});
