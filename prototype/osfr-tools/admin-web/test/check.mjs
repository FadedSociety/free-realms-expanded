// Checks for the admin panel, no browser needed: `node prototype/osfr-tools/admin-web/test/check.mjs`.
// 1. every module under js/ loads; 2. api.js has exactly the contract's routes (PLAN section 7, plus the login
// challenge and logout the gateway added); 3. the mock answers every route with the served fields; 4. the Ed25519
// sign-in matches Node's own signer; 5. the command bar reads commands back; 6. permission greying;
// 7. no HTML parsing of data and no requests to other hosts.
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join, dirname, relative, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createPrivateKey, createPublicKey, sign, randomBytes } from 'node:crypto';
import assert from 'node:assert/strict';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const walk = (d) => readdirSync(d).flatMap((f) => (statSync(join(d, f)).isDirectory() ? walk(join(d, f)) : [join(d, f)]));
const files = walk(root).filter((f) => !relative(root, f).startsWith(`test${sep}`));
const load = (rel) => import(pathToFileURL(join(root, rel)));
let checks = 0;
const ok = (cond, msg) => { assert.ok(cond, msg); checks++; };

// 1. Every module loads.
for (const f of files.filter((f) => f.endsWith('.js'))) await import(pathToFileURL(f));
checks++;

// 2. Routes match the contract exactly (path parameter names normalised to {id}).
const CONTRACT = `
POST /admin/api/login/challenge
POST /admin/api/login
POST /admin/api/logout
GET /admin/api/me
GET /admin/api/status
GET /admin/api/players/online
GET /admin/api/players
GET /admin/api/players/{character_id}
GET /admin/api/players/{id}/inventory
POST /admin/api/players/{id}/inventory/add
POST /admin/api/players/{id}/inventory/remove
POST /admin/api/players/{id}/coins
POST /admin/api/players/{id}/quests
POST /admin/api/players/{id}/moderate
POST /admin/api/players/{id}/notes
POST /admin/api/players/{id}/move
POST /admin/api/powers
GET /admin/api/items
GET /admin/icon/{item_id}
GET /admin/api/mail/jobs
POST /admin/api/mail
GET /admin/api/players/{id}/mail
POST /admin/api/announce
GET /admin/api/audit
GET /admin/api/map/{zone}
GET /admin/api/events`.trim().split('\n').map((l) => l.replace('{character_id}', '{id}'));
const { ROUTES, MOCK_FILES, api, signChallenge } = await load('js/api.js');
assert.deepEqual(Object.values(ROUTES).map(([m, p]) => `${m} ${p}`).sort(), [...CONTRACT].sort(), 'api.js routes differ from the contract');
ok(Object.keys(ROUTES).length === CONTRACT.length, 'one name per route');
for (const name of Object.keys(ROUTES)) ok(name in api || ['loginChallenge', 'login', 'logout', 'icon', 'events'].includes(name), `api.${name} is missing`);

