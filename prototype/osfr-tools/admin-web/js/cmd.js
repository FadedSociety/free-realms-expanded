// The command bar (Ctrl+K or /): every action by name, with autocomplete over actions, players, items and zones.
// Each command runs the same function as its button (actions.js).
import { api } from './api.js';
import { h, clear, icon, modal, debounce, fail, toggleTheme, score, roleName } from './dom.js';
import * as act from './actions.js';
import { live } from './live.js';
import { openPlayer } from './drawer.js';
import { itemArt } from './picker.js';

const NAV = [['dashboard', 'Dashboard'], ['players', 'Players'], ['mail', 'Mail'], ['moderation', 'Moderation'],
  ['world', 'World'], ['audit', 'Audit'], ['server', 'Server']];
const MOD = (op, args, help) => ({ name: op, args, help, perm: op, run: ([p, a, b]) => act.moderate(p, op, op === 'mute' || op === 'ban' ? { minutes: a, reason: b } : { reason: a }) });
const entryOf = async (p, it) => {
  const inv = await api.inventory(p.character_id);
  const e = inv.equipped.find((x) => String(x.item_id) === String(it.item_id));
  return e ? { ...e, equipped: true } : inv.bags.find((x) => String(x.item_id) === String(it.item_id));
};

export const COMMANDS = [
  { name: 'open', args: ['player'], help: 'Open a player', run: ([p]) => openPlayer(p.character_id) },
  { name: 'goto', args: ['player'], help: 'Teleport to a player', perm: 'move', run: ([p]) => act.move(p, 'goto') },
  { name: 'bring', args: ['player'], help: 'Bring a player to you', perm: 'move', run: ([p]) => act.move(p, 'bring') },
  { name: 'send', args: ['player'], help: 'Send a player to a zone or position', perm: 'move', run: ([p]) => act.sendFlow(p) },
  { name: 'spectate', args: ['player'], help: 'Watch a player; you are hidden while you do', perm: 'powers', run: ([p]) => act.power('spectate', p) },
  { name: 'unspectate', args: [], help: 'Stop spectating', perm: 'powers', run: () => act.power('unspectate') },
  { name: 'freeze', args: ['player'], help: 'Hold a player in place', perm: 'powers', run: ([p]) => act.power('freeze', p) },
  { name: 'unfreeze', args: ['player'], help: 'Let a frozen player move again', perm: 'powers', run: ([p]) => act.power('unfreeze', p) },
  { name: 'invisible', args: [], help: 'Hide yourself from other players', perm: 'powers', run: () => act.power('invisible') },
  { name: 'visible', args: [], help: 'Show yourself again', perm: 'powers', run: () => act.power('visible') },
  { name: 'god', args: [], help: 'Toggle god mode (you take no damage)', perm: 'powers', run: () => act.power('god') },
  { name: 'give', args: ['player', 'item', 'count?'], help: 'Give an item, e.g. give Pip 12345 5', perm: 'inventory', run: ([p, it, n]) => act.giveItem(p, it, n || 1) },
  { name: 'take', args: ['player', 'item', 'count?'], help: 'Take an item away (all of it unless you give a count)', perm: 'inventory',
    run: async ([p, it, n]) => {
      const e = await entryOf(p, it);
      if (!e) throw new Error(`${p.name} has no ${it.name || `item ${it.item_id}`}`);
      return act.removeItems(p, [{ ...e, remove: n && n < e.count ? n : 'all' }]);
    } },
  { name: 'coins', args: ['player', ['give', 'take', 'set'], 'int'], help: 'Change coins, e.g. coins Pip give 500', perm: 'coins',
    run: async ([p, op, n]) => act.coins(p, op, n, (await api.profile(p.character_id)).coins) },
  MOD('mute', ['player', 'minutes?', 'text?'], 'Mute, e.g. mute Pip 30m spamming'),
  MOD('unmute', ['player', 'text?'], 'Lift a mute'),
  MOD('kick', ['player', 'text?'], 'Disconnect a player'),
  MOD('ban', ['player', 'minutes?', 'text?'], 'Ban, e.g. ban Pip 7d cheating, or ban Pip perm'),
  MOD('unban', ['player', 'text?'], 'Lift a ban'),
  MOD('warn', ['player', 'text?'], 'Warn with a popup, e.g. warn Pip please keep chat friendly'),
  MOD('jail', ['player', 'text?'], 'Send to jail'),
  MOD('release', ['player', 'text?'], 'Release from jail'),
  { name: 'quest', args: ['player', ['grant', 'complete', 'reset', 'abandon'], 'int'], help: 'Quest edits, e.g. quest Pip grant 314', perm: 'quests', run: ([p, op, id]) => act.quest(p, op, id) },
  { name: 'note', args: ['player', 'text'], help: 'Add a staff note', perm: 'note', run: ([p, t]) => act.note(p, t) },
  { name: 'mail', args: ['target', 'text?'], help: 'Write mail: mail Pip | all | online | role:tester | zone:Seaside, then a subject', perm: 'mail',
    run: ([to, s]) => act.openMail({ to: [to], subject: s }) },
  { name: 'announce', args: ['text'], help: 'Banner to everyone online', perm: 'announce', run: ([t]) => act.announce('all', t, 'banner') },
  { name: 'say', args: ['text'], help: 'Chat line to everyone online', perm: 'announce', run: ([t]) => act.announce('all', t, 'chat') },
  { name: 'theme', args: [], help: 'Switch between dark and light', run: () => toggleTheme() },
  ...NAV.map(([k, l]) => ({ name: k, args: [], help: `Go to ${l}`, nav: true, run: () => { location.hash = `#/${k}`; } })),
];

