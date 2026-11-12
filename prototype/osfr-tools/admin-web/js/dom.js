// DOM helpers. Every string from the API goes in as a text node or an attribute value: nothing here parses HTML.
const SVG = 'http://www.w3.org/2000/svg';

export function h(tag, props, ...kids) {
  const el = document.createElement(tag);
  let value;
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'value') value = v;
    else if (k === 'style') for (const [p, s] of Object.entries(v)) el.style.setProperty(p, s);
    else if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? '' : v);
  }
  append(el, kids);
  if (value !== undefined) el.value = value;
  return el;
}

export function append(el, kids) {
  for (const k of [kids].flat(Infinity)) if (k != null && k !== false) el.append(k instanceof Node ? k : String(k));
  return el;
}

export function clear(el, ...kids) { el.replaceChildren(); return append(el, kids); }

export function svg(tag, attrs, ...kids) {
  const el = document.createElementNS(SVG, tag);
  for (const [k, v] of Object.entries(attrs || {})) if (v != null) el.setAttribute(k, v);
  for (const k of kids.flat()) if (k) el.append(k instanceof Node ? k : String(k));
  return el;
}

// Small line icons drawn for this panel (24 px grid, stroked with currentColor).
const ring = (x, y, r) => `M${x - r} ${y}a${r} ${r} 0 1 0 ${2 * r} 0a${r} ${r} 0 1 0 ${-2 * r} 0`;
const ICONS = {
  dashboard: 'M4 4h7v7H4zM13 4h7v4h-7zM13 10h7v10h-7zM4 13h7v7H4z',
  players: `${ring(9, 8, 3.5)}M2.5 20c.8-3.6 3.4-5.5 6.5-5.5s5.7 1.9 6.5 5.5${ring(17, 9, 2.5)}M17 14.5c2.4 0 4 1.5 4.6 4`,
  mail: 'M3.5 6h17v12h-17zM3.5 6l8.5 7 8.5-7',
  shield: 'M12 3l7.5 3v5.5c0 4.6-3.2 8.2-7.5 9.5-4.3-1.3-7.5-4.9-7.5-9.5V6z',
  globe: `${ring(12, 12, 8.5)}M3.5 12h17M12 3.5c2.5 2.6 3.6 5.4 3.6 8.5s-1.1 5.9-3.6 8.5c-2.5-2.6-3.6-5.4-3.6-8.5s1.1-5.9 3.6-8.5`,
  audit: 'M6 3.5h9l3.5 3.5v13.5H6zM9.5 10h6M9.5 13.5h6M9.5 17h4',
  server: 'M4 4.5h16v6H4zM4 13.5h16v6H4zM7.5 7.5h.01M7.5 16.5h.01',
  search: `${ring(11, 11, 6.5)}M16 16l4.5 4.5`,
  close: 'M6 6l12 12M18 6L6 18',
  plus: 'M12 5v14M5 12h14',
  trash: 'M4.5 7h15M9.5 7V4.5h5V7M6.5 7l1 13h9l1-13M10.5 11v5.5M13.5 11v5.5',
  eye: `M2.5 12S6 5.5 12 5.5 21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12z${ring(12, 12, 3)}`,
  goto: 'M4 12h13M12 6.5l5.5 5.5-5.5 5.5M20 5v14',
  bring: 'M20 12H7M12 6.5L6.5 12l5.5 5.5M4 5v14',
  freeze: 'M12 2.5v19M4 7l16 10M20 7L4 17M9.5 4.5L12 7l2.5-2.5M9.5 19.5L12 17l2.5 2.5',
  mute: 'M4 9.5h3.5L12 5.5v13l-4.5-4H4zM16 9.5l5 5M21 9.5l-5 5',
  kick: 'M14 4.5h5.5v15H14M10 8l-4 4 4 4M6 12h10',
  ban: `${ring(12, 12, 8.5)}M6 6l12 12`,
  sun: `${ring(12, 12, 4)}M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4`,
  moon: 'M19.5 14.5A8 8 0 0 1 9.5 4.5a8 8 0 1 0 10 10z',
  menu: 'M4 7h16M4 12h16M4 17h16',
  coins: `${ring(9, 9, 5.5)}M14.5 9.5a5.5 5.5 0 1 1-5 5`,
  send: 'M20.5 3.5L10 14M20.5 3.5l-6.5 17-4-6.5-6.5-4z',
  pin: `M12 21s-6.5-6-6.5-11a6.5 6.5 0 0 1 13 0c0 5-6.5 11-6.5 11z${ring(12, 10, 2.3)}`,
  chevron: 'M9 5.5l6.5 6.5L9 18.5',
  check: 'M5 12.5l4.5 4.5L19 7.5',
  note: 'M5 4.5h14v15H5zM8.5 9h7M8.5 12.5h7M8.5 16h4',
  quest: 'M6 3.5v17M6 4.5h11l-2.5 4 2.5 4H6',
  bag: 'M5 8h14l-1.2 12H6.2zM9 8V6.5a3 3 0 0 1 6 0V8',
  bolt: 'M13 2.5L5 13.5h6l-1 8 8-11h-6z',
  logout: 'M10 4.5H5v15h5M15 8l4 4-4 4M19 12H9',
  megaphone: 'M4 10v4h3l7 4.5v-13L7 10zM17.5 9.5a3.5 3.5 0 0 1 0 5',
  ghost: 'M6 20V10a6 6 0 0 1 12 0v10l-2-1.5-2 1.5-2-1.5-2 1.5-2-1.5zM9.5 10.5h.01M14.5 10.5h.01',
  star: 'M12 3.5l2.6 5.4 5.9.8-4.3 4.1 1 5.9-5.2-2.8-5.2 2.8 1-5.9-4.3-4.1 5.9-.8z',
  gift: 'M4 9h16v4H4zM5.5 13h13v7.5h-13zM12 9v11.5M12 9C10 5 6.5 5.5 7.5 8 8 9 12 9 12 9zM12 9c2-4 5.5-3.5 4.5-1-.5 1-4.5 1-4.5 1z',
};
export function icon(name, size = 18) {
  return svg('svg', { viewBox: '0 0 24 24', width: size, height: size, class: 'icon', 'aria-hidden': 'true', fill: 'none',
    stroke: 'currentColor', 'stroke-width': 1.8, 'stroke-linecap': 'round', 'stroke-linejoin': 'round' },
  svg('path', { d: ICONS[name] || ICONS.star }));
}

