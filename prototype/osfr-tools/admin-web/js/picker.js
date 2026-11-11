// Item tiles and the searchable item picker (used by the inventory tab, the mail composer and the command bar).
import { api, iconUrl } from './api.js';
import { h, clear, icon, modal, debounce, initials, empty, fail } from './dom.js';

// An item's picture: the gateway's icon when there is one, else a neutral tile with the name's initials.
export function itemArt(item, size = 'md') {
  const ph = h('span', { class: `art art-${size} art-ph`, 'aria-hidden': 'true' }, initials(item.name));
  const src = iconUrl(item);
  if (!src) return ph;
  const img = h('img', { class: `art art-${size}`, src, alt: '', loading: 'lazy', decoding: 'async' });
  img.addEventListener('error', () => img.replaceWith(ph), { once: true });
  return img;
}

// Resolves to {item, count, equip} or null.
export function pickItem({ title = 'Add an item', confirm = 'Add', withEquip = false, who = '' } = {}) {
  return modal('modal-picker', (close) => {
    let chosen = null;
    const results = h('div', { class: 'picker-results', role: 'listbox', 'aria-label': 'Items' });
    const search = h('input', { class: 'input input-lg', type: 'search', placeholder: 'Search items by name or id', 'aria-label': 'Search items', autocomplete: 'off' });
    const count = h('input', { class: 'input count-input', type: 'number', min: 1, max: 999, value: '1', 'aria-label': 'Count', inputmode: 'numeric' });
    const equip = h('input', { type: 'checkbox' });
    const chosenLine = h('div', { class: 'picker-chosen' }, h('span', { class: 'muted' }, 'Pick an item'));
    const add = h('button', { class: 'btn btn-primary', type: 'submit', disabled: true }, confirm);

    function choose(it, btn) {
      chosen = it;
      results.querySelectorAll('[aria-selected="true"]').forEach((b) => b.setAttribute('aria-selected', 'false'));
      btn.setAttribute('aria-selected', 'true');
      clear(chosenLine, itemArt(it, 'sm'), h('span', { class: 'picker-chosen-name' }, it.name), h('span', { class: 'muted' }, `#${it.item_id}`));
      add.disabled = false;
    }
    let token = 0;
    const load = debounce(async () => {
      const mine = ++token;
      try {
        const list = await api.items(search.value.trim());
        if (mine !== token) return;
        clear(results, list.length ? list.map((it) => {
          const btn = h('button', { class: 'pick', type: 'button', role: 'option', 'aria-selected': 'false', title: `${it.name} #${it.item_id}`,
            onclick: () => choose(it, btn), ondblclick: () => { choose(it, btn); form.requestSubmit(); } },
          itemArt(it), h('span', { class: 'pick-name' }, it.name), h('span', { class: 'pick-id' }, `#${it.item_id}`));
          return btn;
        }) : empty('No item matches that search', 'search'));
      } catch (e) { fail(e); }
    }, 140);
    search.addEventListener('input', load);
    search.addEventListener('keydown', (e) => {
      if (e.key === 'ArrowDown') { e.preventDefault(); results.querySelector('.pick')?.focus(); }
      if (e.key === 'Enter') { e.preventDefault(); const first = results.querySelector('.pick'); if (first) first.click(); count.focus(); count.select(); }
    });
    results.addEventListener('keydown', (e) => {
      const all = [...results.querySelectorAll('.pick')];
      const i = all.indexOf(document.activeElement);
      const cols = Math.max(1, Math.round(results.clientWidth / (all[0]?.offsetWidth || 1)));
      const to = { ArrowRight: i + 1, ArrowLeft: i - 1, ArrowDown: i + cols, ArrowUp: i - cols }[e.key];
      if (to !== undefined) { e.preventDefault(); (to < 0 ? search : all[Math.min(to, all.length - 1)])?.focus(); }
    });

    const form = h('form', { class: 'picker', method: 'dialog' },
      h('div', { class: 'modal-head' }, h('h2', {}, title), who ? h('span', { class: 'muted' }, who) : null,
        h('button', { class: 'icon-btn', type: 'button', 'aria-label': 'Close', onclick: () => close(null) }, icon('close'))),
      h('div', { class: 'picker-search' }, icon('search'), search),
      results,
      h('div', { class: 'picker-foot' }, chosenLine,
        h('label', { class: 'inline-field' }, h('span', {}, 'Count'), count),
        withEquip ? h('label', { class: 'check' }, equip, h('span', {}, 'Equip it')) : null,
        add));
    form.addEventListener('submit', (e) => {
      e.preventDefault();
      if (!chosen) return;
      const n = Math.max(1, Math.min(999, Math.round(Number(count.value) || 1)));
      close({ item: chosen, count: n, equip: withEquip && equip.checked });
    });
    load();
    queueMicrotask(() => search.focus());
    return form;
  });
}