// Players and items seen so far, so names with spaces can be read back out of the typed text.
const players = new Map();
const items = new Map();
export const remember = (list) => list.forEach((p) => p?.name && players.set(p.name.toLowerCase(), { ...players.get(p.name.toLowerCase()), ...p }));
const rememberItems = (list) => list.forEach((i) => items.set(i.name.toLowerCase(), i));

const argName = (k) => (Array.isArray(k) ? k.join('|') : { player: 'player', item: 'item', int: 'number', count: 'count', minutes: 'time', text: 'text', target: 'who' }[k] || k);
const baseOf = (k) => (typeof k === 'string' && k.endsWith('?') ? k.slice(0, -1) : k);
export function usage(c) {
  return [c.name, ...c.args.map((k) => (typeof k === 'string' && k.endsWith('?') ? `[${argName(baseOf(k))}]` : `<${argName(k)}>`))].join(' ');
}
export function parseMinutes(t) {
  if (/^(perm|permanent|forever)$/i.test(t)) return '';
  const m = /^(\d+)(m|min|h|d|w)?$/i.exec(t);
  return m ? Number(m[1]) * ({ h: 60, d: 1440, w: 10080 }[(m[2] || 'm')[0].toLowerCase()] || 1) : null;
}

function zoneByName(z) {
  const zl = z.toLowerCase();
  return act.ZONES().find(([id, n]) => id === z || n.toLowerCase() === zl);
}

// Reads one argument of `kind` starting at token i. Returns {value, end} or null.
function consume(kind, toks, i, final) {
  const t = toks[i];
  if (kind === 'player') {
    for (let j = Math.min(toks.length, i + 4); j > i; j--) {
      const p = players.get(toks.slice(i, j).join(' ').toLowerCase());
      if (p) return { value: p, end: j };
    }
    const tl = t.toLowerCase();
    const firsts = [...players.values()].filter((p) => p.name.split(' ')[0].toLowerCase() === tl || String(p.account ?? '').toLowerCase() === tl);
    if (firsts.length === 1) return { value: firsts[0], end: i + 1 };
    return final && t ? { value: { raw: t }, end: i + 1 } : null;
  }
  if (kind === 'item') {
    if (/^\d+$/.test(t)) return { value: { item_id: Number(t), name: [...items.values()].find((x) => String(x.item_id) === t)?.name }, end: i + 1 };
    for (let j = Math.min(toks.length, i + 5); j > i; j--) {
      const it = items.get(toks.slice(i, j).join(' ').toLowerCase());
      if (it) return { value: it, end: j };
    }
    return final && t ? { value: { raw: t }, end: i + 1 } : null;
  }
  if (kind === 'int' || kind === 'count') return /^\d+$/.test(t) ? { value: Number(t), end: i + 1 } : null;
  if (kind === 'minutes') { const m = parseMinutes(t); return m === null ? null : { value: m, end: i + 1 }; }
  if (Array.isArray(kind)) { const o = kind.find((x) => x === t.toLowerCase()); return o ? { value: o, end: i + 1 } : null; }
  if (kind === 'target') {
    const tl = t.toLowerCase();
    if (tl === 'all' || tl === 'everyone') return { value: { kind: 'everyone', label: 'Everyone' }, end: i + 1 };
    if (tl === 'online') return { value: { kind: 'online', label: 'Online' }, end: i + 1 };
    const role = /^role:(\w+)$/i.exec(t);
    const r = role && act.ROLES.find((x) => x.toLowerCase() === role[1].toLowerCase());
    if (r) return { value: { kind: 'role', role: r, label: `Role: ${roleName(r)}` }, end: i + 1 };
    const zone = /^zone:(.+)$/i.exec(t);
    const z = zone && zoneByName(zone[1]);
    if (z) return { value: { kind: 'zone', zone: z[0], label: `Zone: ${z[1]}` }, end: i + 1 };
    const p = consume('player', toks, i, final);
    return p && { value: p.value.raw ? p.value : { kind: 'player', id: p.value.character_id, label: p.value.name }, end: p.end };
  }
  if (kind === 'text') { const s = toks.slice(i).join(' ').trim(); return s ? { value: s, end: toks.length } : null; }
  return null;
}

