// Players: everyone online (live, every 2 s) and every account and character (search, filters, sort, paging).
import { api } from '../api.js';
import { h, clear, dur, ago, when, pos, debounce, fail, score, badge, roleName, zoneLabel, jobLabel, accountLabel } from '../dom.js';
import { onLive } from '../live.js';
import { openPlayer } from '../drawer.js';
import { remember } from '../cmd.js';
import * as act from '../actions.js';
import { pageHead, table, redraw, pager, roleBadge, who, onlineDot, seg } from '../ui.js';

const state = { tab: 'online', filter: '', q: '', online: '', role: '', banned: false, muted: false, sort: 'name', page: 1, per: 20 };

export function render(root, query) {
  if (query.get('tab')) state.tab = query.get('tab') === 'all' ? 'all' : 'online';
  const body = h('div', {});
  const tabs = seg([['online', 'Online now'], ['all', 'All players']], state.tab, (t) => { state.tab = t; show(); }, 'Player list');
  root.append(pageHead('Players', 'Click anyone to open their profile.', tabs), body);
  let off = null;

  function show() {
    off?.();
    off = null;
    clear(body);
    if (state.tab === 'online') onlineList(); else allList();
  }

  function onlineList() {
    const filter = h('input', { class: 'input', type: 'search', placeholder: 'Filter by name, account, zone or job', value: state.filter, 'aria-label': 'Filter online players' });
    const count = h('span', { class: 'muted' });
    const slot = h('div', {});
    body.append(h('div', { class: 'filters' }, h('div', { class: 'grow' }, filter), count), slot);
    let rows = [];
    const cols = [
      { label: 'Player', get: (p) => who(p, accountLabel(p.account)) },
      { label: 'Role', get: (p) => roleBadge(p.role), cls: 'hide-sm' },
      { label: 'Zone', get: (p) => zoneLabel(p) },
      { label: 'Position', get: (p) => pos(p.position), cls: 'hide-md num' },
      { label: 'Job', get: (p) => `${jobLabel(p.job)} · ${p.level ?? '?'}`, cls: 'hide-sm' },
      { label: 'Session', get: (p) => dur(p.session_s), cls: 'hide-sm num' },
      { label: 'Client', get: (p) => (p.client === 'rust' ? 'Rust' : p.client ? 'Original' : 'Unknown'), cls: 'hide-md' },
    ];
    const paint = () => {
      const q = state.filter.trim().toLowerCase();
      const shown = rows.filter((p) => !q || [p.name, accountLabel(p.account), zoneLabel(p), jobLabel(p.job)].some((t) => score(String(t || ''), q) > 1));
      count.textContent = `${shown.length} of ${rows.length} online`;
      redraw(slot, table(cols, shown, { onRow: (p) => openPlayer(p.character_id), empty: q ? 'Nobody online matches that filter' : 'Nobody is online', caption: 'Players online now' }));
    };
    filter.addEventListener('input', () => { state.filter = filter.value; paint(); });
    off = onLive((L) => { rows = L.online; remember(rows); paint(); });
  }

  function allList() {
    const q = h('input', { class: 'input', type: 'search', maxlength: 64, placeholder: 'Search by name, or a character or account id', value: state.q, 'aria-label': 'Search players' });
    const sel = (name, label, opts) => h('label', { class: 'inline-field' }, h('span', {}, label),
      h('select', { class: 'input', value: state[name], onchange: (e) => { state[name] = e.target.value; state.page = 1; load(); } }, opts.map(([v, l]) => h('option', { value: v }, l))));
    const flag = (name, label) => h('label', { class: 'check' }, h('input', { type: 'checkbox', checked: state[name], onchange: (e) => { state[name] = e.target.checked; state.page = 1; load(); } }), h('span', {}, label));
    const slot = h('div', {});
    body.append(h('div', { class: 'filters' },
      h('div', { class: 'grow' }, q),
      sel('online', 'Status', [['', 'Any'], ['1', 'Online'], ['0', 'Offline']]),
      sel('role', 'Role', [['', 'Any'], ...act.ROLES.map((r) => [r, roleName(r)])]),
      sel('sort', 'Sort', [['name', 'Name'], ['last_seen', 'Last seen'], ['created', 'Created']]),
      flag('muted', 'Muted'), flag('banned', 'Banned')), slot);
    const cols = [
      { label: 'Player', get: (p) => who(p, accountLabel(p.account)) },
      { label: 'Role', get: (p) => roleBadge(p.role), cls: 'hide-sm' },
      { label: 'Status', get: (p) => h('span', { class: 'status' }, onlineDot(p.online), p.online ? 'Online' : p.last_seen ? ago(p.last_seen) : 'Never', p.banned ? badge('Banned', 'danger') : null, p.muted ? badge('Muted', 'warn') : null) },
      { label: 'Last seen', get: (p) => (p.online ? 'Now' : p.last_seen ? when(p.last_seen) : 'Never'), cls: 'hide-md' },
      { label: 'Created', get: (p) => (p.created ? when(p.created) : 'Not saved yet'), cls: 'hide-sm' },
    ];
    let token = 0;
    async function load() {
      const mine = ++token;
      try {
        const d = await api.players({ q: state.q.trim(), online: state.online, role: state.role, banned: state.banned ? 1 : '', muted: state.muted ? 1 : '', sort: state.sort, page: state.page });
        if (mine !== token) return;
        remember(d.rows);
        if (d.page === 1 && d.rows.length && d.rows.length < d.total) state.per = d.rows.length;
        clear(slot, table(cols, d.rows, { onRow: (p) => openPlayer(p.character_id), empty: 'No player matches', caption: 'All players' }),
          pager(d.page || state.page, d.rows.length, d.page_size || state.per, d.total, (pg) => { state.page = pg; load(); }));
      } catch (e) { fail(e); }
    }
    q.addEventListener('input', debounce(() => { state.q = q.value; state.page = 1; load(); }, 150));
    load();
    queueMicrotask(() => q.focus());
  }

  show();
  return () => off?.();
}
