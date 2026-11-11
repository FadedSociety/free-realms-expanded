// One shared 2 s poll of /status and /players/online for every screen, plus the short history the panel draws:
// tick times for the sparkline and each player's recent positions for the map trail.
import { api } from './api.js';

export const live = { status: null, online: [], error: null, ticks: [], trails: new Map(), powers: null };
const subs = new Set();
const HISTORY = 150; // 5 minutes at 2 s
const TRAIL = 40;
let timer = null;

export function onLive(fn) {
  subs.add(fn);
  if (live.status) fn(live);
  return () => subs.delete(fn);
}
export const notify = () => subs.forEach((fn) => fn(live));

async function poll() {
  if (document.hidden) return;
  try {
    const [status, online] = await Promise.all([api.status(), api.online()]);
    live.status = status;
    live.online = online;
    live.error = null;
    live.ticks.push({ t: Date.now(), mean: status.tick_ms?.mean, max: status.tick_ms?.max });
    if (live.ticks.length > HISTORY) live.ticks.shift();
    const seen = new Set();
    for (const p of online) {
      seen.add(p.character_id);
      if (!Array.isArray(p.position)) continue; // a session still loading has no position yet
      const trail = live.trails.get(p.character_id) || [];
      const last = trail[trail.length - 1];
      if (!last || last[0] !== p.position[0] || last[2] !== p.position[2] || last.zone !== p.zone) {
        trail.push(Object.assign([...p.position], { zone: p.zone }));
        if (trail.length > TRAIL) trail.shift();
      }
      live.trails.set(p.character_id, trail.filter((q) => q.zone === p.zone));
    }
    for (const id of live.trails.keys()) if (!seen.has(id)) live.trails.delete(id);
  } catch (e) {
    live.error = e;
  }
  notify();
}

export function startLive() {
  if (timer) return;
  poll();
  timer = setInterval(poll, 2000);
  document.addEventListener('visibilitychange', () => { if (!document.hidden) poll(); });
}
export const refreshLive = poll;