// Typing: returns where the cursor is (stage = arg index, or 'command'/'done'). final: returns args or an error.
export function parse(text, final = false) {
  const toks = text.replace(/^\s+/, '').split(/\s+/);
  const cmd = COMMANDS.find((c) => c.name === toks[0].toLowerCase());
  if (!cmd || (toks.length === 1 && !final)) return { cmd: null, stage: 'command', query: text.trim(), exact: cmd };
  const args = [];
  let i = 1;
  for (let k = 0; k < cmd.args.length; k++) {
    const kind = baseOf(cmd.args[k]);
    const opt = kind !== cmd.args[k];
    const typing = { cmd, args, stage: k, kind, start: i, query: toks.slice(i).join(' ') };
    if (i >= toks.length || (i === toks.length - 1 && toks[i] === '')) {
      if (!final) return { ...typing, query: '' };
      if (opt) { args.push(undefined); continue; }
      return { cmd, error: `${usage(cmd)}: the ${argName(kind)} is missing` };
    }
    if (!final && kind === 'player') {
      const rest = typing.query.toLowerCase();
      if (rest.includes(' ') && [...players.keys()].some((n) => n.startsWith(rest) && n !== rest)) return typing;
    }
    const r = consume(kind, toks, i, final);
    if (!r) {
      if (opt) { args.push(undefined); continue; }
      if (!final) return typing;
      return { cmd, error: `Can't read "${toks[i]}" as a ${argName(kind)}. Use: ${usage(cmd)}` };
    }
    if (!final && r.end >= toks.length) return typing;
    args.push(r.value);
    i = r.end;
  }
  return { cmd, args, stage: 'done' };
}

async function resolve(cmd, args) {
  return Promise.all(args.map(async (a, k) => {
    if (!a?.raw) return a;
    const kind = baseOf(cmd.args[k]);
    if (kind === 'item') {
      const list = await api.items(a.raw);
      const it = list.find((x) => x.name.toLowerCase() === a.raw.toLowerCase()) || (list.length === 1 && list[0]);
      if (!it) throw new Error(list.length ? `Several items match "${a.raw}": ${list.slice(0, 4).map((x) => x.name).join(', ')}` : `No item matches "${a.raw}"`);
      return it;
    }
    const { rows } = await api.players({ q: a.raw });
    remember(rows);
    const rl = a.raw.toLowerCase();
    const exact = rows.filter((p) => p.name.toLowerCase() === rl || p.name.split(' ')[0].toLowerCase() === rl || String(p.account ?? '').toLowerCase() === rl);
    const p = exact.length === 1 ? exact[0] : rows.length === 1 ? rows[0] : null;
    if (!p) throw new Error(rows.length ? `Several players match "${a.raw}": ${rows.slice(0, 4).map((x) => x.name).join(', ')}` : `No player matches "${a.raw}"`);
    return kind === 'target' ? { kind: 'player', id: p.character_id, label: p.name } : p;
  }));
}

