// Dashboard: who is online, tick health, the live map, recent staff actions and recent mail jobs.
import { api } from '../api.js';
import { h, clear, num, dur, ago, plural, empty, nameOf, zoneLabel } from '../dom.js';
import { live, onLive } from '../live.js';
import { openPlayer, changeText } from '../drawer.js';
import * as act from '../actions.js';
import { zoneMap } from '../map.js';
import { pageHead, card, stat, sparkline, tickHealth, contentText } from '../ui.js';
import { jobRow } from './mail.js';

export function render(root) {
  const stats = h('div', { class: 'stats' });
  const chips = h('div', { class: 'chips zone-chips', role: 'group', 'aria-label': 'Zone' });
  const map = zoneMap({ onPick: (id) => openPlayer(id), label: 'Live map of players' });
  const mapNote = h('p', { class: 'muted small' });
  const auditList = h('ul', { class: 'list list-tight' });
  const jobsList = h('div', { class: 'jobs' });
  let zone = null;
  let zoneName = '';

  root.append(
    pageHead('Dashboard', 'The realm right now. Updates every 2 seconds.'),
    stats,
    h('div', { class: 'dash-grid' },
      card('Live map', { cls: 'map-card', aside: chips }, h('div', { class: 'map-box' }, map.el), mapNote),
      h('div', { class: 'dash-side' },
        card('Recent staff actions', { aside: h('a', { class: 'link', href: '#/audit' }, 'All') }, auditList),
        card('Recent mail', { aside: h('a', { class: 'link', href: '#/mail' }, 'Compose') }, jobsList))));

  function paintStats(L) {
    const s = L.status;
    if (!s) return;
    const [kind, health] = tickHealth(s.tick_ms);
    const rust = L.online.filter((p) => p.client === 'rust').length;
    clear(stats,
      stat('Players online', num(s.online), rust ? `${num(rust)} on the Rust client` : null, 'accent'),
      h('div', { class: `stat stat-${kind}` }, h('span', { class: 'stat-label' }, 'Tick health'),
        h('strong', { class: 'stat-value' }, health), h('span', { class: 'stat-sub' }, `mean ${s.tick_ms.mean} ms · worst ${s.tick_ms.max} ms`), sparkline(L.ticks, { key: 'max', max: 60 })),
      stat('Uptime', dur(s.uptime_s), s.memory_mb == null ? null : `memory ${num(s.memory_mb)} MB`),
      stat('Saves', s.saves_ok ? 'OK' : 'Failing', s.saves_ok ? `content ${contentText(s.content)}` : 'Check the server log', s.saves_ok ? '' : 'bad'));
  }

  function paintZones(L) {
    const counts = new Map();
    for (const p of L.online) if (p.zone != null) counts.set(String(p.zone), { name: zoneLabel(p), n: (counts.get(String(p.zone))?.n || 0) + 1 });
    const zones = [...counts.entries()].sort((a, b) => b[1].n - a[1].n);
    const had = zone;
    if (zone == null || !counts.has(zone)) {
      const watched = act.spectating && L.online.find((p) => String(p.character_id) === String(act.spectating.character_id));
      zone = watched ? String(watched.zone) : zones[0]?.[0] ?? null;
    }
    zoneName = counts.get(zone)?.name || '';
    clear(chips, zones.map(([z, { name, n }]) => h('button', { class: 'chip chip-btn', type: 'button', 'aria-pressed': String(z === zone),
      onclick: () => { zone = z; paintZones(live); drawMap(); } }, `${name} · ${n}`)));
    if (zone !== had || zone == null) drawMap();
  }

  async function drawMap() {
    if (zone == null) { mapNote.textContent = 'Nobody is online.'; return; }
    try {
      const data = await api.map(zone);
      map.draw(data, zoneName, act.spectating?.character_id);
      mapNote.textContent = `${plural(data.players.length, 'player')} in ${zoneName}. ${act.spectating ? `The line is ${act.spectating.name}'s recent path.` : 'Click a dot to open the player.'}`;
    } catch (e) { mapNote.textContent = e.message; }
  }

  async function paintSide() {
    try {
      const [audit, jobs] = await Promise.all([api.audit({ page: 1 }), api.mailJobs()]);
      clear(auditList, audit.rows.slice(0, 7).map((r) => h('li', { class: 'list-row' },
        h('div', { class: 'list-main' }, h('strong', {}, `${nameOf(r.staff)} · ${r.action}`), h('span', { class: 'muted' }, `${r.target ? `${nameOf(r.target)} · ` : ''}${ago(r.time)}`),
          changeText(r) ? h('span', { class: 'small change' }, changeText(r)) : null))));
      clear(jobsList, jobs.length ? jobs.slice(0, 4).map(jobRow) : empty('No mail sent yet', 'mail'));
    } catch { /* the live poll shows connection errors */ }
  }

  const off = onLive((L) => { paintStats(L); paintZones(L); });
  paintSide();
  const t1 = setInterval(drawMap, 2000);
  const t2 = setInterval(paintSide, 6000);
  return () => { off(); clearInterval(t1); clearInterval(t2); };
}
