// Entry point: picks live or mock data, signs in when the gateway asks for it, then runs the shell
// (sidebar, top bar, hash router, Ctrl+K command bar, theme).
import { api, detectMode, isMock, login, signOut } from './api.js';
import { h, clear, icon, num, initials, names, savedTheme, setTheme, toggleTheme, roleName } from './dom.js';
import { live, onLive, startLive } from './live.js';
import * as act from './actions.js';
import { openCommandBar } from './cmd.js';
import * as dashboard from './views/dashboard.js';
import * as players from './views/players.js';
import * as mail from './views/mail.js';
import * as moderation from './views/moderation.js';
import * as world from './views/world.js';
import * as audit from './views/audit.js';
import * as server from './views/server.js';

const VIEWS = {
  dashboard: ['Dashboard', 'dashboard', dashboard], players: ['Players', 'players', players], mail: ['Mail', 'mail', mail],
  moderation: ['Moderation', 'shield', moderation], world: ['World', 'globe', world], audit: ['Audit', 'audit', audit], server: ['Server', 'server', server],
};
const $ = (id) => document.getElementById(id);
let cleanup = null;
let started = false;
const top = {};

async function boot() {
  setTheme(savedTheme());
  $('login-form').addEventListener('submit', onLogin);
  await detectMode();
  try {
    Object.assign(act.me, await api.me());
  } catch (e) {
    if (e.status === 401) return showLogin();
    clear($('boot'), h('p', {}, `The admin API did not answer: ${e.message}`), h('button', { class: 'btn', onclick: () => location.reload() }, 'Try again'));
    return;
  }
  start();
}

function showLogin(message, isError = false) {
  $('boot').hidden = true;
  $('shell').hidden = true;
  $('login').hidden = false;
  const err = $('login-error');
  err.hidden = !message;
  err.className = isError ? 'form-error' : 'form-note';
  err.textContent = message || '';
  $('login-form').elements.account.focus();
}

async function onLogin(e) {
  e.preventDefault();
  const f = e.target;
  const btn = f.querySelector('button[type=submit]');
  btn.disabled = true;
  try {
    const file = f.elements.key.files[0];
    await login(f.elements.account.value.trim(), file ? new Uint8Array(await file.arrayBuffer()) : null);
    f.elements.key.value = '';
    Object.assign(act.me, await api.me());
    start();
  } catch (err) {
    showLogin(err.status === 401 ? 'That key did not sign in: check the account handle and the key file.' : err.message, true);
  } finally {
    btn.disabled = false;
  }
}

function start() {
  // Job and quest names for ids; without them the panel shows the ids.
  api.names().then((n) => { Object.assign(names, n); window.dispatchEvent(new Event('hashchange')); }).catch(() => {});
  $('boot').hidden = true;
  $('login').hidden = true;
  $('shell').hidden = false;
  if (!started) {
    started = true;
    buildNav();
    buildTop();
    window.addEventListener('hashchange', route);
    window.addEventListener('admin:signed-out', () => showLogin('Your session ended. Sign in again.'));
    document.addEventListener('keydown', keys);
    document.querySelectorAll('[data-cmdk]').forEach((b) => b.addEventListener('click', () => openCommandBar()));
    $('scrim').addEventListener('click', () => sidebar(false));
    startLive();
    onLive(paintTop);
  }
  paintStaff();
  route();
}

function buildNav() {
  clear($('nav'), Object.entries(VIEWS).map(([k, [label, ic]]) => h('li', {},
    h('a', { class: 'nav-link', href: `#/${k}`, 'data-view': k }, icon(ic, 18), h('span', {}, label), k === 'players' ? h('span', { class: 'nav-count', id: 'nav-online' }) : null))));
  $('nav').addEventListener('click', () => sidebar(false));
}

