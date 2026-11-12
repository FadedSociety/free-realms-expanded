// Gifts & Codes: the gift catalog (the original's free items: the profile page's offers, its published codes and
// promotions), a gift sent straight into a player's Claim window, a code for an event's winner, the codes made so
// far, and the standing offers turned on or off. Staff send; admins make codes and turn offers on or off.
import { api } from '../api.js';
import { h, clear, icon, ask, toast, fail, num, plural, empty, badge, debounce } from '../dom.js';
import * as act from '../actions.js';
import { pageHead, card, table, pager, suggest } from '../ui.js';

const SOURCE = { profile: ['Offer', 'accent'], code: ['Code', 'info'], promotional: ['Promotion', 'warn'], member: ['Member pack', 'accent'] };
const words = (key) => String(key).replace(/_/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase());
const gives = (g) => [...g.items.map((i) => (i.count > 1 ? `${i.count} × ` : '') + `#${i.item_id}`), ...(g.coins ? [`${num(g.coins)} coins`] : [])].join(', ');

function rules(g) {
  const out = [];
  if (g.live_event) out.push(badge(words(g.live_event), 'warn'));
  if (g.repeatable) out.push(badge(`Every ${g.repeat_hours} h`, 'info'));
  else out.push(badge(g.per === 'character' ? 'Once per character' : 'Once per account'));
  if (g.tradeable) out.push(badge('Tradeable'));
  if (g.stock_per_event) out.push(badge(`${num(g.stock_per_event)} per event`, 'warn'));
  if (g.per_player) out.push(badge(`${num(g.per_player)} per player`));
  return h('span', { class: 'gift-rules' }, out);
}

// A player chosen through the shared search; `onPick(player | null)`.
function playerField(label, onPick) {
  const input = h('input', { class: 'input', placeholder: 'Player name', 'aria-label': label });
  const chosen = h('span', { class: 'muted small' }, 'Nobody chosen');
  let player = null;
  const set = (p) => {
    player = p;
    clear(chosen, p ? h('span', {}, h('strong', {}, p.name), ` · #${p.character_id} `,
      h('button', { class: 'token-x', type: 'button', 'aria-label': `Clear ${p.name}`, onclick: () => set(null) }, icon('close', 14))) : 'Nobody chosen');
    input.value = '';
    onPick(p);
  };
  const box = suggest(input, (o) => set({ character_id: o.value.character_id, name: o.value.name }));
  return { el: h('div', { class: 'field' }, h('span', {}, label), box, chosen), get: () => player, set };
}

