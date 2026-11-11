// Live zone map: the zone's map art when the gateway has one (a flat field when it does not), a dot per player,
// and the recent trail of the player being watched. Clicking or pressing Enter on a dot opens that player.
// bounds = [min_x, min_z, max_x, max_z] in world metres, drawn north up as the game's map lays its tiles:
// +x (north) up and +z (east) to the right.
import { svg } from './dom.js';
import { live } from './live.js';

const W = 1000;

export function zoneMap({ onPick, label = 'Zone map' }) {
  const el = svg('svg', { class: 'map-svg', role: 'group', 'aria-label': label, viewBox: `0 0 ${W} 600` });
  let last = null;
  new ResizeObserver(() => last && draw(...last)).observe(el);

  function draw(data, zoneName, watchId) {
    last = [data, zoneName, watchId];
    const [x0, z0, x1, z1] = Array.isArray(data.bounds) && data.bounds.length === 4 ? data.bounds : [-500, -500, 500, 500];
    const H = Math.round((W * (x1 - x0)) / (z1 - z0)) || 600;
    const k = W / (el.clientWidth || W); // keeps dots and labels the same on-screen size at any width
    const X = (p) => ((p[2] - z0) / (z1 - z0)) * W;
    const Y = (p) => ((x1 - p[0]) / (x1 - x0)) * H;
    el.setAttribute('viewBox', `0 0 ${W} ${H}`);
    const players = data.players || [];
    const showNames = players.length <= 14 && el.clientWidth > 520;
    const trail = watchId != null ? live.trails.get(watchId) : null;
    el.replaceChildren(
      data.image ? svg('image', { href: data.image, width: W, height: H, preserveAspectRatio: 'none' })
        : svg('rect', { class: 'map-ground', width: W, height: H }),
      data.image ? null : svg('text', { class: 'map-name', x: W / 2, y: H / 2, 'text-anchor': 'middle', 'dominant-baseline': 'central', 'font-size': Math.min(H / 4, W / 7) }, zoneName),
      trail && trail.length > 1 ? svg('polyline', { class: 'map-trail', points: trail.map((p) => `${X(p)},${Y(p)}`).join(' '), 'stroke-width': 3 * k }) : null,
      ...players.map((p) => {
        const cx = X(p.position);
        const cy = Y(p.position);
        const watched = String(p.character_id) === String(watchId);
        const g = svg('g', { class: `map-dot${watched ? ' is-watched' : ''}`, tabindex: '0', role: 'button', 'aria-label': `${p.name}, open player` },
          svg('title', {}, p.name),
          svg('circle', { cx, cy, r: 16 * k, class: 'map-hit' }),
          svg('circle', { cx, cy, r: (watched ? 8 : 6) * k, 'stroke-width': 2.5 * k }),
          svg('text', { x: cx + 11 * k, y: cy + 4.5 * k, 'font-size': 12.5 * k, 'stroke-width': 3 * k, class: showNames || watched ? 'map-label is-shown' : 'map-label', 'text-anchor': cx > W * 0.82 ? 'end' : 'start', dx: cx > W * 0.82 ? -22 * k : 0 }, p.name.split(' ')[0]));
        g.addEventListener('click', () => onPick(p.character_id));
        g.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onPick(p.character_id); } });
        return g;
      }));
  }
  return { el, draw };
}
