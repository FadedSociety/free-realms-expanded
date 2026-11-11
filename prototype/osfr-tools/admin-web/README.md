# Admin panel (web)

The staff web panel for the gateway: dashboard and live map, players (online now and everyone), the player drawer
(profile, inventory, quests, mail, moderation, notes, audit), mail to one, several or everyone, moderation, world
announcements, the audit log and server status. A command bar (Ctrl+K or `/`) runs every action by name, e.g.
`goto Pip`, `give Pip 12345 5`, `mute Pip 30m spamming`, `mail all Kart race tonight`.

Plain HTML, CSS and JavaScript modules. No build step, no dependencies, no requests to other hosts.

## How the gateway serves it
- The gateway serves this folder as static files (`index.html`, `css/`, `js/`, `mock/`) at one path, e.g.
  `/admin/`. All references inside are relative, so any mount point works. Module scripts need a JavaScript MIME
  type for `.js`; send `Cache-Control: no-cache` so an updated panel loads on the next visit.
- The panel calls the JSON API at absolute paths under `/admin/api/` (the gm-panel API contract) and item icons at
  `/admin/icon/<item_id>`. `js/api.js` holds the route table; `test/check.mjs` pins it to the contract.
- Sign-in: when `/admin/api/me` answers 401 the panel shows its sign-in form, posts `/admin/api/login`, keeps the
  token in `sessionStorage` and sends it as `Authorization: Bearer <token>`. Any later 401 returns to the form.
- The panel polls `/admin/api/status` and `/admin/api/players/online` every 2 s while the tab is visible.
- Works under a strict Content Security Policy (`default-src 'self'; img-src 'self' data:`, the `data:` for the
  favicon): no inline scripts or style attributes, no `eval`, and no HTML is ever built from API text.

## Mock mode
Open `index.html?mock=1`, or open the panel where `/admin/api/me` does not exist (a 404, a non-JSON page or no
server): it then reads `mock/*.json` and answers every route in memory, writes included, with a "Mock data" pill in
the top bar. All names, items and numbers in `mock/` are invented. Item icons are neutral tiles in mock mode.

To try it locally, serve this folder with any static server, e.g. `python -m http.server 8796` here, then open
`http://127.0.0.1:8796/?mock=1`. Sign out and back in works in mock mode with any account and password.

## Layout
- `css/tokens.css`: every colour, size, radius and timing, dark by default and light under `[data-theme="light"]`.
  Change the look here.
- `css/app.css`: layout and components, using the tokens only.
- `js/app.js` shell and router; `js/api.js` API client; `js/mock.js` mock API; `js/live.js` the shared 2 s poll;
  `js/actions.js` every staff action (buttons and command bar share it); `js/cmd.js` command bar; `js/drawer.js`
  player drawer; `js/picker.js` item picker; `js/map.js` live map; `js/ui.js` shared pieces; `js/views/` screens.

## Check
`node prototype/osfr-tools/admin-web/test/check.mjs` loads every module, compares the route table with the
contract, runs every route against the mock, checks the command parser, and scans for HTML parsing and outside
requests.