// 3. The mock answers every route with the served fields.
const { createMock } = await load('js/mock.js');
const data = Object.fromEntries(MOCK_FILES.map((f) => [f, JSON.parse(readFileSync(join(root, 'mock', `${f}.json`), 'utf8'))]));
const mock = createMock(data);
const has = (obj, keys, what) => keys.forEach((k) => ok(obj && k in obj, `${what} lacks ${k}`));
const call = (name, params = {}, query = {}, body) => mock.handle(name, params, query, body);
const online = await call('online');
const pip = online.find((p) => p.name === 'Pip Mossberry');
ok(pip, 'Pip Mossberry is online in the mock');
const id = { id: pip.character_id };
const ch = await call('loginChallenge', {}, {}, { account: 'marigold' });
has(ch, ['nonce', 'server_id', 'expires_s'], 'login challenge');
has(await call('login', {}, {}, { account: 'marigold', nonce: ch.nonce, public_key: '', signature: '' }), ['token', 'csrf', 'staff', 'expires'], 'login');
has(await call('me'), ['name', 'role', 'permissions', 'csrf'], 'me');
const st = await call('status');
has(st, ['online', 'uptime_s', 'tick_ms', 'memory_mb', 'content', 'saves_ok'], 'status');
has(st.tick_ms, ['mean', 'max'], 'status.tick_ms');
has(pip, ['character_id', 'name', 'account', 'role', 'zone', 'zone_name', 'position', 'job', 'level', 'session_s', 'client'], 'online row');
const list = await call('players', {}, { q: 'pip' });
has(list, ['total', 'page', 'page_size', 'rows'], 'players');
has(list.rows[0], ['character_id', 'name', 'account', 'role', 'online', 'last_seen', 'created', 'muted', 'banned'], 'players row');
ok(list.rows[0].name === 'Pip Mossberry' && list.total === 2, 'search "pip" finds Pip and Pippa, Pip first');
ok((await call('players', {}, { q: 'pmsb' })).rows.some((p) => p.name === 'Pip Mossberry'), 'fuzzy search matches letters in order');
ok((await call('players', {}, { q: String(pip.character_id) })).rows[0]?.name === 'Pip Mossberry', 'a number finds the character id');
const muted = await call('players', {}, { muted: 1 });
ok(muted.total > 0 && muted.total < data.players.length && muted.rows.every((p) => p.muted), 'muted=1 lists only muted players');
const prof = await call('profile', id);
has(prof, ['identity', 'location', 'coins', 'jobs', 'quests', 'mounts', 'pets', 'collections_n', 'sanctions', 'moderation', 'notes'], 'profile');
has(prof.sanctions, ['muted', 'muted_until', 'banned', 'banned_until'], 'profile.sanctions');
const inv = await call('inventory', id);
has(inv, ['bags', 'equipped'], 'inventory');
has(inv.bags[0], ['slot', 'item_id', 'name', 'count', 'icon'], 'inventory row');
const items = await call('items', {}, { q: 'ore' });
has(items[0], ['item_id', 'name', 'icon'], 'items row');
ok((await call('addItem', id, {}, { item_id: items[0].item_id, count: 3 })).bags.some((b) => b.item_id === items[0].item_id), 'add puts the item in the bags');
ok(!(await call('removeItems', id, {}, { items: [{ item_id: items[0].item_id, count: 'all' }] })).bags.some((b) => b.item_id === items[0].item_id), 'remove all takes it out');
ok((await call('coins', id, {}, { op: 'set', amount: 10 })).coins === 10, 'coins set');
await assert.rejects(call('coins', id, {}, { op: 'take', amount: 11 }), /only 10 coins/);
const fresh = data.quests.find((q) => ![...prof.quests.active, ...prof.quests.done].includes(q.quest_id)).quest_id;
has(await call('quests', id, {}, { op: 'grant', quest_id: fresh }), ['active', 'done'], 'quests');
const warned = await call('moderate', id, {}, { op: 'warn', reason: 'test' });
has(warned, ['op', 'row', 'moderation'], 'moderate');
ok(warned.row.after.delivered === true && warned.moderation[0].action === 'warn', 'a warning reaches an online player and heads the history');
await assert.rejects(call('moderate', id, {}, { op: 'warn' }), /reason/);
await assert.rejects(call('moderate', id, {}, { op: 'unmute' }), (e) => e.status === 409);
const forever = await call('moderate', id, {}, { op: 'mute' });
ok(forever.row.after.sanctions.muted_until >= 1e15, 'a mute without minutes lasts until lifted');
await assert.rejects(call('moderate', id, {}, { op: 'jail', reason: 'x' }), (e) => e.status === 501);
const marigold = data.players.find((p) => p.name === 'Marigold Sunvale');
await assert.rejects(call('moderate', { id: marigold.character_id }, {}, { op: 'kick' }), (e) => e.status === 403);
const notes = await call('notes', id, {}, { text: 'hello' });
ok(notes[0].after.text === 'hello', 'notes come back newest first');
has(await call('move', id, {}, { op: 'goto' }), ['zone', 'zone_name', 'position'], 'move');
has(await call('powers', {}, {}, { op: 'freeze', target: pip.character_id }), ['state'], 'powers');
const job = await call('sendMail', {}, {}, { to: { everyone: true }, subject: 'Hi', body: 'Hello' });
has(job, ['job_id', 'recipients'], 'mail send');
ok(job.recipients === data.players.length, 'everyone means every character, online or not');
has((await call('mailJobs'))[0], ['job_id', 'to_summary', 'total', 'done', 'state'], 'mail job');
has((await call('playerMail', id))[0], ['mail_id', 'from', 'subject', 'sent', 'read', 'attachments'], 'player mail');
has(await call('announce', {}, {}, { to: 'all', text: 'Hi', style: 'chat' }), ['delivered'], 'announce');
await assert.rejects(call('announce', {}, {}, { to: 'all', text: 'x'.repeat(201) }), /200/);
const aud = await call('audit', {}, { player: pip.character_id });
has(aud, ['total', 'page', 'page_size', 'rows'], 'audit');
has(aud.rows[0], ['time', 'staff', 'action', 'target', 'before', 'after', 'via'], 'audit row');
ok(aud.rows.every((r) => r.target.character_id === pip.character_id), 'audit filters by player id');
ok((await call('audit', {}, { since: Math.floor(Date.now() / 1000) - 60 })).rows.every((r) => r.time >= Math.floor(Date.now() / 1000) - 60), 'audit since is unix seconds');
const map = await call('map', { zone: pip.zone });
has(map, ['image', 'bounds', 'players'], 'map');
has(map.players[0], ['character_id', 'name', 'position'], 'map player');
await call('logout', {}, {}, {});
await assert.rejects(call('me'), (e) => e.status === 401);

