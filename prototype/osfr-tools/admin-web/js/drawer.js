// The player drawer: opened from anywhere, it slides in a player's profile with tabs and the GM action buttons.
import { api } from './api.js';
import { h, clear, icon, ask, ago, when, stamp, dur, num, pos, plural, initials, empty, badge, fail, field, toMs,
  roleName, nameOf, zoneLabel, jobLabel, questLabel, accountLabel } from './dom.js';
import * as act from './actions.js';
import { itemArt, pickItem } from './picker.js';

const TABS = [['overview', 'Overview'], ['inventory', 'Inventory'], ['quests', 'Quests'], ['mail', 'Mail'],
  ['moderation', 'Moderation'], ['notes', 'Notes'], ['audit', 'Audit']];
let dlg = null;
let cur = null; // { id, tab, profile }

export function hide() {
  cur = null;
  if (dlg?.open) dlg.close();
}

// Redraws the drawer but keeps its toast stack (toasts live inside the open modal; see dom.js toast()).
function paint(...kids) {
  const toasts = dlg.querySelector(':scope > .toasts');
  clear(dlg, ...kids, toasts);
}

export function openPlayer(id, tab = 'overview') {
  if (!dlg) {
    dlg = h('dialog', { class: 'drawer', 'aria-label': 'Player' });
    dlg.addEventListener('click', (e) => { if (e.target === dlg) hide(); });
    dlg.addEventListener('cancel', (e) => { e.preventDefault(); hide(); });
    window.addEventListener('admin:changed', (e) => { if (cur && String(e.detail.id) === String(cur.id)) load(true); });
    document.body.append(dlg);
  }
  cur = { id, tab, profile: null };
  paint(h('div', { class: 'drawer-loading' }, 'Loading player'));
  if (!dlg.open) dlg.showModal();
  load(false);
}

async function load(keepScroll) {
  const id = cur.id;
  const scroll = dlg.querySelector('.drawer-body')?.scrollTop || 0;
  try {
    const profile = await api.profile(id);
    if (!cur || cur.id !== id) return;
    cur.profile = profile;
    render();
    if (keepScroll) dlg.querySelector('.drawer-body').scrollTop = scroll;
  } catch (e) {
    fail(e);
    if (cur && !cur.profile) paint(h('div', { class: 'drawer-loading' }, e.message, h('button', { class: 'btn', onclick: () => hide() }, 'Close')));
  }
}

const who = () => ({ character_id: cur.id, name: cur.profile.identity.name });

