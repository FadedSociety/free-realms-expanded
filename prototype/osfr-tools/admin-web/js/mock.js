// Mock admin API (mock mode): answers every route from mock/*.json in the shapes the gateway serves (the core routes
// as G1 serves them, the mail, inventory and powers routes as the plan's contract has them), keeps writes in memory,
// and walks the online players about so the live views move. All names and items are invented.
import { score } from './dom.js';

const TIME_KEYS = new Set(['time', 'last_seen', 'created', 'sent', 'finished', 'muted_until', 'banned_until']);
export const FOREVER = 18446744073709551615; // "until lifted" (u64::MAX); any value past 1e15 reads that way
const PAGE = 50;
const RANK = { player: 0, tester: 1, builder: 1, moderator: 2, admin: 3 };

class MockError extends Error {
  constructor(status, message) { super(message); this.status = status; }
}
const bad = (m) => { throw new MockError(400, m); };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const now = () => Math.floor(Date.now() / 1000);
const hex = (n) => Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join('');
const intIn = (v, lo, hi, what) => {
  const n = Number(v);
  if (!Number.isInteger(n) || n < lo || n > hi) bad(`${what} must be a whole number from ${lo} to ${hi}`);
  return n;
};
const truthy = (v) => v === true || v === 1 || v === '1' || v === 'true';
const falsy = (v) => v === false || v === 0 || v === '0' || v === 'false';

// Shift every timestamp (unix seconds) so the data's anchor time reads as now.
function rebase(node, shift) {
  if (Array.isArray(node)) node.forEach((v) => rebase(v, shift));
  else if (node && typeof node === 'object') {
    for (const [k, v] of Object.entries(node)) {
      if (TIME_KEYS.has(k) && typeof v === 'number' && v < 1e15) node[k] = v + shift;
      else rebase(v, shift);
    }
  }
}

const SLOT_WORDS = [[/staff|bow|sword|wand|mitts|sling|pickaxe/i, 'weapon'], [/hat|helm|cap/i, 'head'],
  [/tunic|cloak/i, 'chest'], [/trousers/i, 'legs'], [/gloves/i, 'hands'], [/boots|sandals/i, 'feet'], [/buckler/i, 'offhand']];

