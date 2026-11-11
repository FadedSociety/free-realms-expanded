// Audit: every staff action, filterable by staff member, player, action and time, newest first, paged.
import { api } from '../api.js';
import { h, clear, when, ago, debounce, fail, nameOf } from '../dom.js';
import { changeText } from '../drawer.js';
import { pageHead, table, pager, suggest } from '../ui.js';

// The gateway's action names so far; the field also takes any other name.
const ACTIONS = ['login', 'mute', 'unmute', 'kick', 'ban', 'unban', 'warn', 'note', 'announce', 'mail', 'inventory_add', 'inventory_remove', 'coins', 'quests', 'move', 'powers'];
const f = { staff: '', player: '', playerName: '', action: '', since: '', page: 1, per: 50 };

export function render(root, query) {
  if (query.get('player')) { f.player = query.get('player'); f.playerName = `#${f.player}`; f.page = 1; }
  const slot = h('div', {});
  const staff = h('input', { class: 'input', type: 'search', value: f.staff, placeholder: 'Staff name', 'aria-label': 'Staff member' });
  const player = h('input', { class: 'input', type: 'search', value: f.playerName, placeholder: 'Player', 'aria-label': 'Player' });
  const action = h('input', { class: 'input', type: 'search', value: f.action, list: 'audit-actions', placeholder: 'Any action', 'aria-label': 'Action' });
  const actions = h('datalist', { id: 'audit-actions' }, ACTIONS.map((a) => h('option', { value: a })));
  const since = h('input', { class: 'input', type: 'datetime-local', value: f.since, 'aria-label': 'Since' });
  const go = () => { f.page = 1; load(); };
  staff.addEventListener('input', debounce(() => { f.staff = staff.value.trim(); go(); }, 200));
  player.addEventListener('input', () => { if (!player.value) { f.player = ''; f.playerName = ''; go(); } });
  action.addEventListener('input', debounce(() => { f.action = action.value.trim(); go(); }, 200));
  since.addEventListener('change', () => { f.since = since.value; go(); });

  root.append(pageHead('Audit', 'Every staff action, from the panel, chat commands and the Rust client.'),
    h('div', { class: 'filters' },
      h('label', { class: 'inline-field grow' }, h('span', {}, 'Staff'), staff),
      h('label', { class: 'inline-field grow' }, h('span', {}, 'Player'), suggest(player, (o) => { f.player = String(o.value.character_id); f.playerName = o.value.name; player.value = o.value.name; go(); })),
      h('label', { class: 'inline-field' }, h('span', {}, 'Action'), action, actions),
      h('label', { class: 'inline-field' }, h('span', {}, 'Since'), since)),
    slot);

  const cols = [
    { label: 'When', get: (r) => h('span', { title: when(r.time) }, ago(r.time)) },
    { label: 'Staff', get: (r) => nameOf(r.staff) },
    { label: 'Action', get: (r) => h('span', { class: 'badge' }, r.action) },
    { label: 'Target', get: (r) => nameOf(r.target), cls: 'hide-sm' },
    { label: 'Via', get: (r) => r.via ?? '', cls: 'hide-md' },
    { label: 'Change', get: (r) => h('span', { class: 'change clip' }, changeText(r)), cls: 'hide-md' },
  ];
  let token = 0;
  async function load() {
    const mine = ++token;
    try {
      const d = await api.audit({ staff: f.staff, player: f.player, action: f.action, since: f.since ? Math.floor(new Date(f.since).getTime() / 1000) : '', page: f.page });
      if (mine !== token) return;
      if (d.page === 1 && d.rows.length && d.rows.length < d.total) f.per = d.rows.length;
      clear(slot, table(cols, d.rows, { empty: 'No staff action matches', caption: 'Audit log' }),
        pager(d.page || f.page, d.rows.length, d.page_size || f.per, d.total, (p) => { f.page = p; load(); }));
    } catch (e) { fail(e); }
  }
  load();
}
