// Every staff action, shared by the buttons and the command bar. Destructive ones confirm first; each one toasts
// its result, with Undo where the API has one: an edit's undo key (add, remove, coins, quests, jobs: POST /undo) or
// the inverse op (mute/unmute, ban/unban, jail/release, freeze/unfreeze).
import { api } from './api.js';
import { ask, toast, fail, dur, num, zoneLabel, questLabel } from './dom.js';
import { live, notify } from './live.js';

export const me = { name: '', role: '', permissions: null };
// The server enforces roles; the panel only greys out what /me's permission list leaves out. A list that uses none
// of these names (the gateway named its actions differently) greys out nothing rather than everything.
const PERMS = new Set(['view', 'mute', 'unmute', 'kick', 'ban', 'unban', 'warn', 'jail', 'release', 'note', 'mail', 'announce',
  'inventory', 'coins', 'quests', 'jobs', 'move', 'powers']);
export const can = (perm) => {
  const list = me.permissions;
  return !Array.isArray(list) || list.includes('*') || list.includes(perm) || !list.some((x) => PERMS.has(x));
};

export const changed = (id) => window.dispatchEvent(new CustomEvent('admin:changed', { detail: { id } }));
export const pending = { mail: null };
const name = (p) => p.name || `#${p.character_id}`;

// `undo` is a no-argument inverse, or `(out) => ...` for one that needs the answer (an edit's undo key): that one
// is offered only when the answer has a key.
async function attempt(fn, ok, undo) {
  try {
    const out = await fn();
    const offer = undo && (undo.length ? (out?.undo ? () => undo(out) : undefined) : undo);
    if (ok) toast(typeof ok === 'function' ? ok(out) : ok, { undo: offer });
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
  // The gateway moves a player within the open world (zone 0) to a position.
  const v = await ask({ title: `Send ${name(p)} somewhere`, body: 'Within the open world, to a position (the map and the online list show positions).',
    confirm: 'Send', fields: [
      { name: 'zone', label: 'Zone', type: zones.length ? 'select' : 'text', options: zones, value: '0', required: true },
      { name: 'xyz', label: 'Position x, y, z', placeholder: '-1415, -28, 352', required: true }] });
  if (!v) return null;
  const xyz = v.xyz.split(/[\s,]+/).filter(Boolean).map(Number);
  if (xyz.length !== 3 || !xyz.every(Number.isFinite)) { fail(new Error('The position is three numbers: x, y, z')); return null; }
  const to = { zone: /^\d+$/.test(v.zone) ? Number(v.zone) : v.zone, position: xyz };
  return attempt(() => api.move(p.character_id, 'send', to), (loc) => `Sent ${name(p)} to ${loc?.zone_name || 'the zone'}`)
    .then((r) => (changed(p.character_id), r));
}

// Powers. The gateway answers each with its state: the staff member's own character (character_id null when it is
// not in the game: then only freeze, unfreeze and send work) and every frozen or jailed player.
export const frozen = new Set();
export const self = { invisible: false, flying: false, inGame: false };
export let spectating = null;
function takeState(state) {
  if (!state) return;
  live.powers = state;
  self.invisible = !!state.invisible;
  self.flying = !!state.flying;
  self.inGame = state.character_id != null;
  frozen.clear();
  for (const id of state.frozen || []) frozen.add(id);
  const watched = state.spectating;
  spectating = watched == null ? null : spectating?.character_id === watched ? spectating
    : live.online.find((x) => x.character_id === watched) || { character_id: watched, name: `#${watched}` };
  notify();
}
export async function refreshPowers() {
  try { takeState((await api.powerState()).state); } catch { /* the live poll shows connection errors */ }
}
export async function power(op, p) {
  const out = await attempt(() => api.powers(op, p?.character_id), (r) => r?.message || {
    spectate: `Spectating ${p && name(p)}. You are hidden from other players.`, unspectate: 'Stopped spectating',
    invisible: 'You are invisible', visible: 'You are visible again', fly: 'Flight on', land: 'Flight off',
    freeze: `Froze ${p && name(p)}`, unfreeze: `Unfroze ${p && name(p)}` }[op],
  op === 'freeze' ? () => power('unfreeze', p) : op === 'unfreeze' ? () => power('freeze', p) : undefined);
  if (out) takeState(out.state);
  if (p) changed(p.character_id);
  return out;
}

// An edit's undo by the key its answer or audit row holds.
export async function undoEdit(row) {
  const key = row?.after?.undo ?? row?.undo;
  const who = row?.target?.character_id ?? row?.character_id;
  const out = await attempt(() => api.undo(key), 'Undone', (r) => undoEdit(r));
  if (who != null) changed(who);
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
  const out = await attempt(() => api.addItem(p.character_id, item.item_id, count, equip), text, (r) => undoEdit({ ...r, character_id: p.character_id }));
  changed(p.character_id);
  return out;
}
// entries: inventory rows [{slot, item_id, name, count, equipped?}] with the count to remove ("all" or a number).
export async function removeItems(p, entries, { confirmed = false } = {}) {
  const what = entries.length === 1 ? `${entries[0].remove === 'all' ? '' : `${entries[0].remove} × `}${entries[0].name || entries[0].item_id}` : `${entries.length} stacks`;
  if (!confirmed && !(await ask({ title: `Remove ${what}?`, body: `From ${name(p)}'s ${entries.some((e) => e.equipped) ? 'equipment and bags' : 'bags'}.`, confirm: 'Remove', danger: true }))) return null;
  // A bag stack by its slot (the stack's instance id); a worn item by the bag stack it wears.
  const pick = (e) => (e.equipped ? (e.instance != null ? { slot: e.instance } : { item_id: e.item_id }) : e.slot != null ? { slot: e.slot } : { item_id: e.item_id });
  const body = entries.map((e) => ({ ...pick(e), count: e.remove }));
  const out = await attempt(() => api.removeItems(p.character_id, body), `Removed ${what}`, (r) => undoEdit({ ...r, character_id: p.character_id }));
  changed(p.character_id);
  return out;
}

export async function coins(p, op, amount, before) {
  amount = Number(amount);
  if (op !== 'give' && !(await ask({ title: op === 'set' ? `Set ${name(p)}'s coins to ${num(amount)}?` : `Take ${num(amount)} coins from ${name(p)}?`, confirm: op === 'set' ? 'Set' : 'Take', danger: true }))) return null;
  const out = await attempt(() => api.coins(p.character_id, op, amount), (r) => `${name(p)} now has ${num(r.coins)} coins`,
    (r) => undoEdit({ ...r, character_id: p.character_id }));
  changed(p.character_id);
  return out;
}

export async function quest(p, op, quest_id, questName) {
  const label = questName || questLabel(quest_id);
  if ((op === 'reset' || op === 'abandon') && !(await ask({ title: `${op === 'reset' ? 'Reset' : 'Abandon'} ${label}?`,
    body: op === 'reset' ? 'Their progress on it is cleared and it starts again.' : 'It leaves their quest log.', confirm: op === 'reset' ? 'Reset' : 'Abandon', danger: true }))) return null;
  const out = await attempt(() => api.quests(p.character_id, op, Number(quest_id)), `${{ grant: 'Granted', complete: 'Completed', reset: 'Reset', abandon: 'Abandoned' }[op]} ${label}`,
    (r) => undoEdit({ ...r, character_id: p.character_id }));
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
