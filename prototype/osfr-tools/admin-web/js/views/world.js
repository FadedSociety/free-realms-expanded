// World: announcements (everyone, a zone or a role; banner or chat line), your own staff powers, and the
// restart countdown (P1: the contract has no route for it yet).
import { h, clear, icon, num, roleName } from '../dom.js';
import { live, onLive } from '../live.js';
import * as act from '../actions.js';
import { pageHead, card, seg } from '../ui.js';

const form = { aud: 'all', zone: '', role: 'player', style: 'chat', text: '' };

export function render(root) {
  const text = h('textarea', { class: 'input', rows: 3, maxlength: 200, value: form.text, placeholder: 'The realm restarts in 10 minutes. Finish your race!', 'aria-label': 'Announcement' });
  const zoneSel = h('select', { class: 'input', 'aria-label': 'Zone' });
  const roleSel = h('select', { class: 'input', 'aria-label': 'Role', value: form.role, onchange: () => { form.role = roleSel.value; paint(); } }, act.ROLES.map((r) => h('option', { value: r }, roleName(r))));
  const pick = h('div', { class: 'aud-pick' });
  const preview = h('div', { class: 'announce-preview' });
  const reach = h('p', { class: 'muted small', 'aria-live': 'polite' });
  const counter = h('span', { class: 'muted small' });
  const powers = h('div', { class: 'stack' });
  zoneSel.addEventListener('change', () => { form.zone = zoneSel.value; paint(); });
  text.addEventListener('input', () => { form.text = text.value; paint(); });

  const to = () => (form.aud === 'all' ? 'all' : form.aud === 'zone' ? { zone: /^\d+$/.test(form.zone) ? Number(form.zone) : form.zone } : { role: form.role });
  function paint() {
    if (pick.firstChild !== (form.aud === 'zone' ? zoneSel : form.aud === 'role' ? roleSel : null)) clear(pick, form.aud === 'zone' ? zoneSel : form.aud === 'role' ? roleSel : null);
    const t = form.text.trim() || 'Your announcement appears here.';
    clear(preview, form.style === 'banner'
      ? h('div', { class: 'banner-preview' }, icon('megaphone', 18), h('span', {}, t))
      : h('p', { class: 'chat-preview' }, h('span', { class: 'chat-chan' }, '[Realm]'), ' ', t));
    counter.textContent = `${form.text.length} / 200`;
    paintReach();
  }
  function paintReach() {
    reach.textContent = form.aud === 'zone' && !form.zone ? 'Nobody is online in any zone.' : `${num(act.audienceSize(to()))} online now will see it.`;
  }
  let zoneKey = '';
  function paintZones() {
    const zones = act.ZONES();
    const key = zones.map(([z, n]) => `${z}:${n}:${live.online.filter((p) => String(p.zone) === z).length}`).join('|');
    if (key === zoneKey) return;
    zoneKey = key;
    if (!form.zone && zones[0]) form.zone = zones[0][0];
    clear(zoneSel, zones.map(([z, n]) => h('option', { value: z }, `${n} (${live.online.filter((p) => String(p.zone) === z).length} online)`)));
    zoneSel.value = form.zone;
  }
  function paintPowers() {
    const sp = act.spectating;
    const ok = act.can('powers');
    clear(powers,
      toggleRow('Invisible', 'Hidden from every other player\'s list and world.', act.self.invisible, ok, () => act.power(act.self.invisible ? 'visible' : 'invisible').then(paintPowers)),
      toggleRow('God mode', 'You take no damage.', act.self.god, ok, () => act.power('god').then(paintPowers)),
      h('div', { class: 'power-row' },
        h('div', {}, h('strong', {}, 'Spectating'), h('p', { class: 'muted small' }, sp ? `Following ${sp.name}. You are hidden while you watch.` : 'Open a player and press Spectate to watch them.')),
        sp ? h('button', { class: 'btn btn-sm', type: 'button', onclick: () => act.power('unspectate').then(paintPowers) }, 'Stop') : null));
  }

  const announceForm = h('form', { class: 'stack' },
    h('div', { class: 'field' }, h('span', {}, 'Who hears it'), h('div', { class: 'row-form' },
      seg([['all', 'Everyone'], ['zone', 'A zone'], ['role', 'A role']], form.aud, (v) => { form.aud = v; paint(); }, 'Audience'), pick)),
    h('div', { class: 'field' }, h('span', {}, 'How'), seg([['chat', 'Chat line'], ['banner', 'Banner']], form.style, (v) => { form.style = v; paint(); }, 'Style'),
      h('p', { class: 'muted small' }, 'The original client has no free-text banner yet, so both arrive as a chat line there.')),
    h('label', { class: 'field' }, h('span', { class: 'field-row' }, 'Message', counter), text),
    h('div', { class: 'field' }, h('span', {}, 'Preview'), preview),
    h('div', { class: 'form-foot' }, reach, h('button', { class: 'btn btn-primary', type: 'submit', disabled: !act.can('announce') }, icon('megaphone', 16), 'Announce')));
  announceForm.addEventListener('submit', async (e) => {
    e.preventDefault();
    if (!form.text.trim()) { text.focus(); return; }
    if (await act.announce(to(), form.text.trim(), form.style)) { form.text = ''; text.value = ''; paint(); }
  });

  root.append(
    pageHead('World', 'Talk to the whole realm and manage your own staff powers.'),
    h('div', { class: 'grid-2 grid-wide-left' },
      card('Announcement', {}, announceForm),
      h('div', { class: 'stack-lg' },
        card('Your powers', {}, powers),
        card('Restart countdown', { aside: h('span', { class: 'badge' }, 'P1') },
          h('div', { class: 'stack' },
            h('p', { class: 'muted' }, 'Warns everyone at 10, 5 and 1 minutes, then restarts. Waits for its server route (P1).'),
            h('div', { class: 'row-form' }, h('select', { class: 'input', disabled: true, 'aria-label': 'Countdown length' }, ['10 minutes', '5 minutes', '1 minute'].map((m) => h('option', {}, m))),
              h('button', { class: 'btn', type: 'button', disabled: true }, 'Start countdown')))))));
  paintZones();
  paint();
  paintPowers();
  const powKey = () => `${act.self.invisible}|${act.self.god}|${act.spectating?.character_id}`;
  let shown = powKey();
  return onLive(() => {
    paintZones();
    paintReach();
    if (powKey() !== shown) { shown = powKey(); paintPowers(); }
  });
}

function toggleRow(title, sub, on, ok, onToggle) {
  return h('div', { class: 'power-row' },
    h('div', {}, h('strong', {}, title), h('p', { class: 'muted small' }, sub)),
    h('button', { class: 'switch', type: 'button', role: 'switch', 'aria-checked': String(!!on), 'aria-label': title, disabled: !ok, onclick: onToggle }, h('span', {})));
}
