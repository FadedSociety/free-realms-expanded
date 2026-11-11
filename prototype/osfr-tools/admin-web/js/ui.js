// Shared view pieces: page head, cards, stat cards, tables, pager, segmented control, sparkline, player search.
import { api } from './api.js';
import { h, clear, svg, icon, num, debounce, initials, roleName } from './dom.js';

export const pageHead = (title, sub, ...actions) => h('div', { class: 'page-head' },
  h('div', {}, h('h1', {}, title), sub ? h('p', { class: 'muted' }, sub) : null),
  actions.length ? h('div', { class: 'page-actions' }, actions) : null);

export function card(title, { aside, cls = '' } = {}, ...kids) {
  return h('section', { class: `card ${cls}` }, title ? h('div', { class: 'card-head' }, h('h2', {}, title), aside || null) : null, kids);
}

export function stat(label, value, sub, kind = '') {
  return h('div', { class: `stat ${kind ? `stat-${kind}` : ''}` },
    h('span', { class: 'stat-label' }, label), h('strong', { class: 'stat-value' }, value), sub ? h('span', { class: 'stat-sub' }, sub) : null);
}

export const tickHealth = (t) => (!t ? ['', 'No data'] : t.mean < 20 && t.max < 50 ? ['ok', 'Healthy'] : t.mean < 35 ? ['warn', 'Slow'] : ['bad', 'Overloaded']);

// cols: [{label, get(row) -> node|string, cls}]. onRow makes rows open on click or Enter.
// A table that is redrawn in place keeps keyboard focus on the same row (rows carry data-key = character_id).
export function table(cols, rows, { onRow, empty = 'Nothing here', caption } = {}) {
  const head = h('thead', {}, h('tr', {}, cols.map((c) => h('th', { class: c.cls, scope: 'col' }, c.label))));
  const body = h('tbody', {}, rows.length ? rows.map((r) => {
    const tr = h('tr', onRow ? { class: 'clickable', tabindex: '0', 'data-key': r.character_id, onclick: () => onRow(r), onkeydown: (e) => { if (e.key === 'Enter') onRow(r); } } : {},
      cols.map((c) => h('td', { class: c.cls }, c.get(r))));
    return tr;
  }) : h('tr', {}, h('td', { class: 'table-empty', colspan: cols.length }, empty)));
  return h('div', { class: 'table-wrap' }, h('table', { class: 'table' }, caption ? h('caption', { class: 'sr-only' }, caption) : null, head, body));
}
export function redraw(slot, ...kids) {
  const key = slot.contains(document.activeElement) ? document.activeElement.closest('tr')?.dataset.key : null;
  clear(slot, ...kids);
  if (key != null) slot.querySelector(`tr[data-key="${CSS.escape(key)}"]`)?.focus();
}

export function pager(page, shown, per, total, onPage) {
  const from = total ? (page - 1) * per + 1 : 0;
  const to = (page - 1) * per + shown;
  return h('div', { class: 'pager' },
    h('span', { class: 'muted' }, total ? `${num(from)}–${num(to)} of ${num(total)}` : 'No results'),
    h('button', { class: 'btn btn-sm btn-ghost', type: 'button', disabled: page <= 1, onclick: () => onPage(page - 1) }, 'Previous'),
    h('button', { class: 'btn btn-sm btn-ghost', type: 'button', disabled: to >= total, onclick: () => onPage(page + 1) }, 'Next'));
}

export function seg(options, value, onChange, labelText) {
  const el = h('div', { class: 'seg', role: 'group', 'aria-label': labelText });
  const paint = (v) => clear(el, options.map(([k, l]) => h('button', { class: 'btn btn-sm', type: 'button', 'aria-pressed': String(k === v),
    onclick: () => { paint(k); onChange(k); } }, l)));
  paint(value);
  return el;
}

export function sparkline(points, { w = 240, ht = 48, key = 'mean', max } = {}) {
  const vals = points.map((p) => p[key]).filter(Number.isFinite);
  const top = max || Math.max(1, ...vals) * 1.15;
  const step = w / Math.max(1, vals.length - 1);
  return svg('svg', { class: 'spark', viewBox: `0 0 ${w} ${ht}`, preserveAspectRatio: 'none', 'aria-hidden': 'true' },
    vals.length > 1 ? svg('polyline', { points: vals.map((v, i) => `${(i * step).toFixed(1)},${(ht - (v / top) * (ht - 4) - 2).toFixed(1)}`).join(' '), 'vector-effect': 'non-scaling-stroke' }) : null);
}

export const roleBadge = (role) => h('span', { class: `badge ${role && String(role).toLowerCase() !== 'player' ? 'badge-role' : ''}` }, roleName(role));
export const contentText = (c) => (c && typeof c === 'object' ? Object.entries(c).map(([k, v]) => `${k} ${v}`).join(' · ') : String(c ?? 'unknown'));
export const who = (p, sub) => h('span', { class: 'who' }, h('span', { class: 'avatar', 'aria-hidden': 'true' }, initials(p.name)),
  h('span', { class: 'who-text' }, h('strong', {}, p.name), sub ? h('span', { class: 'muted' }, sub) : null));
export const onlineDot = (on) => h('span', { class: `dot ${on ? 'dot-on' : ''}`, 'aria-hidden': 'true' });

// Autocomplete under a text input. extra(q) adds non-player options; onPick({label, value}) gets the choice.
export function suggest(input, onPick, { extra, players = true } = {}) {
  const list = h('ul', { class: 'suggest', role: 'listbox', hidden: true });
  let opts = [];
  let hi = 0;
  let token = 0;
  input.setAttribute('role', 'combobox');
  input.setAttribute('aria-autocomplete', 'list');
  input.setAttribute('aria-expanded', 'false');
  input.setAttribute('autocomplete', 'off');
  const paint = () => {
    list.hidden = !opts.length;
    input.setAttribute('aria-expanded', String(!!opts.length));
    clear(list, opts.map((o, k) => h('li', { class: 'suggest-item', role: 'option', 'aria-selected': String(k === hi),
      onmousedown: (e) => { e.preventDefault(); pick(o); } }, o.icon ? icon(o.icon, 16) : null, h('span', {}, o.label), o.sub ? h('span', { class: 'muted' }, o.sub) : null)));
  };
  const pick = (o) => { opts = []; paint(); onPick(o); };
  const load = debounce(async () => {
    const q = input.value.trim();
    const mine = ++token;
    let rows = [];
    if (players && q) { try { rows = (await api.players({ q })).rows; } catch { /* no suggestions */ } }
    if (mine !== token) return;
    opts = [...(extra ? extra(q) : []), ...rows.slice(0, 6).map((p) => ({ label: p.name, sub: p.online ? 'Online' : 'Offline', icon: 'players', value: { kind: 'player', ...p } }))];
    hi = 0;
    paint();
  }, 120);
  input.addEventListener('input', load);
  input.addEventListener('focus', load);
  input.addEventListener('blur', () => setTimeout(() => { opts = []; paint(); }, 120));
  input.addEventListener('keydown', (e) => {
    if (!opts.length) return;
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); hi = (hi + (e.key === 'ArrowDown' ? 1 : -1) + opts.length) % opts.length; paint(); }
    else if (e.key === 'Enter' || e.key === 'Tab') { if (e.key === 'Enter' || input.value) { e.preventDefault(); pick(opts[hi]); } }
    else if (e.key === 'Escape') { e.stopPropagation(); opts = []; paint(); }
  });
  return h('div', { class: 'suggest-wrap' }, input, list);
}
