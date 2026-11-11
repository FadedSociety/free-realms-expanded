// Moderation: active mutes and bans with their end times, a warn form, and the player reports queue (P1).
import { api } from '../api.js';
import { h, clear, icon, ago, when, fail, empty, nameOf, accountLabel } from '../dom.js';
import { openPlayer } from '../drawer.js';
import * as act from '../actions.js';
import { pageHead, card, table, who, suggest } from '../ui.js';

export function render(root) {
  const mutes = h('div', {});
  const bans = h('div', {});
  const muteN = h('span', { class: 'muted small' });
  const banN = h('span', { class: 'muted small' });
  root.append(
    pageHead('Moderation', 'Who is muted or banned right now, and quick warnings.'),
    h('div', { class: 'grid-2' },
      card('Active mutes', { aside: muteN }, mutes),
      card('Active bans', { aside: banN }, bans)),
    h('div', { class: 'grid-2' }, card('Warn a player', {}, warnForm()), card('Player reports', { aside: h('span', { class: 'badge' }, 'P1') },
      empty('Reports from the /report chat command will queue here, newest first, with status. The server route for them comes in P1.', 'shield'))));

  async function active(flag, op, slot, countEl) {
    try {
      const { rows, total } = await api.players({ [flag]: 1, page: 1 });
      const profiles = await Promise.all(rows.map((r) => api.profile(r.character_id).catch(() => null)));
      const list = rows.map((r, i) => {
        const pr = profiles[i];
        const last = (pr?.moderation || []).find((m) => (m.op ?? m.action) === op) || {};
        return { ...r, last, until: pr?.sanctions?.[op === 'mute' ? 'muted_until' : 'banned_until'] };
      });
      countEl.textContent = `${total} ${op === 'mute' ? 'muted' : 'banned'}`;
      clear(slot, table([
        { label: 'Player', get: (p) => who(p, accountLabel(p.account)) },
        { label: 'Ends', cls: 'num', get: (p) => (p.until != null && p.until < 1e15 ? h('span', { title: when(p.until) }, ago(p.until)) : 'When lifted') },
        { label: 'Reason', cls: 'hide-sm', get: (p) => h('span', { class: 'who-text' }, h('span', { class: 'clip' }, p.last.reason || 'No reason given'),
          h('span', { class: 'muted' }, nameOf(p.last.staff) ? `by ${nameOf(p.last.staff)}` : '')) },
        { label: '', cls: 'cell-action', get: (p) => h('button', { class: 'btn btn-sm', type: 'button', disabled: !act.can(op === 'mute' ? 'unmute' : 'unban'),
          onclick: (e) => { e.stopPropagation(); act.moderate(p, op === 'mute' ? 'unmute' : 'unban'); } }, op === 'mute' ? 'Unmute' : 'Unban') },
      ], list, { onRow: (p) => openPlayer(p.character_id), empty: op === 'mute' ? 'Nobody is muted' : 'Nobody is banned' }));
    } catch (e) { fail(e); }
  }
  const load = () => { active('muted', 'mute', mutes, muteN); active('banned', 'ban', bans, banN); };
  load();
  window.addEventListener('admin:changed', load);
  return () => window.removeEventListener('admin:changed', load);
}

function warnForm() {
  let target = null;
  const input = h('input', { class: 'input', placeholder: 'Start typing a name', 'aria-label': 'Player to warn' });
  const chosen = h('p', { class: 'muted small' }, 'Pick a player from the list.');
  const reason = h('textarea', { class: 'input', rows: 3, maxlength: 500, required: true, placeholder: 'What they read, e.g. Please keep chat friendly.' });
  const form = h('form', { class: 'stack' },
    h('div', { class: 'field' }, h('span', {}, 'Player'), suggest(input, (o) => { target = o.value; input.value = o.value.name; chosen.textContent = `${o.value.name} · ${o.value.online ? 'online now, sees it at once' : 'offline, sees it on next login'}`; })),
    chosen,
    h('label', { class: 'field' }, h('span', {}, 'Reason'), reason),
    h('div', {}, h('button', { class: 'btn btn-primary', type: 'submit', disabled: !act.can('warn') }, icon('shield', 16), 'Send warning')));
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!target || input.value !== target.name) { chosen.textContent = 'Pick a player from the list first.'; input.focus(); return; }
    const out = await act.moderate(target, 'warn', { reason: reason.value.trim() });
    if (out) { reason.value = ''; input.value = ''; target = null; chosen.textContent = 'Pick a player from the list.'; }
  });
  return form;
}