export function createMock(source) {
  const data = structuredClone(source);
  rebase(data, now() - data.meta.anchor);
  const t0 = Date.now();
  const { me, players, inventory, items, quests, zones, audit } = data;
  const jobs = data.mail.jobs;
  const boxes = data.mail.mailboxes;
  const powers = { spectating: null, invisible: false, god: false, frozen: [] };
  const sessionBase = new Map(players.map((p) => [p.character_id, p.session_s]));
  const heading = new Map(players.map((p, i) => [p.character_id, (i * 2.39) % (Math.PI * 2)]));
  let signedOut = false;
  let lastStep = 0;
  let seq = 100;

  const player = (id) => players.find((p) => String(p.character_id) === String(id)) || (() => { throw new MockError(404, 'no such character'); })();
  const staffChar = () => players.find((p) => p.name === me.name) || players[0];
  const zoneOf = (z) => zones.find((x) => String(x.zone) === String(z) || x.zone_name.toLowerCase() === String(z).toLowerCase());
  const itemOf = (id) => items.find((i) => String(i.item_id) === String(id));
  const location = (p) => ({ online: p.online, zone: p.zone, zone_name: p.zone_name, position: p.position });
  const invOf = (id) => (inventory[String(id)] ||= { bags: [], equipped: [] });
  const target = (p) => ({ character_id: p.character_id, account_id: p.account, name: p.name });
  const log = (action, p, before, after, reason) => {
    const row = { time: now(), staff: { account_id: me.account, name: me.name }, action, ...(p ? { target: target(p) } : {}),
      before: before ?? {}, after: after ?? {}, ...(reason ? { reason } : {}), via: 'panel' };
    audit.unshift(row);
    return row;
  };

  // Online players wander: a small step along a slowly turning heading, at most once a second.
  function step() {
    if (Date.now() - lastStep < 1000) return;
    lastStep = Date.now();
    for (const p of players) {
      if (!p.online || powers.frozen.includes(p.character_id)) continue;
      const [x0, z0, x1, z1] = zoneOf(p.zone).bounds;
      let a = heading.get(p.character_id) + (Math.random() - 0.5) * 0.9;
      const speed = 4 + (p.character_id % 7) * 2;
      let x = p.position[0] + Math.cos(a) * speed;
      let z = p.position[2] + Math.sin(a) * speed;
      if (x < x0 * 0.92 || x > x1 * 0.92 || z < z0 * 0.92 || z > z1 * 0.92) { a += Math.PI; x = p.position[0]; z = p.position[2]; }
      heading.set(p.character_id, a);
      p.position = [Math.round(x * 10) / 10, p.position[1], Math.round(z * 10) / 10];
    }
  }

  const onlineRow = (p) => ({ character_id: p.character_id, name: p.name, account: p.account, role: p.role, zone: p.zone,
    zone_name: p.zone_name, position: p.position, job: p.job, level: p.level,
    session_s: (sessionBase.get(p.character_id) || 0) + Math.round((Date.now() - t0) / 1000), client: p.client });

  function recipients(to) {
    if (!to || typeof to !== 'object') bad('Pick who the mail goes to');
    if (Array.isArray(to.characters)) {
      if (!to.characters.length) bad('Pick at least one player');
      return to.characters.map((id) => player(id));
    }
    if (to.everyone === true) return players;
    if (to.online === true) return players.filter((p) => p.online);
    if (to.role) return players.filter((p) => p.role === to.role);
    if (to.zone != null) return players.filter((p) => String(p.zone) === String(to.zone));
    return bad('Unknown recipient kind');
  }

  const H = {
    loginChallenge(_, __, b) {
      if (!b.account) bad('account is required');
      return { nonce: hex(64), server_id: hex(64), expires_s: 60 };
    },
    login(_, __, b) {
      if (!b.account || !b.nonce) throw new MockError(401, 'login refused');
      signedOut = false;
      log('login', null, {}, {});
      return { token: hex(64), csrf: me.csrf, staff: { name: me.name, role: me.role, account: me.account }, expires: now() + 28800, expires_s: 28800 };
    },
    logout() { signedOut = true; return { ok: true }; },
    me: () => me,
    status() {
      const jitter = Math.sin(Date.now() / 7000) + Math.random() * 0.8;
      return { ...data.status, online: players.filter((p) => p.online).length,
        uptime_s: data.status.uptime_s + Math.round((Date.now() - t0) / 1000),
        tick_ms: { mean: +(data.status.tick_ms.mean + jitter).toFixed(1), max: +(data.status.tick_ms.max + jitter * 6 + Math.random() * 5).toFixed(1) },
        memory_mb: data.status.memory_mb + Math.round((Date.now() - t0) / 60000) };
    },
    online() { step(); return players.filter((p) => p.online).map(onlineRow).sort((a, b) => a.name.localeCompare(b.name)); },
    players(_, q) {
      const text = String(q.q || '').trim().toLowerCase().slice(0, 64);
      const num = /^\d+$/.test(text) ? Number(text) : null;
      let rows = players.filter((p) => (num !== null ? p.character_id === num || p.account === num : score(p.name, text) > 0)
        && (!truthy(q.online) || p.online) && (!falsy(q.online) || !p.online)
        && (!q.role || p.role === q.role) && (!truthy(q.banned) || p.sanctions.banned) && (!truthy(q.muted) || p.sanctions.muted));
      const key = { last_seen: (p) => -p.last_seen, created: (p) => -p.created }[q.sort];
      rows.sort(key ? (a, b) => key(a) - key(b) : (a, b) => score(b.name, text) - score(a.name, text) || a.name.localeCompare(b.name));
      const page = Math.max(1, parseInt(q.page, 10) || 1);
      const total = rows.length;
      rows = rows.slice((page - 1) * PAGE, page * PAGE).map((p) => ({ character_id: p.character_id, name: p.name, account: p.account,
        role: p.role, online: p.online, last_seen: p.online ? now() : p.last_seen, created: p.created, muted: p.sanctions.muted, banned: p.sanctions.banned }));
      return { total, page, page_size: PAGE, rows };
    },
    profile({ id }) {
      const p = player(id);
      return {
        identity: { character_id: p.character_id, name: p.name, first_name: p.first_name, last_name: p.last_name, account: p.account,
          account_characters: Object.fromEntries(players.filter((x) => x.account === p.account).map((x) => [String(x.character_id), x.name])),
          role: p.role, online: p.online, created: p.created, last_seen: p.online ? now() : p.last_seen, play_time_s: p.play_time_s },
        location: location(p), coins: p.coins, jobs: p.jobs, quests: p.quests, mounts: p.mounts, pets: p.pets,
        collections_n: p.collections_n, collections_done: p.collections_done, sanctions: p.sanctions,
        moderation: p.moderation.slice(0, 50), notes: p.notes.slice(0, 50),
      };
    },
    inventory: ({ id }) => (player(id), invOf(id)),
    addItem({ id }, _, b) {
      const p = player(id);
      const it = itemOf(b.item_id) || bad(`No item with id ${b.item_id}`);
      const count = intIn(b.count, 1, 999, 'Count');
      const inv = invOf(id);
      if (b.equip) {
        const slot = SLOT_WORDS.find(([re]) => re.test(it.name))?.[1] || bad(`${it.name} can't be equipped`);
        const old = inv.equipped.find((e) => e.slot === slot);
        inv.equipped = inv.equipped.filter((e) => e.slot !== slot);
        inv.equipped.push({ slot, item_id: it.item_id, name: it.name, count: 1, icon: null });
        if (old) inv.bags.push({ ...old, slot: nextSlot(inv) });
      } else {
        const stack = inv.bags.find((e) => e.item_id === it.item_id);
        if (stack) stack.count += count;
        else inv.bags.push({ slot: nextSlot(inv), item_id: it.item_id, name: it.name, count, icon: null });
      }
      log('inventory_add', p, {}, { item_id: it.item_id, count, equip: !!b.equip });
      return inv;
    },
    removeItems({ id }, _, b) {
      const p = player(id);
      const inv = invOf(id);
      if (!Array.isArray(b.items) || !b.items.length) bad('Pick at least one item');
      const removed = [];
      for (const r of b.items) {
        const e = 'slot' in r ? [...inv.bags, ...inv.equipped].find((x) => x.slot === r.slot)
          : [...inv.bags, ...inv.equipped].find((x) => String(x.item_id) === String(r.item_id));
        if (!e) bad('That item is no longer there');
        const n = r.count === 'all' ? e.count : intIn(r.count, 1, e.count, 'Count');
        e.count -= n;
        removed.push({ item_id: e.item_id, count: n });
      }
      inv.bags = inv.bags.filter((e) => e.count > 0);
      inv.equipped = inv.equipped.filter((e) => e.count > 0);
      log('inventory_remove', p, { items: removed }, {});
      return inv;
    },
    coins({ id }, _, b) {
      const p = player(id);
      const amount = intIn(b.amount, 0, 10_000_000, 'Amount');
      const before = p.coins;
      if (b.op === 'give') p.coins += amount;
      else if (b.op === 'take') { if (amount > p.coins) bad(`${p.name} has only ${p.coins} coins`); p.coins -= amount; }
      else if (b.op === 'set') p.coins = amount;
      else bad('op must be give, take or set');
      log('coins', p, { coins: before }, { coins: p.coins });
      return { coins: p.coins };
    },
    quests({ id }, _, b) {
      const p = player(id);
      const qid = Number(b.quest_id);
      if (!quests.some((x) => x.quest_id === qid)) bad(`No quest with id ${b.quest_id}`);
      const { active, done } = p.quests;
      const drop = (list) => list.filter((x) => x !== qid);
      if (b.op === 'grant') { if (active.includes(qid) || done.includes(qid)) bad(`${p.name} already has quest ${qid}`); active.push(qid); }
      else if (b.op === 'complete') { if (!active.includes(qid)) bad(`Quest ${qid} is not active`); p.quests = { active: drop(active), done: [qid, ...done] }; }
      else if (b.op === 'reset') p.quests = { active: [...drop(active), qid], done: drop(done) };
      else if (b.op === 'abandon') { if (!active.includes(qid)) bad(`Quest ${qid} is not active`); p.quests.active = drop(active); }
      else bad('op must be grant, complete, reset or abandon');
      log('quests', p, {}, { op: b.op, quest_id: qid });
      return p.quests;
    },
    moderate({ id }, _, b) {
      const p = player(id);
      const ops = ['mute', 'unmute', 'kick', 'ban', 'unban', 'warn', 'jail', 'release'];
      if (!ops.includes(b.op)) bad('unknown op');
      if (b.op === 'jail' || b.op === 'release') throw new MockError(501, `${b.op} is not built yet; it is wired in at integration`);
      const reason = String(b.reason ?? '').trim();
      if (reason.length > 500) bad('a reason is at most 500 characters');
      if (b.op === 'warn' && !reason) bad('a warning needs a reason');
      const minutes = b.minutes == null ? null : intIn(b.minutes, 1, 525600, 'minutes');
      if (minutes && b.op !== 'mute' && b.op !== 'ban') bad('minutes are only for mute and ban');
      if (p.account === me.account || (RANK[p.role] ?? 0) >= (RANK[me.role] ?? 0)) throw new MockError(403, 'you can only act on roles below yours');
      const s = p.sanctions;
      if (b.op === 'kick' && !p.online) throw new MockError(409, `${p.name} is not online`);
      if (b.op === 'unmute' && !s.muted) throw new MockError(409, `${p.name} is not muted`);
      if (b.op === 'unban' && !s.banned) throw new MockError(409, `${p.name} is not banned`);
      const before = Object.fromEntries(Object.entries(s).filter(([k, v]) => k.endsWith('_until') && v != null));
      const until = minutes ? now() + minutes * 60 : FOREVER;
      if (b.op === 'mute') Object.assign(s, { muted: true, muted_until: Math.max(s.muted_until ?? 0, until) });
      if (b.op === 'unmute') Object.assign(s, { muted: false, muted_until: null });
      if (b.op === 'ban') Object.assign(s, { banned: true, banned_until: Math.max(s.banned_until ?? 0, until) });
      if (b.op === 'unban') Object.assign(s, { banned: false, banned_until: null });
      if (b.op === 'kick' || b.op === 'ban') { p.online = false; p.last_seen = now(); }
      const after = b.op === 'warn' ? { delivered: p.online } : ['mute', 'unmute', 'ban', 'unban'].includes(b.op) ? { sanctions: { ...s }, minutes } : {};
      const row = log(b.op, p, before, after, reason);
      p.moderation.unshift(row);
      return { op: b.op, row, moderation: p.moderation.slice(0, 50) };
    },
    notes({ id }, _, b) {
      const p = player(id);
      const text = String(b.text || '').trim();
      if (!text || text.length > 500) bad('a note is 1 to 500 characters');
      p.notes.unshift(log('note', p, null, { text }));
      return p.notes.slice(0, 50);
    },
    move({ id }, _, b) {
      const p = player(id);
      const staff = staffChar();
      if (b.op === 'goto') { Object.assign(staff, location(p), { online: true }); log('move', p, {}, { op: 'goto' }); return location(staff); }
      if (b.op === 'bring') Object.assign(p, { zone: staff.zone, zone_name: staff.zone_name, position: staff.position.map((v, i) => (i === 1 ? v : v + 2)) });
      else if (b.op === 'send') {
        const to = b.to || bad('Send needs a destination');
        if (to.character_id != null) { const o = player(to.character_id); Object.assign(p, { zone: o.zone, zone_name: o.zone_name, position: o.position }); }
        else {
          const z = zoneOf(to.zone) || bad('Unknown zone');
          const ps = Array.isArray(to.position) && to.position.length === 3 && to.position.every(Number.isFinite) ? to.position : [0, 10, 0];
          Object.assign(p, { zone: z.zone, zone_name: z.zone_name, position: ps });
        }
      } else bad('op must be goto, bring or send');
      log('move', p, {}, { op: b.op, zone: p.zone_name });
      return location(p);
    },
    powers(_, __, b) {
      const tgt = b.target == null ? null : player(b.target);
      const need = () => tgt || bad('Pick a player');
      if (b.op === 'spectate') { powers.spectating = need().character_id; powers.invisible = true; }
      else if (b.op === 'unspectate') { powers.spectating = null; powers.invisible = false; }
      else if (b.op === 'invisible' || b.op === 'visible') powers.invisible = b.op === 'invisible';
      else if (b.op === 'freeze') { if (!powers.frozen.includes(need().character_id)) powers.frozen.push(tgt.character_id); }
      else if (b.op === 'unfreeze') powers.frozen = powers.frozen.filter((x) => x !== need().character_id);
      else if (b.op === 'god') powers.god = !powers.god;
      else bad('Unknown power');
      log('powers', tgt, {}, { op: b.op });
      return { state: powers };
    },
    items(_, q) {
      const text = String(q.q || '').trim().toLowerCase();
      return items.map((it) => [it, String(it.item_id).startsWith(text) ? 5 : score(it.name, text)])
        .filter(([, s]) => s > 0).sort((a, b) => b[1] - a[1] || a[0].name.localeCompare(b[0].name)).slice(0, 40).map(([it]) => it);
    },
    mailJobs() {
      return jobs.map((j) => {
        const done = j.start ? Math.min(j.total, Math.floor(((Date.now() - j.start) / 1000) * j.rate)) : j.done;
        const { start, rate, ...row } = j;
        return { ...row, done, state: done >= j.total ? 'done' : done ? 'running' : 'queued' };
      });
    },
    sendMail(_, __, b) {
      const subject = String(b.subject || '').trim();
      if (!subject || subject.length > 120) bad('A subject is 1 to 120 characters');
      if (String(b.body || '').length > 4000) bad('The body is at most 4000 characters');
      const list = recipients(b.to);
      if (!list.length) bad('Nobody matches those recipients');
      const att = { items: (b.attachments?.items || []).map((a) => ({ item_id: (itemOf(a.item_id) || bad(`No item with id ${a.item_id}`)).item_id,
        name: itemOf(a.item_id).name, count: intIn(a.count, 1, 999, 'Count') })), coins: intIn(b.attachments?.coins || 0, 0, 1_000_000, 'Coins') };
      const to = b.to;
      const summary = to.characters ? (list.length === 1 ? list[0].name : `${list.length} players`)
        : to.everyone ? 'Everyone' : to.online ? 'Online' : to.role ? `Role: ${to.role}` : `Zone: ${zoneOf(to.zone)?.zone_name || to.zone}`;
      const job_id = `mj-${String(++seq).padStart(4, '0')}`;
      jobs.unshift({ job_id, to_summary: summary, total: list.length, done: 0, state: 'queued', subject, created: now(),
        start: Date.now() + 600, rate: Math.max(4, list.length / 6) });
      for (const p of list) {
        (boxes[String(p.character_id)] ||= []).unshift({ mail_id: `m-${p.character_id}-${seq}`, from: me.name, subject, sent: now(), read: false, attachments: att });
      }
      log('mail', null, {}, { to: summary, subject, recipients: list.length });
      return { job_id, recipients: list.length };
    },
    playerMail: ({ id }) => (player(id), boxes[String(id)] || []),
    announce(_, __, b) {
      const text = String(b.text || '').trim();
      if (!text || text.length > 200) bad('an announcement is 1 to 200 characters');
      const style = b.style ?? 'chat';
      if (!['banner', 'chat'].includes(style)) bad('style is chat or banner');
      const online = players.filter((p) => p.online);
      const to = b.to;
      const list = to === 'all' ? online : to?.zone != null ? online.filter((p) => String(p.zone) === String(to.zone))
        : to?.role ? online.filter((p) => p.role === to.role) : bad('to is "all", {zone} or {role}');
      log('announce', null, {}, { to, style, text, delivered: list.length });
      return { delivered: list.length };
    },
    audit(_, q) {
      const since = Number(q.since);
      const idOrText = (v) => (/^\d+$/.test(String(v)) ? Number(v) : String(v || '').toLowerCase());
      const st = idOrText(q.staff);
      const pl = idOrText(q.player);
      const rows = audit.filter((r) => (!q.staff || (typeof st === 'number' ? r.staff.account_id === st : r.staff.name.toLowerCase().includes(st)))
        && (!q.player || (r.target && (typeof pl === 'number' ? r.target.character_id === pl || r.target.account_id === pl : r.target.name.toLowerCase().includes(pl))))
        && (!q.action || r.action === q.action) && (!q.since || !Number.isFinite(since) || r.time >= since));
      const page = Math.max(1, parseInt(q.page, 10) || 1);
      return { total: rows.length, page, page_size: PAGE, rows: rows.slice((page - 1) * PAGE, page * PAGE) };
    },
    map({ zone }) {
      step();
      const z = zoneOf(zone) || (() => { throw new MockError(404, 'No such zone'); })();
      return { image: z.image, bounds: z.bounds, players: players.filter((p) => p.online && p.zone === z.zone)
        .map((p) => ({ character_id: p.character_id, name: p.name, position: p.position })) };
    },
  };

  function nextSlot(inv) {
    const used = new Set(inv.bags.map((e) => e.slot));
    let s = 0;
    while (used.has(s)) s++;
    return s;
  }

  return {
    async handle(name, params, query, body) {
      await sleep(name === 'online' || name === 'status' || name === 'map' ? 30 : 80 + Math.random() * 140);
      if (signedOut && !['login', 'loginChallenge'].includes(name)) throw new MockError(401, 'sign in first');
      const fn = H[name] || (() => { throw new MockError(501, `${name} is not in the mock`); });
      return structuredClone(fn(params, query, body || {}));
    },
    signOut() { signedOut = true; },
  };
}