function render() {
  const { identity: me, location: loc } = cur.profile;
  const sanc = cur.profile.sanctions || {};
  const muted = sanc.muted ?? me.muted;
  const banned = sanc.banned ?? me.banned;
  const p = who();
  const frozen = me.frozen ?? act.frozen.has(cur.id);
  const spect = act.spectating && String(act.spectating.character_id) === String(cur.id);
  const btn = (label, ic, onclick, opts = {}) => h('button', { class: `btn btn-sm ${opts.kind || ''}`, type: 'button', onclick,
    disabled: opts.disabled || (opts.perm && !act.can(opts.perm)), title: opts.title }, icon(ic, 16), label);

  const body = h('div', { class: 'drawer-body', role: 'tabpanel', id: 'drawer-panel', tabindex: '-1' });
  const tabs = h('div', { class: 'tabs drawer-tabs', role: 'tablist', 'aria-label': 'Player sections' },
    TABS.map(([k, label]) => h('button', { class: 'tab', role: 'tab', type: 'button', 'aria-selected': String(k === cur.tab),
      'aria-controls': 'drawer-panel', tabindex: k === cur.tab ? '0' : '-1', 'data-tab': k, onclick: () => { cur.tab = k; render(); } }, label)));
  tabs.addEventListener('keydown', (e) => {
    const i = TABS.findIndex(([k]) => k === cur.tab);
    const j = { ArrowRight: i + 1, ArrowLeft: i - 1, Home: 0, End: TABS.length - 1 }[e.key];
    if (j === undefined) return;
    e.preventDefault();
    cur.tab = TABS[(j + TABS.length) % TABS.length][0];
    render();
    dlg.querySelector(`[data-tab="${cur.tab}"]`).focus();
  });

  paint(
    h('header', { class: 'drawer-head' },
      h('div', { class: 'avatar avatar-lg', 'aria-hidden': 'true' }, initials(me.name)),
      h('div', { class: 'drawer-title' },
        h('h2', {}, me.name),
        h('p', { class: 'muted' },
          h('span', { class: `dot ${me.online ? 'dot-on' : ''}`, 'aria-hidden': 'true' }),
          me.online ? `Online in ${zoneLabel(loc)}` : `Last seen ${me.last_seen ? ago(me.last_seen) : 'never'}`,
          ' · ', accountLabel(me.account), ' · ', roleName(me.role)),
        h('div', { class: 'chips' },
          muted ? badge('Muted', 'warn') : null, banned ? badge('Banned', 'danger') : null,
          frozen ? badge('Frozen', 'info') : null, spect ? badge('You are spectating', 'accent') : null)),
      h('button', { class: 'icon-btn', type: 'button', 'aria-label': 'Close player', onclick: () => hide() }, icon('close'))),
    h('div', { class: 'drawer-actions' },
      spect ? btn('Stop', 'eye', () => act.power('unspectate'), { kind: 'btn-accent', perm: 'powers' })
        : btn('Spectate', 'eye', () => act.power('spectate', p), { perm: 'powers', disabled: !me.online, title: me.online ? null : 'Offline' }),
      btn('Goto', 'goto', () => act.move(p, 'goto'), { perm: 'move', disabled: !me.online }),
      btn('Bring', 'bring', () => act.move(p, 'bring'), { perm: 'move', disabled: !me.online }),
      btn(frozen ? 'Unfreeze' : 'Freeze', 'freeze', () => act.power(frozen ? 'unfreeze' : 'freeze', p), { perm: 'powers', disabled: !me.online && !frozen }),
      btn(muted ? 'Unmute' : 'Mute', 'mute', () => act.moderate(p, muted ? 'unmute' : 'mute'), { perm: muted ? 'unmute' : 'mute' }),
      btn('Kick', 'kick', () => act.moderate(p, 'kick'), { perm: 'kick', disabled: !me.online }),
      btn(banned ? 'Unban' : 'Ban', 'ban', () => act.moderate(p, banned ? 'unban' : 'ban'), { kind: banned ? '' : 'btn-danger-ghost', perm: banned ? 'unban' : 'ban' }),
      btn('Mail', 'mail', () => { hide(); act.openMail({ to: [{ kind: 'player', id: p.character_id, label: p.name }] }); }, { perm: 'mail' })),
    tabs, body);
  PANELS[cur.tab](body, p);
}

const section = (title, ...kids) => h('section', { class: 'dsec' }, h('h3', {}, title), ...kids);
const facts = (rows) => h('dl', { class: 'facts' }, rows.filter(Boolean).map(([k, v]) => [h('dt', {}, k), h('dd', {}, v ?? '')]));
const chipList = (list, fmt) => (list?.length ? h('div', { class: 'chips' }, list.map((x) => h('span', { class: 'chip' }, fmt(x)))) : h('p', { class: 'muted' }, 'None'));
const label = (x) => (typeof x === 'string' ? x : x?.name ?? JSON.stringify(x));