export function openCommandBar(initial = '') {
  remember(live.online);
  return modal('modal-cmd', (close) => {
    const input = h('input', { class: 'cmd-input', type: 'text', value: initial, placeholder: 'Type a command or a player name', 'aria-label': 'Command',
      autocomplete: 'off', spellcheck: 'false', role: 'combobox', 'aria-expanded': 'true', 'aria-controls': 'cmd-list', 'aria-autocomplete': 'list' });
    const list = h('ul', { class: 'cmd-list', id: 'cmd-list', role: 'listbox', 'aria-label': 'Suggestions' });
    const hint = h('div', { class: 'cmd-hint' });
    let sugg = [];
    let hi = 0;
    let lastQuery = null;

    const replaceFrom = (start, value) => {
      const toks = input.value.replace(/^\s+/, '').split(/\s+/);
      return [...toks.slice(0, start), value].join(' ') + ' ';
    };
    function paint() {
      clear(list, sugg.map((s, k) => {
        const li = h('li', { class: 'cmd-item', role: 'option', id: `cmd-opt-${k}`, 'aria-selected': String(k === hi),
          onmousedown: (e) => { e.preventDefault(); hi = k; accept(true); } },
        s.art || icon(s.icon || 'bolt', 16), h('span', { class: 'cmd-label' }, s.label), s.hint ? h('span', { class: 'cmd-sub' }, s.hint) : null,
        s.disabled ? h('span', { class: 'cmd-sub' }, 'No permission') : null);
        return li;
      }));
      input.setAttribute('aria-activedescendant', sugg.length ? `cmd-opt-${hi}` : '');
      list.querySelector('[aria-selected="true"]')?.scrollIntoView({ block: 'nearest' });
    }
    const searchPlayers = debounce(async (q) => {
      try { const { rows } = await api.players({ q }); remember(rows); if (lastQuery === q) update(true); } catch { /* suggestions only */ }
    }, 120);
    const searchItems = debounce(async (q) => {
      try { const l = await api.items(q); rememberItems(l); itemHits = l; if (lastQuery === q) update(true); } catch { /* suggestions only */ }
    }, 120);
    let itemHits = [];

    const playerSugg = (q, make) => [...players.values()].map((p) => [p, q ? Math.max(score(p.name, q), score(String(p.account ?? ''), q)) : 1])
      .filter(([, s]) => s > 0).sort((a, b) => b[1] - a[1] || (b[0].online ?? 1) - (a[0].online ?? 1) || a[0].name.localeCompare(b[0].name))
      .slice(0, 7).map(([p]) => make(p));
    const where = (p) => (p.zone_name || p.zone != null ? (p.zone_name || `Zone ${p.zone}`) : p.online === false ? 'Offline' : '');

    function update(fromSearch = false) {
      const st = parse(input.value);
      const q = (st.query || '').toLowerCase();
      if (!fromSearch && q !== lastQuery) {
        lastQuery = q;
        if (st.stage === 'command' || st.kind === 'player' || st.kind === 'target') { if (q) searchPlayers(q); }
        if (st.kind === 'item') searchItems(q);
      }
      if (st.stage === 'command') {
        const cmds = COMMANDS.map((c) => [c, score(c.name, q.split(' ')[0])]).filter(([, s]) => s > 1 || !q).sort((a, b) => b[1] - a[1] || (a[0].nav ? 1 : 0) - (b[0].nav ? 1 : 0));
        sugg = [
          ...cmds.slice(0, q ? 6 : 12).map(([c]) => ({ label: usage(c), hint: c.help, icon: c.nav ? 'chevron' : 'bolt', disabled: c.perm && !act.can(c.perm),
            text: `${c.name} `, run: !c.args.length })),
          ...(q ? playerSugg(q, (p) => ({ label: `Open ${p.name}`, hint: where(p), icon: 'players', text: `open ${p.name} `, run: true })) : []),
        ];
        clear(hint, 'Type a command, or a player name to open them.');
      } else if (st.stage === 'done') {
        sugg = [];
        clear(hint, h('strong', {}, usage(st.cmd)), ` · ${st.cmd.help}. Press Enter to run.`);
      } else {
        const k = st.kind;
        const at = (value) => replaceFrom(st.start, value);
        if (k === 'player') sugg = playerSugg(q, (p) => ({ label: p.name, hint: where(p), icon: 'players', text: at(p.name) }));
        else if (k === 'item') sugg = itemHits.slice(0, 8).map((it) => ({ label: it.name, hint: `#${it.item_id}`, art: itemArt(it, 'xs'), text: at(String(it.item_id)) }));
        else if (Array.isArray(k)) sugg = k.filter((o) => o.startsWith(q)).map((o) => ({ label: o, icon: 'chevron', text: at(o) }));
        else if (k === 'minutes') sugg = ['15m', '1h', '4h', '1d', '7d', ...(st.cmd.name === 'ban' ? ['perm'] : [])].filter((o) => o.startsWith(q)).map((o) => ({ label: o, icon: 'chevron', text: at(o) }));
        else if (k === 'target') {
          const zones = [...new Set(live.online.filter((p) => p.zone != null).map((p) => `zone:${p.zone_name ?? p.zone}`))];
          sugg = [...['all', 'online', ...act.ROLES.map((r) => `role:${r}`), ...zones].filter((o) => o.toLowerCase().startsWith(q)).slice(0, 6)
            .map((o) => ({ label: o, icon: 'mail', text: at(o) })),
          ...playerSugg(q, (p) => ({ label: p.name, hint: where(p), icon: 'players', text: at(p.name) }))];
        } else sugg = [];
        const parts = [st.cmd.name, ...st.cmd.args.map((a, i) => {
          const s = typeof a === 'string' && a.endsWith('?') ? `[${argName(baseOf(a))}]` : `<${argName(a)}>`;
          return i === st.stage ? h('mark', {}, s) : s;
        })];
        clear(hint, parts.flatMap((p, i) => (i ? [' ', p] : [p])), ` · ${st.cmd.help}`);
      }
      hi = Math.min(hi, Math.max(0, sugg.length - 1));
      paint();
    }

    async function run() {
      const st = parse(input.value, true);
      if (st.stage === 'command') {
        if (!st.exact && !sugg.length) return clear(hint, h('span', { class: 'cmd-error' }, `Unknown command "${input.value.trim()}". Try "goto", "give" or a player name.`));
        if (sugg.length) return accept(true);
      }
      if (st.error) return clear(hint, h('span', { class: 'cmd-error' }, st.error));
      const cmd = st.cmd || st.exact;
      if (cmd.perm && !act.can(cmd.perm)) return clear(hint, h('span', { class: 'cmd-error' }, 'Your role cannot do that.'));
      let args;
      try { args = await resolve(cmd, st.args || []); } catch (e) { return clear(hint, h('span', { class: 'cmd-error' }, e.message)); }
      close(null);
      try { await cmd.run(args); } catch (e) { fail(e); }
    }
    function accept(andRun) {
      const s = sugg[hi];
      if (!s) return run();
      input.value = s.text;
      hi = 0;
      update();
      if (andRun && (s.run || parse(input.value).stage === 'done')) run();
      input.focus();
    }

    input.addEventListener('input', () => { hi = 0; update(); });
    input.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
        e.preventDefault();
        if (sugg.length) { hi = (hi + (e.key === 'ArrowDown' ? 1 : -1) + sugg.length) % sugg.length; paint(); }
      } else if (e.key === 'Tab' && sugg.length) { e.preventDefault(); accept(false); }
      else if (e.key === 'Enter') { e.preventDefault(); if (sugg.length && parse(input.value).stage !== 'done') accept(true); else run(); }
    });
    queueMicrotask(() => { input.focus(); input.setSelectionRange(input.value.length, input.value.length); update(); });
    return h('div', { class: 'cmd' },
      h('div', { class: 'cmd-bar' }, icon('search', 20), input, h('kbd', {}, 'Esc')),
      list, hint,
      h('div', { class: 'cmd-keys' }, h('span', {}, h('kbd', {}, '↑'), h('kbd', {}, '↓'), ' move'), h('span', {}, h('kbd', {}, 'Tab'), ' complete'),
        h('span', {}, h('kbd', {}, 'Enter'), ' run'), h('span', {}, h('kbd', {}, 'Esc'), ' close')));
  });
}
