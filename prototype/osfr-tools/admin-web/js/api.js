// Client for the admin HTTP API: one entry per route the gateway serves (the gm-panel contract, plus the login
// challenge and logout G1 added, and job edits, undo by key, id names and the powers' state from integration). In mock mode (?mock=1, or when /admin/api/me does not exist) mock.js answers
// the same calls from mock/*.json, in memory.
import { createMock } from './mock.js';

export const ROUTES = {
  loginChallenge: ['POST', '/admin/api/login/challenge'],
  login: ['POST', '/admin/api/login'],
  logout: ['POST', '/admin/api/logout'],
  me: ['GET', '/admin/api/me'],
  status: ['GET', '/admin/api/status'],
  online: ['GET', '/admin/api/players/online'],
  players: ['GET', '/admin/api/players'],
  profile: ['GET', '/admin/api/players/{id}'],
  inventory: ['GET', '/admin/api/players/{id}/inventory'],
  addItem: ['POST', '/admin/api/players/{id}/inventory/add'],
  removeItems: ['POST', '/admin/api/players/{id}/inventory/remove'],
  coins: ['POST', '/admin/api/players/{id}/coins'],
  quests: ['POST', '/admin/api/players/{id}/quests'],
  jobs: ['POST', '/admin/api/players/{id}/jobs'],
  undo: ['POST', '/admin/api/undo'],
  names: ['GET', '/admin/api/names'],
  powerState: ['GET', '/admin/api/powers'],
  moderate: ['POST', '/admin/api/players/{id}/moderate'],
  notes: ['POST', '/admin/api/players/{id}/notes'],
  move: ['POST', '/admin/api/players/{id}/move'],
  powers: ['POST', '/admin/api/powers'],
  items: ['GET', '/admin/api/items'],
  icon: ['GET', '/admin/icon/{item_id}'],
  mailJobs: ['GET', '/admin/api/mail/jobs'],
  sendMail: ['POST', '/admin/api/mail'],
  playerMail: ['GET', '/admin/api/players/{id}/mail'],
  announce: ['POST', '/admin/api/announce'],
  audit: ['GET', '/admin/api/audit'],
  map: ['GET', '/admin/api/map/{zone}'],
  events: ['GET', '/admin/api/events'], // optional stream; the panel polls every 2 s instead
};

export const MOCK_FILES = ['meta', 'me', 'status', 'zones', 'players', 'inventory', 'items', 'quests', 'mail', 'audit'];
const NO_CSRF = new Set(['loginChallenge', 'login']);
let mock = null;
let csrf = null; // the session itself is the gateway's HttpOnly cookie; only the CSRF token lives here

export const isMock = () => mock !== null;

export function path(name, params = {}) {
  return ROUTES[name][1].replace(/\{(\w+)\}/g, (_, k) => encodeURIComponent(params[k] ?? ''));
}

export async function useMock() {
  const data = {};
  await Promise.all(MOCK_FILES.map(async (f) => {
    data[f] = await (await fetch(new URL(`../mock/${f}.json`, import.meta.url))).json();
  }));
  mock = createMock(data);
}

// Live unless ?mock=1, or the gateway has no admin API at this origin (404, a non-JSON page, or no server).
export async function detectMode() {
  if (new URLSearchParams(location.search).get('mock') === '1') return useMock();
  try {
    const r = await fetch(ROUTES.me[1], { headers: { accept: 'application/json' } });
    if (r.status === 401 || (r.ok && (r.headers.get('content-type') || '').includes('json'))) return;
  } catch { /* no server: fall through */ }
  return useMock();
}

export class ApiError extends Error {
  constructor(status, message) { super(message); this.status = status; }
}

async function req(name, params = {}, query = null, body = undefined) {
  if (mock) return mock.handle(name, params, query || {}, body);
  const [method] = ROUTES[name];
  const headers = { accept: 'application/json' };
  if (body !== undefined) headers['content-type'] = 'application/json';
  if (method === 'POST' && csrf && !NO_CSRF.has(name)) headers['x-csrf-token'] = csrf;
  const qs = query ? new URLSearchParams(Object.entries(query).filter(([, v]) => v !== '' && v != null)).toString() : '';
  let r;
  try {
    r = await fetch(path(name, params) + (qs ? `?${qs}` : ''), { method, headers, body: body === undefined ? undefined : JSON.stringify(body) });
  } catch {
    throw new ApiError(0, 'The server did not answer. Is the gateway running?');
  }
  const data = await r.json().catch(() => null);
  if (r.status === 401 && !NO_CSRF.has(name)) window.dispatchEvent(new Event('admin:signed-out'));
  if (!r.ok) throw new ApiError(r.status, data?.error || `${r.status} ${r.statusText}`);
  return data;
}

