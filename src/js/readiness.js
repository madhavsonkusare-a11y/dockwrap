// Poll only the visible workspace. A generation invalidates replies when an
// app changes, an operation starts, or the user leaves. At most four IPC probes
// run together; a slow round cannot overlap the next one.
export function createReadinessMonitor({probe, changed = () => {}, interval = 15000,
  schedule = setTimeout, cancel = clearTimeout, concurrency = 4}) {
  const values = new Map();
  let apps = [], signature = '', generation = 0, timer, running = false, disposed = false;
  function stopTimer() { if (timer !== undefined) cancel(timer); timer = undefined; }
  async function round() {
    if (running || disposed || !apps.length) return;
    running = true;
    const version = generation, queue = [...apps];
    async function worker() {
      while (version === generation && queue.length && !disposed) {
        const app = queue.shift();
        let result;
        try { result = await probe(app); } catch { /* Unknown is not an outage. */ }
        if (version !== generation || disposed) return;
        const next = ['ready', 'unreachable'].includes(result) ? result : undefined;
        if (values.get(app.id) !== next) {
          if (next) values.set(app.id, next); else values.delete(app.id);
          changed();
        }
      }
    }
    try { await Promise.all(Array.from({length: Math.min(concurrency, queue.length)}, worker)); }
    finally {
      running = false;
      if (!disposed && apps.length) {
        if (version !== generation) void round();
        else timer = schedule(() => { timer = undefined; void round(); }, interval);
      }
    }
  }
  return {
    get: id => values.get(id),
    update(candidates, enabled = true) {
      if (disposed) return;
      const eligible = enabled ? candidates.filter(app => !app.busy &&
        (app.status === 'running' || app.runtime?.kind !== 'compose')) : [];
      const next = JSON.stringify(eligible.map(app => [app.id, app.launch_url, app.status]));
      if (signature === next) return;
      signature = next;
      generation++;
      stopTimer();
      const previous = new Map(apps.map(app => [app.id, app]));
      apps = eligible.map(app => ({...app}));
      const valid = new Set(apps.filter(app => {
        const old = previous.get(app.id);
        return old?.launch_url === app.launch_url && old?.status === app.status;
      }).map(app => app.id));
      let cleared = false;
      for (const id of values.keys()) if (!valid.has(id)) { values.delete(id); cleared = true; }
      if (cleared) changed();
      void round();
    },
    dispose() {
      disposed = true;
      generation++;
      stopTimer();
      values.clear();
      apps = [];
    },
  };
}