function buildTop() {
  top.online = h('a', { class: 'pill pill-live', href: '#/players', title: 'Players online now' });
  top.spect = h('div', { class: 'top-spect' });
  top.staff = h('div', { class: 'top-staff' });
  top.theme = h('button', { class: 'icon-btn', type: 'button', onclick: toggleTheme });
  const paintTheme = () => {
    const light = document.documentElement.dataset.theme === 'light';
    clear(top.theme, icon(light ? 'moon' : 'sun'));
    top.theme.setAttribute('aria-label', light ? 'Switch to the dark theme' : 'Switch to the light theme');
  };
  window.addEventListener('admin:theme', paintTheme);
  paintTheme();
  clear($('topbar'),
    h('button', { class: 'icon-btn menu-btn', type: 'button', 'aria-label': 'Open the menu', 'aria-controls': 'sidebar', onclick: () => sidebar(true) }, icon('menu')),
    h('div', { class: 'top-server' }, h('strong', {}, 'Free Realms'), h('span', { class: 'muted' }, location.host || 'local')),
    h('button', { class: 'cmd-btn', type: 'button', onclick: () => openCommandBar(), 'aria-label': 'Open the command bar' },
      icon('search', 16), h('span', { class: 'cmd-btn-text' }, 'Search players or run a command'), h('kbd', {}, 'Ctrl K')),
    h('div', { class: 'top-right' },
      isMock() ? h('span', { class: 'pill pill-mock', title: 'The gateway has no admin API here, so the panel shows invented data from mock/*.json' }, 'Mock', h('span', { class: 'hide-sm' }, ' data')) : null,
      top.spect, top.online, top.theme, top.staff));
}

function paintStaff() {
  clear(top.staff,
    h('span', { class: 'avatar', 'aria-hidden': 'true' }, initials(act.me.name)),
    h('span', { class: 'top-staff-text' }, h('strong', {}, act.me.name), h('span', { class: 'muted' }, roleName(act.me.role))),
    h('button', { class: 'icon-btn', type: 'button', 'aria-label': 'Sign out', title: 'Sign out', onclick: async () => { await signOut(); showLogin(isMock() ? 'Signed out. In mock mode any account handle signs in, with or without a key file.' : 'Signed out.'); } }, icon('logout')));
}

let spectKey = '';
function paintTop(L) {
  const n = L.status?.online ?? L.online.length;
  clear(top.online, h('span', { class: `dot ${L.error ? 'dot-bad' : 'dot-on'}`, 'aria-hidden': 'true' }), L.error ? 'Offline' : num(n), L.error ? null : h('span', { class: 'hide-sm' }, ' online'));
  const count = $('nav-online');
  if (count) count.textContent = L.error ? '' : num(n);
  const key = act.spectating ? String(act.spectating.character_id) : '';
  if (key === spectKey) return;
  spectKey = key;
  clear(top.spect, act.spectating ? h('span', { class: 'pill pill-spect' }, icon('eye', 16), h('span', { class: 'hide-sm' }, `Watching ${act.spectating.name}`),
    h('button', { class: 'pill-btn', type: 'button', onclick: () => act.power('unspectate') }, 'Stop')) : null);
}

function route() {
  const [path, qs] = location.hash.replace(/^#\/?/, '').split('?');
  const key = VIEWS[path] ? path : 'dashboard';
  cleanup?.();
  cleanup = null;
  const content = $('content');
  clear(content);
  const out = VIEWS[key][2].render(content, new URLSearchParams(qs || ''));
  cleanup = typeof out === 'function' ? out : null;
  document.querySelectorAll('.nav-link').forEach((a) => (a.dataset.view === key ? a.setAttribute('aria-current', 'page') : a.removeAttribute('aria-current')));
  document.title = `${VIEWS[key][0]} · Realm admin`;
  window.scrollTo(0, 0);
}

function sidebar(open) {
  $('sidebar').classList.toggle('open', open);
  $('scrim').hidden = !open;
  if (open) $('sidebar').querySelector('a')?.focus();
}

function keys(e) {
  const typing = e.target.closest?.('input, textarea, select, [contenteditable]');
  const modalOpen = document.querySelector('dialog.modal[open]');
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault();
    if (!document.querySelector('dialog.modal-cmd[open]')) openCommandBar();
  } else if (e.key === '/' && !typing && !modalOpen) {
    e.preventDefault();
    openCommandBar();
  } else if (e.key === 'Escape' && $('sidebar').classList.contains('open')) {
    sidebar(false);
  }
}

if (globalThis.document) boot();