// Ed25519 sign-in (api-served.md section 2): sign "osfr-login-v1" + server_id + nonce + account with the staff
// member's launcher key, a raw 32-byte seed. The seed stays in this page.
const fromHex = (h) => Uint8Array.from(h.match(/../g) || [], (b) => parseInt(b, 16));
const toHex = (buf) => [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, '0')).join('');
const PKCS8_ED25519 = '302e020100300506032b657004220420';
export async function signChallenge(seed, account, serverId, nonce) {
  if (!globalThis.crypto?.subtle) throw new ApiError(0, 'Signing in needs a secure page: open the panel at 127.0.0.1 or over https.');
  if (seed.length !== 32) throw new ApiError(0, 'That is not a key file: key.ed25519 from your launcher profile is exactly 32 bytes.');
  const key = await crypto.subtle.importKey('pkcs8', new Uint8Array([...fromHex(PKCS8_ED25519), ...seed]), { name: 'Ed25519' }, true, ['sign']);
  const { x } = await crypto.subtle.exportKey('jwk', key);
  const pub = Uint8Array.from(atob(x.replace(/-/g, '+').replace(/_/g, '/')), (c) => c.charCodeAt(0));
  const text = new TextEncoder();
  const message = new Uint8Array([...text.encode('osfr-login-v1'), ...fromHex(serverId), ...fromHex(nonce), ...text.encode(account)]);
  return { public_key: toHex(pub), signature: toHex(await crypto.subtle.sign('Ed25519', key, message)) };
}

export async function login(account, seed) {
  const ch = await req('loginChallenge', {}, null, { account });
  const signed = seed ? await signChallenge(seed, account, ch.server_id, ch.nonce) : mock ? { public_key: '', signature: '' } : null;
  if (!signed) throw new ApiError(0, 'Choose your key file.');
  const out = await req('login', {}, null, { account, nonce: ch.nonce, ...signed });
  csrf = out.csrf ?? csrf;
  return out;
}

export async function signOut() {
  try { await req('logout', {}, null, {}); } catch { /* signed out either way */ }
  csrf = null;
}

const id = (character_id) => ({ id: character_id });
export const api = {
  me: async () => { const me = await req('me'); csrf = me.csrf ?? csrf; return me; },
  status: () => req('status'),
  online: () => req('online'),
  players: (query) => req('players', {}, query),
  profile: (cid) => req('profile', id(cid)),
  inventory: (cid) => req('inventory', id(cid)),
  addItem: (cid, item_id, count, equip) => req('addItem', id(cid), null, equip ? { item_id, count, equip: true } : { item_id, count }),
  removeItems: (cid, items) => req('removeItems', id(cid), null, { items }),
  coins: (cid, op, amount) => req('coins', id(cid), null, { op, amount }),
  quests: (cid, op, quest_id) => req('quests', id(cid), null, { op, quest_id }),
  jobs: (cid, job_id, set) => req('jobs', id(cid), null, { job_id, ...set }),
  undo: (key) => req('undo', {}, null, { undo: key }),
  names: () => req('names'),
  powerState: () => req('powerState'),
  moderate: (cid, op, minutes, reason) => req('moderate', id(cid), null, { op, ...(minutes ? { minutes } : {}), ...(reason ? { reason } : {}) }),
  notes: (cid, text) => req('notes', id(cid), null, { text }),
  move: (cid, op, to) => req('move', id(cid), null, to ? { op, to } : { op }),
  powers: (op, target) => req('powers', {}, null, target == null ? { op } : { op, target }),
  items: (q) => req('items', {}, { q }),
  mailJobs: () => req('mailJobs'),
  sendMail: (to, subject, body, attachments) => req('sendMail', {}, null, attachments ? { to, subject, body, attachments } : { to, subject, body }),
  playerMail: (cid) => req('playerMail', id(cid)),
  announce: (to, text, style) => req('announce', {}, null, { to, text, style }),
  audit: (query) => req('audit', {}, query),
  map: (zone) => req('map', { zone }),
};

// An item's icon: the API's icon URL when it gives one, else the gateway's icon route; none in mock mode.
export function iconUrl(item) {
  if (mock || !item) return null;
  if (typeof item.icon === 'string' && /^(\/|https?:)/.test(item.icon)) return item.icon;
  return path('icon', { item_id: item.item_id });
}