const PANELS = {
  overview(body, p) {
    const pr = cur.profile;
    const id = pr.identity;
    const others = Object.entries(id.account_characters || {}).filter(([cid]) => String(cid) !== String(cur.id));
    const amount = h('input', { class: 'input', type: 'number', min: 0, inputmode: 'numeric', placeholder: 'Amount', 'aria-label': 'Coin amount' });
    const coinOp = (op) => () => {
      if (amount.value === '' || Number(amount.value) < 0) { amount.focus(); return; }
      act.coins(p, op, amount.value, pr.coins);
    };
    clear(body,
      h('div', { class: 'stat-row' },
        stat('Coins', pr.coins == null ? 'Unknown' : num(pr.coins)), stat('Play time', id.play_time_s == null ? 'Not tracked' : dur(id.play_time_s)),
        stat('Collections', `${num(pr.collections_n ?? 0)}${pr.collections_done != null ? ` · ${num(pr.collections_done)} done` : ''}`),
        stat(id.online ? 'Status' : 'Last seen', id.online ? 'Online now' : id.last_seen ? ago(id.last_seen) : 'Never')),
      section('Coins', h('div', { class: 'row-form' }, amount,
        h('div', { class: 'seg' }, ['give', 'take', 'set'].map((op) => h('button', { class: 'btn btn-sm', type: 'button', onclick: coinOp(op), disabled: !act.can('coins') }, op[0].toUpperCase() + op.slice(1)))))),
      section('Identity', facts([['Character id', String(id.character_id ?? cur.id)], ['Account', accountLabel(id.account)], ['Role', roleName(id.role)],
        ['Created', id.created ? when(id.created) : 'Not saved yet'], ['Last seen', id.online ? 'Now' : id.last_seen ? when(id.last_seen) : 'Never'],
        id.client && ['Client', id.client === 'rust' ? 'Rust client' : 'Original client']])),
      others.length ? section('Same account', h('div', { class: 'chips' }, others.map(([cid, n]) => h('button', { class: 'chip chip-btn', type: 'button', onclick: () => openPlayer(cid) }, n)))) : null,
      section('Location', facts([['Zone', zoneLabel(pr.location)], ['Position', pos(pr.location?.position) || 'Unknown']]),
        h('button', { class: 'btn btn-sm', type: 'button', onclick: () => act.sendFlow(p), disabled: !act.can('move') }, icon('pin', 16), 'Send to a place')),
      section('Jobs', chipList(pr.jobs, (j) => `${jobLabel(j.job ?? j.name)}${j.level != null ? ` · level ${j.level}` : ''}${j.active ? ' · active' : ''}`)),
      section('Mounts', chipList(pr.mounts, label)),
      section('Pets', chipList(pr.pets, label)));
  },

  async inventory(body, p) {
    clear(body, h('p', { class: 'muted' }, 'Loading inventory'));
    let inv;
    try { inv = await api.inventory(cur.id); } catch (e) { if (body.isConnected) clear(body, empty(e.message, 'bag')); return; }
    if (!body.isConnected) return;
    const selected = new Set();
    // The gateway's worn rows name the job (profile) and slot number and the bag stack they wear (`instance`);
    // each is one item.
    const worn = (e) => ({ ...e, equipped: true, count: e.count ?? 1, slot: typeof e.slot === 'number' ? `slot ${e.slot}` : e.slot });
    const entries = [...inv.equipped.map(worn), ...inv.bags];
    const info = h('p', { class: 'inv-info', 'aria-live': 'polite' }, 'Click items to select them. Delete removes the selection.');
    const removeBtn = h('button', { class: 'btn btn-sm btn-danger-ghost', type: 'button', disabled: true, onclick: () => removeSelected() }, icon('trash', 16), 'Remove');
    const sync = () => {
      removeBtn.disabled = !selected.size || !act.can('inventory');
      clear(removeBtn, icon('trash', 16), selected.size ? `Remove ${selected.size}` : 'Remove');
    };
    const tile = (e) => {
      const b = h('button', { class: 'tile', type: 'button', 'aria-pressed': 'false', title: `${e.name} · #${e.item_id} · ${e.equipped ? e.slot : `slot ${e.slot}`}`,
        'aria-label': `${e.name}, ${e.count} (${e.equipped ? e.slot : `slot ${e.slot}`})` },
      itemArt(e), e.count > 1 ? h('span', { class: 'tile-count' }, num(e.count)) : null,
      e.equipped ? h('span', { class: 'tile-slot' }, e.slot) : null);
      const show = () => clear(info, h('strong', {}, e.name), ` · #${e.item_id} · ${e.equipped ? `equipped (${e.slot})` : `bag slot ${e.slot}`} · ×${num(e.count)}`);
      b.addEventListener('mouseenter', show);
      b.addEventListener('focus', show);
      b.addEventListener('click', () => {
        selected.has(e) ? selected.delete(e) : selected.add(e);
        b.setAttribute('aria-pressed', String(selected.has(e)));
        sync();
      });
      return b;
    };
    async function removeSelected() {
      const list = [...selected];
      if (!list.length) return;
      if (list.length === 1 && list[0].count > 1) {
        const v = await ask({ title: `Remove ${list[0].name}`, body: `${p.name} has ${num(list[0].count)}.`, confirm: 'Remove', danger: true,
          fields: [{ name: 'n', label: 'How many', type: 'number', min: 1, max: list[0].count, value: String(list[0].count), required: true }] });
        if (!v) return;
        const n = Math.min(list[0].count, Math.max(1, Math.round(Number(v.n))));
        return act.removeItems(p, [{ ...list[0], remove: n === list[0].count ? 'all' : n }], { confirmed: true });
      }
      return act.removeItems(p, list.map((e) => ({ ...e, remove: 'all' })));
    }
    const grid = (list) => {
      const g = h('div', { class: 'inv-grid' }, list.map(tile));
      g.addEventListener('keydown', (e) => { if (e.key === 'Delete' || e.key === 'Backspace') { e.preventDefault(); removeSelected(); } });
      return g;
    };
    const total = entries.reduce((s, e) => s + (e.count || 0), 0);
    clear(body,
      h('div', { class: 'toolbar' },
        h('button', { class: 'btn btn-sm btn-primary', type: 'button', disabled: !act.can('inventory'), onclick: async () => {
          const r = await pickItem({ title: 'Add an item', who: `to ${p.name}`, withEquip: true });
          if (r) act.giveItem(p, r.item, r.count, r.equip);
        } }, icon('plus', 16), 'Add item'),
        removeBtn,
        h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: () => {
          const all = selected.size < entries.length;
          selected.clear();
          if (all) entries.forEach((e) => selected.add(e));
          body.querySelectorAll('.tile').forEach((t) => t.setAttribute('aria-pressed', String(all)));
          sync();
        } }, 'Select all'),
        h('span', { class: 'muted toolbar-end' }, `${plural(entries.length, 'stack')} · ${plural(total, 'item')}`)),
      section('Equipped', inv.equipped.length ? grid(entries.filter((e) => e.equipped)) : h('p', { class: 'muted' }, 'Nothing equipped')),
      section('Bags', inv.bags.length ? grid(entries.filter((e) => !e.equipped)) : empty('The bags are empty', 'bag')),
      info);
  },

  quests(body, p) {
    const norm = (x) => (x && typeof x === 'object' ? x : { quest_id: x });
    const q = { active: (cur.profile.quests?.active || []).map(norm), done: (cur.profile.quests?.done || []).map(norm) };
    const qid = h('input', { class: 'input', type: 'number', min: 1, inputmode: 'numeric', placeholder: 'Quest id', 'aria-label': 'Quest id' });
    const ok = act.can('quests');
    const row = (x, buttons) => h('li', { class: 'list-row' },
      h('div', { class: 'list-main' }, h('strong', {}, x.name || questLabel(x.quest_id)),
        h('span', { class: 'muted' }, `#${x.quest_id}${x.step ? ` · ${x.step}` : ''}${x.finished ? ` · finished ${ago(x.finished)}` : ''}`)),
      h('div', { class: 'list-actions' }, buttons.map(([l, op, kind]) => h('button', { class: `btn btn-sm ${kind || 'btn-ghost'}`, type: 'button', disabled: !ok,
        onclick: () => act.quest(p, op, x.quest_id, x.name || questLabel(x.quest_id)) }, l))));
    clear(body,
      section('Grant a quest', h('form', { class: 'row-form', onsubmit: (e) => { e.preventDefault(); if (qid.value) act.quest(p, 'grant', qid.value); } },
        qid, h('button', { class: 'btn btn-sm btn-primary', type: 'submit', disabled: !ok }, 'Grant'))),
      section(`Active (${q.active.length})`, q.active.length ? h('ul', { class: 'list' }, q.active.map((x) => row(x, [['Complete', 'complete', 'btn-accent'], ['Reset', 'reset'], ['Abandon', 'abandon', 'btn-danger-ghost']]))) : h('p', { class: 'muted' }, 'No active quests')),
      section(`Finished (${q.done.length})`, q.done.length ? h('ul', { class: 'list' }, q.done.map((x) => row(x, [['Reset', 'reset']]))) : h('p', { class: 'muted' }, 'None finished yet')));
  },

  async mail(body, p) {
    clear(body, h('p', { class: 'muted' }, 'Loading mail'));
    let list;
    try { list = await api.playerMail(cur.id); } catch (e) { if (body.isConnected) clear(body, empty(e.message, 'mail')); return; }
    if (!body.isConnected) return;
    clear(body,
      h('div', { class: 'toolbar' }, h('button', { class: 'btn btn-sm btn-primary', type: 'button', disabled: !act.can('mail'),
        onclick: () => { hide(); act.openMail({ to: [{ kind: 'player', id: p.character_id, label: p.name }] }); } }, icon('mail', 16), `Write to ${p.name}`),
      h('span', { class: 'muted toolbar-end' }, `${plural(list.length, 'message')} · ${num(list.filter((m) => !m.read).length)} unread`)),
      list.length ? h('ul', { class: 'list' }, list.map((m) => h('li', { class: 'list-row' },
        h('div', { class: 'list-main' },
          h('strong', {}, m.subject),
          h('span', { class: 'muted' }, `From ${m.from} · ${ago(m.sent)}${attachText(m.attachments)}`)),
        m.read ? badge('Read') : badge('Unread', 'accent')))) : empty('No mail yet', 'mail'));
  },

  moderation(body, p) {
    const pr = cur.profile;
    const s = pr.sanctions || { muted: pr.identity.muted, banned: pr.identity.banned };
    const hist = [...(pr.moderation || [])].sort((a, b) => toMs(b.time) - toMs(a.time));
    const until = (u) => (u == null || u >= 1e15 ? ' until lifted' : `, ends ${ago(u)}`);
    const b = (l, op, kind) => h('button', { class: `btn btn-sm ${kind || ''}`, type: 'button', disabled: !act.can(op), onclick: () => act.moderate(p, op) }, l);
    clear(body,
      h('div', { class: 'stat-row' },
        stat('Chat', s.muted ? `Muted${until(s.muted_until)}` : 'Can talk'),
        stat('Account', s.banned ? `Banned${until(s.banned_until)}` : 'In good standing'),
        stat('Record', plural(hist.length, 'entry', 'entries'))),
      section('Actions', h('div', { class: 'btn-row' }, b('Warn', 'warn'), s.muted ? b('Unmute', 'unmute') : b('Mute', 'mute'),
        b('Kick', 'kick'), s.banned ? b('Unban', 'unban') : b('Ban', 'ban', 'btn-danger-ghost'), b('Jail', 'jail'), b('Release', 'release'))),
      section('History', hist.length ? h('ul', { class: 'list' }, hist.map((m) => {
        const op = String(m.op ?? m.action ?? '');
        const minutes = m.minutes ?? m.after?.minutes;
        return h('li', { class: 'list-row' },
          h('div', { class: 'list-main' },
            h('strong', {}, `${op[0]?.toUpperCase() ?? ''}${op.slice(1)}${minutes ? ` · ${dur(minutes * 60)}` : ''}`),
            h('span', {}, m.reason || 'No reason given'),
            h('span', { class: 'muted' }, `${nameOf(m.staff) || 'Staff'} · ${when(m.time)}${m.via === 'chat' ? ' · by chat command' : ''}`)));
      })) : empty('A clean record', 'shield')));
  },

  notes(body, p) {
    const notes = [...(cur.profile.notes || [])].sort((a, b) => toMs(b.time) - toMs(a.time));
    const form = h('form', { class: 'stack' }, field({ name: 'text', label: 'Add a staff note (only staff see it)', type: 'textarea', maxlength: 500, required: true, rows: 3 }),
      h('div', {}, h('button', { class: 'btn btn-sm btn-primary', type: 'submit', disabled: !act.can('note') }, icon('note', 16), 'Save note')));
    form.addEventListener('submit', (e) => { e.preventDefault(); const t = form.elements.text.value.trim(); if (t) act.note(p, t); });
    clear(body, form, section(`Notes (${notes.length})`, notes.length ? h('ul', { class: 'list' }, notes.map((n) => h('li', { class: 'list-row' },
      h('div', { class: 'list-main' }, h('span', { class: 'note-text' }, n.text ?? n.after?.text ?? ''),
        h('span', { class: 'muted' }, `${n.author || nameOf(n.staff) || 'Staff'} · ${when(n.time)}`))))) : h('p', { class: 'muted' }, 'No notes yet')));
  },

  async audit(body) {
    clear(body, h('p', { class: 'muted' }, 'Loading audit'));
    let page;
    try { page = await api.audit({ player: cur.id, page: 1 }); } catch (e) { if (body.isConnected) clear(body, empty(e.message, 'audit')); return; }
    if (!body.isConnected) return;
    clear(body,
      h('div', { class: 'toolbar' }, h('span', { class: 'muted' }, `${plural(page.total, 'staff action')} on this player`),
        h('a', { class: 'btn btn-sm btn-ghost toolbar-end', href: `#/audit?player=${encodeURIComponent(cur.id)}`, onclick: () => hide() }, 'Open in Audit')),
      page.rows.length ? h('ul', { class: 'list' }, page.rows.map((r) => h('li', { class: 'list-row' },
        h('div', { class: 'list-main' }, h('strong', {}, r.action), h('span', { class: 'muted' }, `${nameOf(r.staff)} · ${stamp(r.time)}${r.via === 'chat' ? ' · by chat command' : ''}`),
          changeText(r) ? h('span', { class: 'small change' }, changeText(r)) : null)))) : empty('No staff actions yet', 'audit'));
  },
};

