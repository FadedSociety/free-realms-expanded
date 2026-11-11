// Mail composer: a "To" token field (players, Everyone, Online, a role, a zone), subject and body with a live
// preview, item and coin attachments, a confirm with the recipient count, and every send's progress.
import { api } from '../api.js';
import { h, clear, icon, ask, toast, fail, num, ago, plural, empty, badge, roleName } from '../dom.js';
import { live } from '../live.js';
import * as act from '../actions.js';
import { itemArt, pickItem } from '../picker.js';
import { pageHead, card, suggest } from '../ui.js';

const STATE = { done: ['Sent', 'accent'], running: ['Sending', 'info'], queued: ['Queued', 'warn'], failed: ['Failed', 'danger'] };
export function jobRow(j) {
  const pct = j.total ? Math.round((j.done / j.total) * 100) : 0;
  const [label, kind] = STATE[j.state] || [j.state, ''];
  return h('div', { class: `job ${j.state === 'done' ? 'job-done' : ''}` },
    h('div', { class: 'job-top' }, h('strong', {}, j.subject || j.to_summary), badge(label, kind)),
    h('div', { class: 'progress', role: 'progressbar', 'aria-label': `Delivery of ${j.subject || j.to_summary}`, 'aria-valuemin': '0', 'aria-valuemax': String(j.total), 'aria-valuenow': String(j.done) },
      h('span', { style: { width: `${pct}%` } })),
    h('span', { class: 'muted small' }, `${j.to_summary} · ${num(j.done)} of ${plural(j.total, 'recipient')}${j.created ? ` · ${ago(j.created)}` : ''}`));
}

const GROUP = new Set(['everyone', 'online', 'role', 'zone']);
function payload(chips) {
  const g = chips.find((c) => GROUP.has(c.kind));
  if (!g) return { characters: chips.map((c) => c.id) };
  if (g.kind === 'everyone') return { everyone: true };
  if (g.kind === 'online') return { online: true };
  if (g.kind === 'role') return { role: g.role };
  return { zone: /^\d+$/.test(String(g.zone)) ? Number(g.zone) : g.zone };
}
async function countFor(chips) {
  const g = chips.find((c) => GROUP.has(c.kind));
  if (!g) return [chips.length, ''];
  if (g.kind === 'online') return [live.online.length, ''];
  if (g.kind === 'zone') return [live.online.filter((p) => String(p.zone) === String(g.zone)).length, ' online in that zone now'];
  const d = await api.players(g.kind === 'role' ? { role: g.role } : {});
  return [d.total, ''];
}