// 4. The Ed25519 sign-in signs exactly what api-served.md section 2 describes, the same as Node's own signer.
const seed = randomBytes(32);
const serverId = randomBytes(32).toString('hex');
const nonce = randomBytes(32).toString('hex');
const mine = await signChallenge(new Uint8Array(seed), 'ada_admin', serverId, nonce);
const der = Buffer.concat([Buffer.from('302e020100300506032b657004220420', 'hex'), seed]);
const key = createPrivateKey({ key: der, format: 'der', type: 'pkcs8' });
const message = Buffer.concat([Buffer.from('osfr-login-v1'), Buffer.from(serverId, 'hex'), Buffer.from(nonce, 'hex'), Buffer.from('ada_admin')]);
ok(mine.signature === sign(null, message, key).toString('hex'), 'signature matches');
ok(mine.public_key === Buffer.from(createPublicKey(key).export({ format: 'jwk' }).x, 'base64url').toString('hex'), 'public key matches');
await assert.rejects(signChallenge(new Uint8Array(31), 'a', serverId, nonce), /32 bytes/);

// 5. The command bar reads names with spaces, item ids, counts and durations.
const { parse, remember, parseMinutes } = await load('js/cmd.js');
remember(online);
const give = parse('give Pip Mossberry 12345 5', true);
ok(give.cmd.name === 'give' && give.args[0].name === 'Pip Mossberry' && give.args[1].item_id === 12345 && give.args[2] === 5, 'give parses');
ok(parse('goto pip', true).args[0].name === 'Pip Mossberry', 'a unique first name resolves');
ok(parse('goto Pip Mo').stage === 0, 'a half-typed full name is still being typed');
const mute = parse('mute Pip 30m spamming trade', true);
ok(mute.args[1] === 30 && mute.args[2] === 'spamming trade', 'mute reads minutes and reason');
ok(parse('ban Pip perm cheating', true).args[1] === '', 'perm is a ban until lifted');
ok(parseMinutes('2h') === 120 && parseMinutes('1d') === 1440 && parseMinutes('x') === null, 'durations');
ok(parse('mail all Hello there', true).args[0].kind === 'everyone', 'mail all');
ok(parse('mail role:Tester Patch', true).args[0].role === 'tester', 'roles are the lowercase names the gateway uses');
ok(parse('give Pip', true).error, 'a missing item is an error');

// 6. Permission greying follows /me's list.
const act = await load('js/actions.js');
act.me.permissions = ['view', 'mute', 'unmute', 'kick', 'ban', 'unban', 'warn', 'jail', 'release', 'note', 'mail'];
ok(act.can('mute') && act.can('mail') && !act.can('inventory') && !act.can('announce'), 'a moderator list greys out item edits and announcements');
act.me.permissions = ['players.write'];
ok(act.can('inventory'), 'unknown permission names grey out nothing');

// 7. No HTML parsing of data, no external requests.
for (const f of files.filter((f) => /\.(js|html|css)$/.test(f))) {
  const src = readFileSync(f, 'utf8');
  const rel = relative(root, f);
  ok(!/innerHTML|outerHTML|insertAdjacentHTML|document\.write|\beval\(|new Function/.test(src), `${rel} parses HTML or code`);
  const urls = (src.match(/https?:\/\/[^\s'")]+/g) || []).filter((u) => u !== 'http://www.w3.org/2000/svg');
  ok(!urls.length, `${rel} reaches outside: ${urls.join(' ')}`);
  ok(!/@import|url\(\s*['"]?\/\//.test(src), `${rel} loads remote CSS`);
  ok(!/<script(?![^>]*\bsrc=)[^>]*>|\son[a-z]+=/i.test(src) || !rel.endsWith('.html'), `${rel} has inline script, which the gateway's CSP blocks`);
}
console.log(`admin-web: ${checks} checks passed`);
