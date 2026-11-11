// Every staff action, shared by the buttons and the command bar. Destructive ones confirm first; each one toasts
// its result, with Undo where the API has the inverse (mute/unmute, ban/unban, freeze/unfreeze, add/remove, coins).
import { api } from './api.js';
import { ask, toast, fail, dur, num, zoneLabel } from './dom.js';
import { live, notify } from './live.js';

export const me = { name: '', role: '', permissions: null };
// The server enforces roles; the panel only greys out what /me's permission list leaves out. A list that uses none
// of these names (the gateway named its actions differently) greys out nothing rather than everything.
const PERMS = new Set(['view', 'mute', 'unmute', 'kick', 'ban', 'unban', 'warn', 'jail', 'release', 'note', 'mail', 'announce',
  'inventory', 'coins', 'quests', 'move', 'powers']);
export const can = (perm) => {
  const list = me.permissions;
  return !Array.isArray(list) || list.includes('*') || list.includes(perm) || !list.some((x) => PERMS.has(x));
};

export const changed = (id) => window.dispatchEvent(new CustomEvent('admin:changed', { detail: { id } }));
export const pending = { mail: null };
const name = (p) => p.name || `#${p.character_id}`;

async function attempt(fn, ok, undo) {
  try {
    const out = await fn();
    if (ok) toast(typeof ok === 'function' ? ok(out) : ok, { undo });
    return out ?? true;
  } catch (e) {
    fail(e);
    return null;
  }
}

export const DURATIONS = [['15', '15 minutes'], ['60', '1 hour'], ['240', '4 hours'], ['1440', '1 day'], ['10080', '7 days'], ['43200', '30 days']];
export const ZONES = () => [...new Map(live.online.filter((p) => p.zone != null).map((p) => [String(p.zone), zoneLabel(p)])).entries()];
export const ROLES = ['admin', 'moderator', 'builder', 'tester', 'player'];

// Movement
export function move(p, op) {
  const text = { goto: `Teleported to ${name(p)}`, bring: `Brought ${name(p)} to you` }[op];
  return attempt(() => api.move(p.character_id, op), text).then((r) => (changed(p.character_id), r));
}
export async function sendFlow(p) {
  const zones = ZONES();
  const v = await ask({ title: `Send ${name(p)} somewhere`, confirm: 'Send', fields: [
    { name: 'zone', label: 'Zone', type: zones.length ? 'select' : 'text', options: zones, required: true },
    { name: 'xyz', label: 'Position x, y, z (blank for the zone start)', placeholder: '0, 10, 0' }] });
  if (!v) return null;
  const xyz = v.xyz.split(/[\s,]+/).filter(Boolean).map(Number);
  const to = xyz.length === 3 && xyz.every(Number.isFinite) ? { zone: v.zone, position: xyz } : { zone: v.zone };
  return attempt(() => api.move(p.character_id, 'send', to), (loc) => `Sent ${name(p)} to ${loc?.zone_name || 'the zone'}`)
    .then((r) => (changed(p.character_id), r));
}

// Powers. The contract's {state} shape is open, so the panel also remembers what it asked for this session.
export const frozen = new Set();
export const self = { invisible: false, god: false };
export let spectating = null;
export async function power(op, p) {
  const out = await attempt(() => api.powers(op, p?.character_id), {
    spectate: `Spectating ${p && name(p)}. You are hidden from other players.`, unspectate: 'Stopped spectating',
    invisible: 'You are invisible', visible: 'You are visible again', god: 'God mode toggled',
    freeze: `Froze ${p && name(p)}`, unfreeze: `Unfroze ${p && name(p)}` }[op],
  op === 'freeze' ? () => power('unfreeze', p) : op === 'unfreeze' ? () => power('freeze', p) : undefined);
  if (out) {
    if (op === 'freeze') frozen.add(p.character_id);
    if (op === 'unfreeze') frozen.delete(p.character_id);
    if (op === 'spectate' || op === 'unspectate') { spectating = op === 'spectate' ? p : null; self.invisible = !!spectating; }
    if (op === 'invisible' || op === 'visible') self.invisible = op === 'invisible';
    if (op === 'god') self.god = !self.god;
    live.powers = out.state || null;
    notify();
  }
  if (p) changed(p.character_id);
  return out;
}