export function render(root) {
  const chips = [];
  const attach = [];
  const pre = act.pending.mail;
  act.pending.mail = null;
  if (pre?.to) chips.push(...pre.to.filter(Boolean));

  const tokenBox = h('div', { class: 'tokens' });
  const toInput = h('input', { class: 'token-input', placeholder: 'Player name, Everyone, Online, Role or Zone', 'aria-label': 'Recipients' });
  const subject = h('input', { class: 'input', maxlength: 120, required: true, value: pre?.subject || '', placeholder: 'What is it about?' });
  const body = h('textarea', { class: 'input', rows: 8, maxlength: 4000, placeholder: 'Write the message the player will read.' });
  const coins = h('input', { class: 'input', type: 'number', min: 0, max: 1000000, inputmode: 'numeric', placeholder: '0' });
  const attachList = h('div', { class: 'attach-list' });
  const reach = h('p', { class: 'muted small', 'aria-live': 'polite' });
  const send = h('button', { class: 'btn btn-primary', type: 'submit' }, icon('send', 16), 'Send');
  const preview = h('div', { class: 'letter' });
  const jobs = h('div', { class: 'jobs' });
  const counter = h('span', { class: 'muted small' });

  const groups = (q) => {
    const zones = act.ZONES();
    return [{ label: 'Everyone', sub: 'online and offline', icon: 'globe', value: { kind: 'everyone', label: 'Everyone' } },
      { label: 'Online', sub: `${live.online.length} now`, icon: 'players', value: { kind: 'online', label: 'Online' } },
      ...act.ROLES.map((r) => ({ label: `Role: ${roleName(r)}`, icon: 'shield', value: { kind: 'role', role: r, label: `Role: ${roleName(r)}` } })),
      ...zones.map(([z, n]) => ({ label: `Zone: ${n}`, icon: 'pin', value: { kind: 'zone', zone: z, label: `Zone: ${n}` } }))]
      .filter((o) => !q || o.label.toLowerCase().includes(q.toLowerCase()));
  };
  function addChip(c) {
    if (GROUP.has(c.kind) || chips.some((x) => GROUP.has(x.kind))) chips.length = 0; // one group, or any number of players
    if (c.kind === 'player' && chips.some((x) => String(x.id) === String(c.id))) return;
    chips.push(c);
    toInput.value = '';
    paint();
    toInput.focus();
  }
  function paintTokens() {
    clear(tokenBox, chips.map((c, i) => h('span', { class: `token ${GROUP.has(c.kind) ? 'token-group' : ''}` }, c.label,
      h('button', { type: 'button', class: 'token-x', 'aria-label': `Remove ${c.label}`, onclick: () => { chips.splice(i, 1); paint(); toInput.focus(); } }, icon('close', 14)))),
    toWrap);
  }
  const toWrap = suggest(toInput, (o) => addChip(o.value.kind === 'player' ? { kind: 'player', id: o.value.character_id, label: o.value.name } : o.value), { extra: groups });
  toInput.addEventListener('keydown', (e) => { if (e.key === 'Backspace' && !toInput.value && chips.length) { chips.pop(); paint(); toInput.focus(); } });
  tokenBox.addEventListener('click', (e) => { if (e.target === tokenBox) toInput.focus(); });

  let reachToken = 0;
  async function paintReach() {
    const mine = ++reachToken;
    if (!chips.length) { reach.textContent = 'Add who it goes to. A group (Everyone, Online, a role or a zone) replaces any players.'; return; }
    try {
      const [n, note] = await countFor(chips);
      if (mine === reachToken) reach.textContent = `Goes to ${plural(n, 'player')}${note}.`;
    } catch { reach.textContent = ''; }
  }
  function paintAttach() {
    clear(attachList, attach.map((a, i) => h('span', { class: 'attach' }, itemArt(a.item, 'xs'), `${a.count} × ${a.item.name}`,
      h('button', { type: 'button', class: 'token-x', 'aria-label': `Remove ${a.item.name}`, onclick: () => { attach.splice(i, 1); paint(); } }, icon('close', 14)))),
    h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: async () => {
      const r = await pickItem({ title: 'Attach an item', confirm: 'Attach' });
      if (r) { attach.push(r); paint(); }
    } }, icon('plus', 16), 'Attach item'));
  }
  function paintPreview() {
    const c = Number(coins.value) || 0;
    clear(preview,
      h('div', { class: 'letter-from' }, h('span', { class: 'avatar', 'aria-hidden': 'true' }, icon('mail', 16)), h('span', {}, h('strong', {}, act.me.name || 'Staff'), h('span', { class: 'muted' }, 'just now'))),
      h('h3', { class: 'letter-subject' }, subject.value.trim() || 'No subject yet'),
      h('p', { class: 'letter-body' }, body.value || 'The message appears here as you type.'),
      attach.length || c ? h('div', { class: 'letter-attach' }, attach.map((a) => h('span', { class: 'letter-item', title: a.item.name }, itemArt(a.item, 'sm'), h('span', {}, `×${a.count}`))),
        c ? h('span', { class: 'letter-item' }, h('span', { class: 'art art-sm art-ph' }, icon('coins', 18)), h('span', {}, num(c))) : null,
      h('span', { class: 'btn btn-sm btn-primary letter-claim', 'aria-hidden': 'true' }, 'Claim')) : null);
    counter.textContent = `${body.value.length} / 4000`;
  }
  function paint() { paintTokens(); paintAttach(); paintPreview(); paintReach(); }

  const form = h('form', { class: 'stack' },
    h('div', { class: 'field' }, h('span', {}, 'To'), tokenBox),
    h('label', { class: 'field' }, h('span', {}, 'Subject'), subject),
    h('label', { class: 'field' }, h('span', { class: 'field-row' }, 'Message', counter), body),
    h('div', { class: 'field' }, h('span', {}, 'Attachments'), attachList),
    h('label', { class: 'field field-narrow' }, h('span', {}, 'Coins'), coins),
    h('div', { class: 'form-foot' }, reach, send));
  form.addEventListener('input', paintPreview);
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!chips.length) { toast('Add who it goes to', { kind: 'error' }); toInput.focus(); return; }
    if (!subject.value.trim()) { subject.focus(); return; }
    let n;
    let note;
    try { [n, note] = await countFor(chips); } catch (err) { fail(err); return; }
    const big = chips.some((c) => GROUP.has(c.kind));
    const ok = await ask({ title: `Send to ${plural(n, 'player')}?`, confirm: `Send to ${num(n)}`, danger: big && n > 20,
      body: `“${subject.value.trim()}” to ${chips.map((c) => c.label).join(', ')}${note}.${attach.length || Number(coins.value) ? ' Each one can claim the attachments once.' : ''}` });
    if (!ok) return;
    send.disabled = true;
    try {
      const att = attach.length || Number(coins.value) ? { items: attach.map((a) => ({ item_id: a.item.item_id, count: a.count })), coins: Number(coins.value) || 0 } : undefined;
      const r = await api.sendMail(payload(chips), subject.value.trim(), body.value, att);
      toast(`Mail queued for ${plural(r.recipients, 'player')}`);
      chips.length = 0; attach.length = 0; subject.value = ''; body.value = ''; coins.value = '';
      paint();
      loadJobs();
    } catch (err) { fail(err); } finally { send.disabled = false; }
  });

  async function loadJobs() {
    try {
      const list = await api.mailJobs();
      clear(jobs, list.length ? list.map(jobRow) : empty('No mail sent yet', 'mail'));
    } catch (e) { clear(jobs, empty(e.message, 'mail')); }
  }

  root.append(
    pageHead('Mail', 'Write to one player, a few, a group, or everyone. Offline players get it on their next login.'),
    h('div', { class: 'mail-grid' },
      card('Compose', { cls: 'compose' }, act.can('mail') ? form : h('p', { class: 'muted' }, 'Your role cannot send mail.')),
      card('What the player sees', { cls: 'preview' }, preview)),
    card('Past sends', { aside: h('span', { class: 'muted small' }, 'Updates every 2 seconds') }, jobs));
  paint();
  loadJobs();
  const t = setInterval(loadJobs, 2000);
  if (pre) queueMicrotask(() => (chips.length ? subject : toInput).focus());
  return () => clearInterval(t);
}
