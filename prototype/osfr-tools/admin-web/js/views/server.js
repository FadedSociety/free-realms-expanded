// Server: status cards from /status, refreshed with the shared 2 s poll, and the tick history since the page opened.
import { h, clear, num, dur } from '../dom.js';
import { isMock } from '../api.js';
import { onLive } from '../live.js';
import { pageHead, card, stat, sparkline, tickHealth, contentText } from '../ui.js';

export function render(root) {
  const cards = h('div', { class: 'stats stats-3' });
  const mean = h('div', { class: 'spark-box' });
  const max = h('div', { class: 'spark-box' });
  const conn = h('dl', { class: 'facts' });
  root.append(pageHead('Server', 'Health of the gateway this panel talks to.'), cards,
    h('div', { class: 'grid-2' },
      card('Tick time, last 5 minutes', { aside: h('span', { class: 'muted small' }, 'budget 50 ms') },
        h('p', { class: 'muted small' }, 'Mean'), mean, h('p', { class: 'muted small' }, 'Worst'), max),
      card('This panel', {}, conn)));
  return onLive((L) => {
    const s = L.status;
    if (L.error && !s) { clear(cards, stat('Gateway', 'Not answering', L.error.message, 'bad')); return; }
    if (!s) return;
    const [kind, health] = tickHealth(s.tick_ms);
    clear(cards,
      stat('Players online', num(s.online), null, 'accent'),
      stat('Uptime', dur(s.uptime_s)),
      stat('Tick', health, `mean ${s.tick_ms.mean} ms · worst ${s.tick_ms.max} ms`, kind),
      stat('Memory', s.memory_mb == null ? 'Not measured' : `${num(s.memory_mb)} MB`, s.memory_mb == null ? 'the gateway reads it on Linux only' : null),
      stat('Content', contentText(s.content), 'the loaded game data', 'text'),
      stat('Saves', s.saves_ok ? 'OK' : 'Failing', s.saves_ok ? 'atomic writes succeeding' : 'check the server log', s.saves_ok ? '' : 'bad'));
    clear(mean, sparkline(L.ticks, { key: 'mean', w: 600, ht: 70, max: 50 }));
    clear(max, sparkline(L.ticks, { key: 'max', w: 600, ht: 70, max: 60 }));
    clear(conn, [['Data', isMock() ? 'Mock data from mock/*.json' : 'Live gateway API'], ['API', `${location.origin}/admin/api/`],
      ['Refresh', 'every 2 s while this tab is visible'], ['Last update', L.error ? `failed: ${L.error.message}` : new Date().toLocaleTimeString()]]
      .map(([k, v]) => [h('dt', {}, k), h('dd', {}, v)]));
  });
}