export function render(root) {
  let page = 1;
  let query = '';
  let target = null;
  const bundle = [];
  const search = h('input', { class: 'input', type: 'search', placeholder: 'Search by name, key or code', 'aria-label': 'Search gifts' });
  const rows = h('div');
  const foot = h('div');
  const unresolved = h('p', { class: 'muted small' });
  const codes = h('div');
  const made = h('div', { class: 'gift-made', 'aria-live': 'polite' });

  // Send to player
  const sendGift = h('p', { class: 'muted small' }, 'Pick a gift with Send in the catalog.');
  let sending = null;
  const sendTo = playerField('Player', (p) => { target = p; });
  const sendNote = h('input', { class: 'input', maxlength: 200, placeholder: 'Optional, for example “Costume contest winner”' });
  const sendBtn = h('button', { class: 'btn btn-primary', type: 'submit' }, icon('send', 16), 'Send');
  const sendForm = h('form', { class: 'stack' }, h('div', { class: 'field' }, h('span', {}, 'Gift'), sendGift), sendTo.el,
    h('label', { class: 'field' }, h('span', {}, 'Note'), sendNote), h('div', { class: 'form-foot' }, h('span', { class: 'muted small' },
      'It waits in their Claim window, online or not.'), sendBtn));
  sendForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!sending) { toast('Pick a gift first', { kind: 'error' }); return; }
    if (!target) { toast('Pick who it goes to', { kind: 'error' }); return; }
    sendBtn.disabled = true;
    try {
      await api.giftSend(target.character_id, sending.key, sendNote.value.trim());
      toast(`${sending.name} is waiting in ${target.name}’s Claim window`);
      sendNote.value = '';
    } catch (err) { fail(err); } finally { sendBtn.disabled = false; }
  });
  const chooseSend = (g) => { sending = g; clear(sendGift, h('strong', {}, g.name), ' ', h('span', { class: 'muted' }, gives(g))); sendTo.el.querySelector('input').focus(); };

  // Make code
  const picked = h('div', { class: 'attach-list' });
  const uses = h('input', { class: 'input', type: 'number', min: 1, max: 10000, value: 1, inputmode: 'numeric' });
  const days = h('input', { class: 'input', type: 'number', min: 1, max: 3650, placeholder: 'Never', inputmode: 'numeric' });
  let bound = null;
  const bindTo = playerField('Only this player may redeem it (optional)', (p) => { bound = p; });
  const codeNote = h('input', { class: 'input', maxlength: 200, placeholder: 'Shown as the gift’s name; the gifts’ names by default' });
  const makeBtn = h('button', { class: 'btn btn-primary', type: 'submit' }, icon('plus', 16), 'Make code');
  const paintBundle = () => clear(picked, bundle.length ? bundle.map((g, i) => h('span', { class: 'attach' }, g.name,
    h('button', { type: 'button', class: 'token-x', 'aria-label': `Remove ${g.name}`, onclick: () => { bundle.splice(i, 1); paintBundle(); } }, icon('close', 14))))
    : h('span', { class: 'muted small' }, 'Add gifts with Add to code in the catalog. Several make a bundle.'));
  const codeForm = h('form', { class: 'stack' }, h('div', { class: 'field' }, h('span', {}, 'Gives'), picked),
    h('div', { class: 'grid-2' }, h('label', { class: 'field' }, h('span', {}, 'Uses in all'), uses),
      h('label', { class: 'field' }, h('span', {}, 'Works for (days)'), days)),
    bindTo.el, h('label', { class: 'field' }, h('span', {}, 'Name'), codeNote), h('div', { class: 'form-foot' }, made, makeBtn));
  codeForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!bundle.length) { toast('Add a gift to the code first', { kind: 'error' }); return; }
    makeBtn.disabled = true;
    try {
      const r = await api.giftMakeCode({ keys: bundle.map((g) => g.key), uses: Number(uses.value) || 1,
        ...(days.value ? { expires_days: Number(days.value) } : {}), ...(bound ? { character_id: bound.character_id } : {}), note: codeNote.value.trim() });
      clear(made, h('span', { class: 'gift-code' }, h('code', {}, r.code)),
        h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: () => navigator.clipboard?.writeText(r.code).then(() => toast('Copied'), () => {}) }, 'Copy'),
        h('span', { class: 'muted small' }, `The player types: realm redeem ${r.code}`));
      bundle.length = 0; codeNote.value = ''; bindTo.set(null); paintBundle(); loadCodes();
    } catch (err) { fail(err); } finally { makeBtn.disabled = false; }
  });

  async function toggle(g) {
    const on = !g.enabled_now;
    const ok = await ask({ title: `${on ? 'Turn on' : 'Turn off'} ${g.name}?`, confirm: on ? 'Turn on' : 'Turn off', danger: !on,
      body: on ? 'Every account sees it in its Claim window again (during its event, if it has one).' : 'It leaves every Claim window that has not claimed it yet; its code stops working.' });
    if (!ok) return;
    try { await api.giftOffer(g.key, on); toast(`${g.name} is ${on ? 'on' : 'off'}`); load(); } catch (err) { fail(err); }
  }

  const cols = [
    { label: 'Gift', get: (g) => h('span', { class: 'who-text' }, h('strong', {}, g.name), h('span', { class: 'muted small' }, gives(g))) },
    { label: 'Source', get: (g) => { const [l, k] = SOURCE[g.source] || [g.source, '']; return badge(l, k); } },
    { label: 'Code', cls: 'hide-sm', get: (g) => (g.code ? h('code', {}, g.code) : h('span', { class: 'muted' }, '—')) },
    { label: 'Rules', cls: 'hide-sm', get: rules },
    { label: 'On', get: (g) => (act.can('gift_codes')
      ? h('button', { class: `btn btn-sm ${g.enabled_now ? '' : 'btn-ghost'}`, type: 'button', 'aria-pressed': String(g.enabled_now), onclick: (e) => { e.stopPropagation(); toggle(g); } }, g.enabled_now ? 'On' : 'Off')
      : badge(g.enabled_now ? 'On' : 'Off', g.enabled_now ? 'accent' : '')) },
    { label: '', get: (g) => h('span', { class: 'row-actions' },
      h('button', { class: 'btn btn-sm', type: 'button', disabled: !act.can('gifts'), onclick: () => chooseSend(g) }, 'Send'),
      act.can('gift_codes') ? h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: () => { if (!bundle.some((b) => b.key === g.key)) bundle.push(g); paintBundle(); } }, 'Add to code') : null) },
  ];

  async function load() {
    try {
      const d = await api.giftCatalog({ q: query, page });
      clear(rows, table(cols, d.rows, { empty: query ? 'No gift matches' : 'The catalog is empty', caption: 'Gift catalog' }));
      clear(foot, pager(d.page, d.rows.length, d.page_size, d.total, (p) => { page = p; load(); }));
      unresolved.textContent = d.unresolved?.length
        ? `${plural(d.unresolved.length, 'listed item')} cannot be given yet: no item row yet, or no source names them (see gift_catalog.json).` : '';
    } catch (err) { clear(rows, empty(err.message, 'star')); clear(foot); }
  }
  async function loadCodes() {
    try {
      const list = await api.giftCodes();
      clear(codes, table([
        { label: 'Code', get: (c) => h('code', {}, c.code) },
        { label: 'Gives', get: (c) => c.note || plural(c.items.length, 'item') },
        { label: 'Used', get: (c) => `${num(c.uses)} of ${c.max_uses ? num(c.max_uses) : 'any'}` },
        { label: 'Expires', cls: 'hide-sm', get: (c) => c.expires || 'Never' },
        { label: 'For', cls: 'hide-sm', get: (c) => (c.bound_account ? `Account #${c.bound_account}` : 'Anyone') },
        { label: '', get: (c) => (!c.enabled ? badge('Revoked', 'danger') : act.can('gift_codes')
          ? h('button', { class: 'btn btn-sm btn-ghost', type: 'button', onclick: async () => {
            const ok = await ask({ title: `Revoke ${c.code}?`, body: 'Nobody can redeem it after this. Gifts already redeemed stay.', confirm: 'Revoke', danger: true });
            if (!ok) return;
            try { await api.giftRevoke(c.code); toast(`${c.code} revoked`); loadCodes(); } catch (err) { fail(err); }
          } }, 'Revoke') : badge('Active', 'accent')) },
      ], list, { empty: 'No codes made yet', caption: 'Codes' }));
    } catch (err) { clear(codes, empty(err.message, 'star')); }
  }

  search.addEventListener('input', debounce(() => { query = search.value.trim(); page = 1; load(); }, 200));
  root.append(
    pageHead('Gifts & Codes', 'The original’s free items. Send one to an event’s winner, or make a code they can redeem.'),
    card('Catalog', { aside: search }, rows, foot, unresolved),
    h('div', { class: 'grid-2' },
      card('Send to player', {}, act.can('gifts') ? sendForm : h('p', { class: 'muted' }, 'Your role cannot send gifts.')),
      card('Make a code', {}, act.can('gift_codes') ? codeForm : h('p', { class: 'muted' }, 'Admins make codes.'))),
    card('Codes', {}, codes));
  paintBundle();
  load();
  loadCodes();
}