// Moderation: a short form for each op, prefilled from the command bar when it has the values.
export async function moderate(p, op, preset = {}) {
  const reason = { name: 'reason', label: op === 'warn' ? 'What they read' : 'Reason (kept on their record)', type: op === 'warn' ? 'textarea' : 'text',
    value: preset.reason || '', required: op === 'warn', maxlength: 500 };
  const withPreset = (list) => (preset.minutes && !list.some(([v]) => v === String(preset.minutes))
    ? [[String(preset.minutes), dur(preset.minutes * 60)], ...list] : list);
  const forms = {
    mute: { title: `Mute ${name(p)}`, confirm: 'Mute', fields: [{ name: 'minutes', label: 'For', type: 'select', options: withPreset([...DURATIONS, ['', 'Until lifted']]), value: String(preset.minutes ?? '60') }, reason] },
    ban: { title: `Ban ${name(p)}`, confirm: 'Ban', danger: true, body: 'They are disconnected and cannot sign in until the ban ends.',
      fields: [{ name: 'minutes', label: 'For', type: 'select', options: withPreset([...DURATIONS.slice(3), ['', 'Until lifted']]), value: String(preset.minutes ?? '1440') }, reason] },
    kick: { title: `Kick ${name(p)}`, confirm: 'Kick', danger: true, body: 'They are disconnected and can sign in again.', fields: [reason] },
    warn: { title: `Warn ${name(p)}`, confirm: 'Send warning', body: 'They get it as a staff message in chat if online, and it is kept on their record.', fields: [reason] },
    jail: { title: `Jail ${name(p)}`, confirm: 'Jail', danger: true, fields: [reason] },
  };
  const form = forms[op] || { title: `${op[0].toUpperCase()}${op.slice(1)} ${name(p)}`, confirm: 'Confirm', fields: [reason] };
  const v = await ask(form);
  if (!v) return null;
  const minutes = v.minutes ? Number(v.minutes) : undefined;
  const undo = { mute: 'unmute', ban: 'unban', jail: 'release' }[op];
  const done = { mute: `Muted ${name(p)} ${minutes ? `for ${dur(minutes * 60)}` : 'until lifted'}`, ban: `Banned ${name(p)} ${minutes ? `for ${dur(minutes * 60)}` : 'until lifted'}`,
    kick: `Kicked ${name(p)}`, warn: `Warned ${name(p)}`, unmute: `Unmuted ${name(p)}`, unban: `Unbanned ${name(p)}`,
    jail: `Jailed ${name(p)}`, release: `Released ${name(p)}` }[op];
  const out = await attempt(() => api.moderate(p.character_id, op, minutes, v.reason.trim()),
    (r) => (op === 'warn' && r?.row?.after?.delivered === false ? `Warning kept on ${name(p)}'s record; they are offline and did not see it` : done),
    undo ? () => attempt(() => api.moderate(p.character_id, undo, undefined, 'Undo'), `Undone: ${done}`).then(() => changed(p.character_id)) : undefined);
  changed(p.character_id);
  return out;
}