// Formatting
export const toMs = (t) => (typeof t === 'number' ? (t < 1e12 ? t * 1000 : t) : Date.parse(t));
export function ago(t) {
  const ms = toMs(t);
  if (!Number.isFinite(ms)) return '';
  const s = Math.round((Date.now() - ms) / 1000);
  if (s < 0) return `in ${dur(-s)}`;
  if (s < 45) return 'just now';
  if (s < 3600) return `${Math.round(s / 60)} min ago`;
  if (s < 86400) return `${Math.round(s / 3600)} h ago`;
  if (s < 86400 * 30) return `${Math.round(s / 86400)} d ago`;
  return when(t);
}
export function dur(s) {
  s = Math.max(0, Math.round(s));
  if (s < 60) return `${s} s`;
  if (s < 3600) return `${Math.floor(s / 60)} min`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ${Math.floor((s % 3600) / 60)} min`;
  return `${Math.floor(s / 86400)} d ${Math.floor((s % 86400) / 3600)} h`;
}
export function when(t) {
  const ms = toMs(t);
  return Number.isFinite(ms) ? new Date(ms).toLocaleString(undefined, { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' }) : '';
}
// The local date and time to the second, for the audit log.
export function stamp(t) {
  const ms = toMs(t);
  return Number.isFinite(ms) ? new Date(ms).toLocaleString(undefined, { year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit', second: '2-digit' }) : '';
}
// Labels for the gateway's raw values: roles are lowercase, accounts and jobs are numbers, zone names may be null,
// audit staff and targets are objects, and an "until" past 1e15 (u64::MAX) means until lifted.
export const roleName = (r) => (r ? String(r)[0].toUpperCase() + String(r).slice(1) : 'Player');
export const nameOf = (x) => (x && typeof x === 'object' ? x.name ?? '' : x ?? '');
export const zoneLabel = (p) => p?.zone_name || (p?.zone != null ? `Zone ${p.zone}` : 'Loading');
// Names the gateway serves for ids (GET /admin/api/names): jobs by profile id, quests by id. Filled at sign-in;
// an id it does not name shows as the id.
export const names = { jobs: {}, quests: {} };
export const jobLabel = (j) => (typeof j === 'string' ? j : j != null ? names.jobs[j] || `Job ${j}` : '');
export const questLabel = (q) => names.quests[q] || `Quest ${q}`;
export const accountLabel = (a) => (typeof a === 'number' ? `account ${a}` : a ?? '');
export const untilText = (u) => (u == null ? '' : u >= 1e15 ? 'until lifted' : ago(u));
export const num = (n) => (Number.isFinite(+n) ? (+n).toLocaleString() : String(n ?? ''));
export const pos = (p) => (Array.isArray(p) ? p.map((v) => Math.round(v)).join(', ') : '');
export const plural = (n, one, many = `${one}s`) => `${num(n)} ${n === 1 ? one : many}`;
export const initials = (name) => String(name || '?').split(/\s+/).slice(0, 2).map((w) => w[0]).join('').toUpperCase();
// Search rank: prefix 4, word prefix 3, substring 2, letters in order 1, else 0.
export function score(text, q) {
  const t = text.toLowerCase();
  if (!q) return 1;
  if (t.startsWith(q)) return 4;
  if (t.split(/[\s_]+/).some((w) => w.startsWith(q))) return 3;
  if (t.includes(q)) return 2;
  let i = 0;
  for (const c of t) if (c === q[i]) i++;
  return i === q.length ? 1 : 0;
}
export function debounce(fn, ms = 160) {
  let t;
  return (...a) => { clearTimeout(t); t = setTimeout(() => fn(...a), ms); };
}

// Toasts: a short message, optionally with an Undo button that runs `undo` once.
// While a modal (the drawer, a dialog) is open the rest of the page is inert and under the top layer, so the toast
// goes inside the topmost modal, where it shows and its Undo button can be pressed.
export function toast(text, { kind = 'ok', undo } = {}) {
  const host = [...document.querySelectorAll('dialog[open]')].filter((d) => d.matches(':modal')).pop();
  const box = host ? host.querySelector(':scope > .toasts') || host.appendChild(h('div', { class: 'toasts', 'aria-live': 'polite' }))
    : document.getElementById('toasts');
  const el = h('div', { class: `toast toast-${kind}`, role: kind === 'error' ? 'alert' : 'status' },
    icon(kind === 'error' ? 'close' : 'check', 16), h('span', {}, text));
  if (undo) {
    el.append(h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: async () => { el.remove(); await undo(); } }, 'Undo'));
  }
  box.append(el);
  setTimeout(() => el.remove(), undo ? 8000 : 4500);
}
export const fail = (e) => toast(e?.message || String(e), { kind: 'error' });

// A modal built on <dialog>; resolves when it closes. `build(close)` returns the dialog's content.
// It resolves from its own close(), Escape (cancel) or a backdrop click, not from the `close` event, which a
// background tab may hold back.
export function modal(className, build) {
  return new Promise((resolve) => {
    const dlg = h('dialog', { class: `modal ${className || ''}` });
    let done = false;
    const close = (v) => {
      if (done) return;
      done = true;
      if (dlg.open) dlg.close();
      dlg.remove();
      resolve(v);
    };
    dlg.append(build(close, dlg));
    dlg.addEventListener('cancel', (e) => { e.preventDefault(); close(null); });
    dlg.addEventListener('close', () => close(null));
    dlg.addEventListener('click', (e) => { if (e.target === dlg) close(null); });
    document.body.append(dlg);
    dlg.showModal();
  });
}

// A confirm or small form. fields: [{name, label, type: text|textarea|number|select, options, value, required}]
export function ask({ title, body, confirm = 'Confirm', danger = false, fields = [] }) {
  return modal('modal-ask', (close) => {
    const form = h('form', { class: 'ask', method: 'dialog' },
      h('h2', { class: 'ask-title' }, title),
      body ? h('p', { class: 'ask-body' }, body) : null,
      fields.map(field),
      h('div', { class: 'ask-actions' },
        h('button', { class: 'btn btn-ghost', type: 'button', onclick: () => close(null) }, 'Cancel'),
        h('button', { class: `btn ${danger ? 'btn-danger' : 'btn-primary'}`, type: 'submit' }, confirm)));
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      close(Object.fromEntries(fields.map((f) => [f.name, form.elements[f.name].value])));
    });
    // Start on the first empty text field; with none, on Cancel for a destructive confirm, else on the confirm button.
    const buttons = form.querySelectorAll('.ask-actions .btn');
    const first = [...form.querySelectorAll('input, textarea')].find((i) => !i.value) || (danger ? buttons[0] : buttons[1]);
    first.setAttribute('autofocus', '');
    return form;
  });
}

export function field(f) {
  const attrs = { name: f.name, required: f.required, placeholder: f.placeholder, min: f.min, max: f.max, maxlength: f.maxlength };
  let input;
  if (f.type === 'select') input = h('select', { ...attrs, value: f.value }, f.options.map(([v, l]) => h('option', { value: v }, l)));
  else if (f.type === 'textarea') input = h('textarea', { ...attrs, rows: f.rows || 3, value: f.value });
  else input = h('input', { ...attrs, type: f.type || 'text', value: f.value, inputmode: f.type === 'number' ? 'numeric' : null });
  return h('label', { class: 'field' }, h('span', {}, f.label), input);
}

export function empty(text, iconName = 'star') {
  return h('div', { class: 'empty' }, icon(iconName, 22), h('p', {}, text));
}

// Theme: dark by default, light on request; remembered per browser when storage allows.
export function setTheme(t) {
  document.documentElement.dataset.theme = t;
  try { localStorage.setItem('admin-theme', t); } catch { /* not remembered */ }
  window.dispatchEvent(new Event('admin:theme'));
}
export function savedTheme() {
  try { return localStorage.getItem('admin-theme') === 'light' ? 'light' : 'dark'; } catch { return 'dark'; }
}
export const toggleTheme = () => setTheme(document.documentElement.dataset.theme === 'light' ? 'dark' : 'light');

export function badge(text, kind = '') { return h('span', { class: `badge ${kind ? `badge-${kind}` : ''}` }, text); }