function stat(k, v) { return h('div', { class: 'mini-stat' }, h('span', { class: 'muted' }, k), h('strong', {}, v)); }
export const attachText = (a) => {
  if (!a) return '';
  // Rows are {item_id, count} from the routes, [item_id, count] in a mail audit row.
  const rows = (a.items || []).map((i) => (Array.isArray(i) ? { item_id: i[0], count: i[1] } : i));
  const parts = [...rows.map((i) => `${i.count} × ${i.name || `item #${i.item_id}`}`), a.coins ? `${num(a.coins)} coins` : null].filter(Boolean);
  return parts.length ? ` · ${parts.join(', ')}` : '';
};
// A staff audit row in words: what changed, before → after, with local times and names for ids, then the reason.
// Keys the gateway keeps for undo (the change record, its key) are not shown.
const HIDDEN = new Set(['undo', 'undid', 'change', 'summary', 'online', 'result', 'minutes']);
const until = (u) => (u == null ? null : u >= 1e15 ? 'until lifted' : `until ${stamp(u)}`);
const items = (list) => (list?.length ? list.map((i) => `${i.name || `item #${i.item_id}`} ×${num(i.count)}`).join(', ') : 'none');
const word = (k) => k.replace(/_/g, ' ');
function plain(k, v) {
  if (v == null || v === '') return 'none';
  if (typeof v === 'number' && /(_until|_at|^time|^sent)$/.test(k)) return v >= 1e15 ? 'until lifted' : stamp(v);
  if (typeof v === 'boolean') return v ? 'yes' : 'no';
  if (Array.isArray(v)) return v.length ? v.map((x) => plain(k, x)).join(', ') : 'none';
  if (typeof v === 'object') return Object.entries(v).filter(([, x]) => x != null).map(([kk, x]) => `${word(kk)} ${plain(kk, x)}`).join(', ');
  return typeof v === 'number' ? num(v) : String(v);
}
const audience = (to) => (to === 'all' ? 'everyone online' : to?.zone != null ? `zone ${to.zone}` : to?.role ? `every ${to.role}` : plain('to', to));
function changes(r) {
  const b = r.before && typeof r.before === 'object' ? r.before : {};
  const a = r.after && typeof r.after === 'object' ? r.after : {};
  const s = a.sanctions;
  switch (r.action) {
    case 'mute': case 'unmute':
      return [`Chat: ${b.muted_until != null ? `muted ${until(b.muted_until)}` : 'can talk'} → ${s?.muted_until != null ? `muted ${until(s.muted_until)}` : 'can talk'}`];
    case 'ban': case 'unban':
      return [`Account: ${b.banned_until != null ? `banned ${until(b.banned_until)}` : 'in good standing'} → ${s?.banned_until != null ? `banned ${until(s.banned_until)}` : 'in good standing'}`];
    case 'kick': return ['Disconnected'];
    case 'warn': return [a.delivered ? 'Shown to the player in chat' : 'Kept on record; the player was offline'];
    case 'note': return [`“${a.text ?? ''}”`];
    case 'login': return [`Signed in as ${a.role ?? 'staff'}`];
    case 'announce': return [`“${a.text ?? ''}” to ${audience(a.to)} as a ${a.style === 'banner' ? 'banner' : 'chat line'}, delivered to ${num(a.delivered ?? 0)}`];
    case 'mail': return [`“${a.subject ?? ''}” to ${a.to ?? 'players'} (${plural(a.recipients ?? 0, 'recipient')})${attachText(a.attachments)}`];
    default: break;
  }
  const lines = [];
  if ('coins' in a || 'coins' in b) lines.push(`Coins ${num(b.coins ?? 0)} → ${num(a.coins ?? 0)}`);
  if ('items' in a || 'items' in b) lines.push(`Items ${items(b.items)} → ${items(a.items)}`);
  if ('worn' in a || 'worn' in b) lines.push(`Worn ${plain('worn', b.worn)} → ${plain('worn', a.worn)}`);
  for (const id of new Set([...Object.keys(b.quests || {}), ...Object.keys(a.quests || {})])) {
    lines.push(`${questLabel(id)}: ${b.quests?.[id] ?? 'none'} → ${a.quests?.[id] ?? 'none'}`);
  }
  for (const id of new Set([...Object.keys(b.job_xp || {}), ...Object.keys(a.job_xp || {})])) {
    lines.push(`${jobLabel(Number(id))} XP ${num(b.job_xp?.[id] ?? 0)} → ${num(a.job_xp?.[id] ?? 0)}`);
  }
  const known = new Set([...HIDDEN, 'coins', 'items', 'worn', 'quests', 'job_xp']);
  for (const k of new Set([...Object.keys(b), ...Object.keys(a)])) {
    if (!known.has(k)) lines.push(k in b ? `${word(k)} ${plain(k, b[k])} → ${plain(k, a[k])}` : `${word(k)} ${plain(k, a[k])}`);
  }
  if (a.result) lines.push(a.result);
  if (a.undid) lines.push('Undid an earlier edit');
  return lines;
}
export function changeText(r) {
  return [...changes(r), r.reason ? `Reason: ${r.reason}` : ''].filter(Boolean).join(' · ');
}