// Inventory
export async function giveItem(p, item, count = 1, equip = false) {
  const text = `Gave ${num(count)} × ${item.name || `item ${item.item_id}`} to ${name(p)}`;
  const out = await attempt(() => api.addItem(p.character_id, item.item_id, count, equip), text,
    () => attempt(() => api.removeItems(p.character_id, [{ item_id: item.item_id, count }]), 'Undone').then(() => changed(p.character_id)));
  changed(p.character_id);
  return out;
}
// entries: inventory rows [{slot, item_id, name, count, equipped?}] with the count to remove ("all" or a number).
export async function removeItems(p, entries, { confirmed = false } = {}) {
  const what = entries.length === 1 ? `${entries[0].remove === 'all' ? '' : `${entries[0].remove} × `}${entries[0].name || entries[0].item_id}` : `${entries.length} stacks`;
  if (!confirmed && !(await ask({ title: `Remove ${what}?`, body: `From ${name(p)}'s ${entries.some((e) => e.equipped) ? 'equipment and bags' : 'bags'}.`, confirm: 'Remove', danger: true }))) return null;
  const body = entries.map((e) => ({ ...(e.equipped || e.slot == null ? { item_id: e.item_id } : { slot: e.slot }), count: e.remove }));
  const back = entries.map((e) => ({ item_id: e.item_id, name: e.name, count: e.remove === 'all' ? e.count : e.remove, equip: !!e.equipped }));
  const out = await attempt(() => api.removeItems(p.character_id, body), `Removed ${what}`, async () => {
    for (const b of back) await attempt(() => api.addItem(p.character_id, b.item_id, b.count, b.equip));
    toast('Items put back');
    changed(p.character_id);
  });
  changed(p.character_id);
  return out;
}

export async function coins(p, op, amount, before) {
  amount = Number(amount);
  if (op !== 'give' && !(await ask({ title: op === 'set' ? `Set ${name(p)}'s coins to ${num(amount)}?` : `Take ${num(amount)} coins from ${name(p)}?`, confirm: op === 'set' ? 'Set' : 'Take', danger: true }))) return null;
  const out = await attempt(() => api.coins(p.character_id, op, amount), (r) => `${name(p)} now has ${num(r.coins)} coins`,
    before == null ? undefined : () => attempt(() => api.coins(p.character_id, 'set', before), `Coins back to ${num(before)}`).then(() => changed(p.character_id)));
  changed(p.character_id);
  return out;
}

export async function quest(p, op, quest_id, questName) {
  const label = questName || `quest ${quest_id}`;
  if ((op === 'reset' || op === 'abandon') && !(await ask({ title: `${op === 'reset' ? 'Reset' : 'Abandon'} ${label}?`,
    body: op === 'reset' ? 'Their progress on it is cleared and it starts again.' : 'It leaves their quest log.', confirm: op === 'reset' ? 'Reset' : 'Abandon', danger: true }))) return null;
  const out = await attempt(() => api.quests(p.character_id, op, Number(quest_id)), `${{ grant: 'Granted', complete: 'Completed', reset: 'Reset', abandon: 'Abandoned' }[op]} ${label}`);
  changed(p.character_id);
  return out;
}

export async function note(p, text) {
  const out = await attempt(() => api.notes(p.character_id, text), 'Note saved');
  changed(p.character_id);
  return out;
}

// World
export function audienceSize(to) {
  if (to === 'all') return live.online.length;
  if (to.zone != null) return live.online.filter((p) => String(p.zone) === String(to.zone)).length;
  return live.online.filter((p) => p.role === to.role).length;
}
export async function announce(to, text, style) {
  const n = audienceSize(to);
  const where = to === 'all' ? 'everyone online' : to.zone != null ? `zone ${ZONES().find(([z]) => z === String(to.zone))?.[1] || to.zone}` : `every ${to.role} online`;
  if (!(await ask({ title: `Announce to ${where}?`, body: `${num(n)} online now will see it as a ${style === 'banner' ? 'banner' : 'chat line'}: “${text}”`, confirm: 'Announce' }))) return null;
  return attempt(() => api.announce(to, text, style), (r) => `Delivered to ${num(r.delivered)} ${r.delivered === 1 ? 'player' : 'players'}`);
}

export function openMail(prefill) {
  pending.mail = prefill;
  if (location.hash === '#/mail') window.dispatchEvent(new HashChangeEvent('hashchange'));
  else location.hash = '#/mail';
}
