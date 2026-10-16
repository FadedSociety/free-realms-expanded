# Studio Toolkit

the owner's direction (2026-10-15): decompiled functions do not give us dungeon bosses, enemy behaviour or content, so most systems are built new. Build them "in an intuitive and intelligent way", and build a **toolkit** so one engineer, or anyone who clones the repo, can move fast: hot-swap values, place NPCs, inspect anything, test without fighting the client UI. Inspirations he named: ForgeLight tools, Unity/Blender hooks, Roblox Studio, and a latitude/longitude-style location system.

## Shape

One local **Studio** service inside the gateway, enabled only with `--dev-tools` and bound to loopback. It is a small HTTP endpoint polled in the gateway's single-threaded loop, so it needs no new dependencies or threads. Three layers sit on it:

1. **Studio API (JSON).** Scripts, tests and the lead agent drive the live server directly:
   - `GET /api/state`: the character (zone, position, heading, job, level, health), the active encounter, and nearby actors.
   - `POST /api/command`: runs any `realm …` developer command as if typed in chat. Replies still show in the client.
   - `POST /api/reload`: hot-reloads content files (battles, job abilities, field enemies, then quests and NPCs) without a restart. This also removes the stale-binary/data mismatch seen in run 150132.
2. **Studio UI (one HTML page served at `/`).** Tables for every content file (quest rewards, battles, job abilities, NPCs) with inline edit, save and hot reload; a live **inspector** (nearby actors, with "teleport to", "move here" and "show data"); and a **world map**.
3. **Terrain and map.** A server-side **terrain height service** read from the zone's GCNK chunks. It gives the ground height at any (x, z) for teleports, NPC placement and AI ground-following, and permanently fixes the "phasing under the slopes" class of bugs. The map view uses the client's minimap tiles with click-to-teleport and drag-to-place.

## Decisions and pushback

- **Latitude/longitude.** The world already uses metre coordinates (x, z). Instead of a second coordinate system, the map view gives the same "pick a spot and go there" convenience: click the map, or type a grid reference such as `D7` or `x,z`. Height comes from the terrain service, so you always land on the ground. Named waypoints (`realm mark bridge`) can be stored for reuse.
- **Roblox-style dragging inside the game client** needs client modification, which the native client doesn't support. The equivalent lives in the Studio map and inspector: drag a marker or edit a value, and the server moves the actor live, so you watch it move in the game.
- **"Scripts inside a model":** our behaviour is data plus Rust (actor roles such as `skittish` and summons). The inspector shows an actor's data record and behaviour flags. Making behaviours data-driven (a behaviour field per actor) is the path to editing them in Studio.

## Phases

1. **API core:** state, command, and `realm here` (position, zone, nearest actors). This replaces chat-typing and click automation in testing.
2. **Hot reload:** battles, job abilities, field enemies.
3. **Studio UI:** content tables and inspector.
4. **Terrain heights and map:** click-to-teleport, place NPCs, named marks.
5. **Behaviour presets:** data-driven enemy behaviour (chase, skittish, summoner, ranged) editable in Studio.

6. **Asset graph:** links items, models, effects, animations, wield types, NPCs and abilities in both directions (e.g. "what uses effect 15449?"), as a reference and for the Studio inspector. Started 2026-10-15 at the owner's request, after the Player Studio weapons question showed the asset index lists everything but connects nothing.

Related: [[Debug Harness]], [[Build Ledger]].

## Status

- **Phase 1 built and verified natively (run 205627, 2026-10-15).** Start a run with `--dev-tools` (the default port is `127.0.0.1:47800`; use `--studio-port N`, or 0 to disable) and open `http://127.0.0.1:47800/`.
  - `GET /api/state`: character, gameplay summary, and the 60 nearest actors.
  - `POST /api/command` with `{"text":"goto Troll Summoner"}`: any `realm` command, run through the chat path, so its reply shows in the game.
  - `realm here`: zone, position and the nearest five actors.
- **Verified:** the API listed the summoner at 5 m along with the nearby troll NPCs, ran `goto` and `here`, and the page showed the live character (Medic, 900 health under the new job scaling).
- Example: `curl -s -X POST -d '{"text":"here"}' http://127.0.0.1:47800/api/command`
- **Phases 2, 3 and 6 built; whole toolkit verified natively (run client-baseline-20261015T224902Z, 2026-10-15).** The Studio page has nine tabs, and each was checked against the live game:
  - **Live:** state, cheat console (`level`, `heal`, `god`, `give`, `coins`, `battle`, `win`, `sim`, `fightlog`, `reload`).
  - **Content:** table and raw editor for every content file, saved with hot reload.
  - **Assets:** search over the client asset index (e.g. "trail gray" finds effect 5485).
  - **Inspector:** the asset graph (143k linked nodes, 403k links) is loaded by the browser, which does the searching, so the game loop never stalls. "ice axe" shows its class, frost effect, model, gray swing trail, and its Freezing Strike and Snowstorm specials.
  - **Packets** (recorded traffic plus poke), **Fight** (balance simulator and fight log), **Saves** (save and load states), **Report** (content checker) and **Client log**.
- **Bot client (`osfr-bot`) verified against the real gateway with real data:** login, cheat console, battle start, the zone change into the arena, the encounter details, then the win and its results. The run found that the bot ignored zone changes (31/28 ClientBeginZoning): the gateway waits for ready and loaded again, so the bot stalled outside the arena. Fixed: the bot now re-sends ready and loaded after any zone change once it is in the world. Regression test: `bot_zones_into_battle_arena_and_wins` (ignored by default because it needs lab reference data; run it with `--include-ignored`).
  - Example: `osfr-bot --gateway 127.0.0.1:PORT --reference DIR --cmd "chat realm battle 96" --cmd "expect 41 114" --cmd "chat realm win" --cmd "expect 39 19"`
  - Cheats need the `realm ` prefix. Chat replies arrive as 15/3. Families 39 and 45 use one-byte sub-codes, and the log shows them as `39/17 (0x6011)`.
  - Don't point a bot at the live player's character: it shares the save and would kick the native session. Give a bot its own gateway, or an account-store character.
- **Phase 4, terrain heights, map and marks (built 2026-10-15; verified with the bot and Studio against the real gateway and data; native client landing check pending):**
  - **Live check:** under the player's real saved position, the grid reads −45.81 and the native client recorded −45.77. A Studio map click teleported the bot to the clicked spot, onto the ground. Marks and placements round-tripped through Studio. The full suite passed (2059 tests, 0 failures).
  - **Evidence:** GCNK terrain vertex positions are world coordinates (x, y up, z). Scored against 3,264 Legacy FabledRealms NPC spawns, the 2 m grid has a median height error of **0.05 m**, and 85% of spawns are within 0.5 m. The rest stand on bridges and buildings. Swapped or negated axes were tens of metres out.
  - **Build:** run `python -I project/scripts/build_terrain.py`. It reads zone names and ids from the content (the world, dungeons and battle arenas) and runs `osfr-terrain`, writing `lab/terrain/zone_<id>.hgt`. FabledRealms is 2561×3073 nodes (15.7 MB) and builds in 3 s. The output is derived from client data, so it stays private and out of Git.
  - **Interiors are refused.** The tutorial cave (zone 411) ships a flat y = 0 terrain plane, while its floor (about y = 28) is room-model geometry. A grid there would have pulled cave enemies 28 m underground, so the builder writes nothing for flat placeholder terrain.
  - **Gateway:** `--terrain DIR`. The launcher passes `lab/terrain` when it exists.
  - **Enemies:** chase, return-home, the run-away summoner and the wandering battle marker now follow the ground. Wander goals are snapped too, so the walker can still reach them. Snapping is skipped where the terrain is more than 6 m from the enemy's own height (`MAX_SNAP`), because that means a bridge or building, and the grid holds terrain only.
  - **Commands:**
    - `realm goto <x> <z>`: lands 0.5 m above the ground.
    - `realm goto <x> <y> <z>`: an explicit height.
    - `realm mark <name>`, `realm marks`, `realm unmark <name>`, `realm goto mark <name>`: named marks, stored in `data/content/marks.json`, so they are shared through the repo.
    - `realm here` now also shows the ground height.
  - **Studio Map tab:**
    - Shows a shaded relief of the zone with north (+z) up: you (arrow), nearby actors, every Legacy NPC spawn, battle markers and marks.
    - Click to teleport onto the ground. Drag to pan, wheel to zoom. The cursor readout shows x, z and the ground height.
    - Routes: `GET /api/terrain` (grids), `GET /api/terrain/<zone>` (the raw grid; the browser draws it, so the game loop never renders anything), `GET /api/height?zone=&x=&z=`.
  - **Open:**
    - Map orientation versus the in-game minimap is unchecked. I chose +z as north.
    - Bridges and buildings have no heights yet. That needs runtime-object collision (`CDTA`, piece 064) placed with each object's transform.
- **NPC placer (built 2026-10-15; verified with the bot and Studio; native look pending):**
  - Placing where you stand puts the NPC on the ground under you. A teleport leaves you 0.5 m up, and the first live check showed NPCs would float. The ground is ignored if it is more than 6 m away (a bridge or roof).
  - `realm place <model id> [name text id] [at <x> <z>]`: places an NPC where you stand, facing your way, or on the ground at x, z, facing you. It appears in the game at once (35/2).
  - `realm unplace <guid|nearest>` removes one (35/3). `realm placements` lists the nearest.
  - `realm reload placements` applies hand edits to the file live.
  - **Storage:** `data/content/npc_placements.json`, kept separate from the Legacy spawns (`npc_spawns_extra.json`) so provenance stays clear. GUIDs start at 130,000,000,001.
    - Each record gets a `source` tag ("(R) placed with the Studio placer"). Hand-added fields such as notes survive the tools' rewrites, and marks work the same way.
    - The content checker validates their model and name ids automatically.
  - **Studio Map:** tick "place NPCs on click", enter a model id (and optionally a name text id), and click the map. The Placed NPCs list has go-to and remove buttons. Ids come from the Assets tab (models; text for names).
  - **Gap:** placed NPCs are scenery only, with no dialogue, quests or behaviour yet. Behaviour presets (phase 5) are the next layer.
- **Phase 5, behaviour presets (built 2026-10-15; unit-tested):**
  - `data/content/behaviours.json` holds `melee`, `skittish` and `stationary`. Each parameter's default is the old constant, so play is unchanged until content opts in.
  - **Parameters:** `move_speed`, `reach`, `attack_ms`, `chase_limit`, plus the skittish cycle's `run_ms`, `rest_ms`, `run_speed`, `leash`, `run_from` and `rest_animation`.
  - **Using a preset:** an enemy record in `field_enemies.json`, `field_battles.json` or `encounters.json` uses `"behaviour": "stationary"` or `{"preset": "melee", "move_speed": 6.0}`.
  - **Mistakes are refused at load and the old behaviour is kept:** an unknown preset or a misspelt field ("move_sped") is rejected.
  - **Hot reload:** `realm reload behaviours`.
  - **Tested:** a stationary Hooligan never moves but still swings within reach, and a 6 m/s override moves twice as far per tick.
- **Original phase 5 plan:**
  - **The problem:** enemy behaviour is constants in `encounter.rs` (3 m/s chase, 2.5 m reach, 1 s attacks, the summoner's 4 s run and 3 s rest, the 12 m leash). Tuning one enemy means a Rust change and a rebuild.
  - `data/content/behaviours.json` holds named presets with every parameter. Any enemy record names one with `"behaviour": "skittish"`, or overrides parameters with `{"preset": "melee", "move_speed": 4.0}`. Hot reload with `realm reload behaviours`.
    - **First presets:**
      - `melee`: the current chase-and-swing.
      - `skittish`: the summoner's gang-up, then run and rest when alone.
      - `stationary`: never moves.
      - `wanderer`: the battle marker's loop.
    - The old `skittish: true` flag stays as an alias, so existing content keeps working.
  - **Deliberately not yet:** a `ranged` preset. The first ranged invention looked broken in the game (wrong animation, endless resets). The Troll Novice Archer's "Troll Bow Shot" (42078) needs a real projectile effect and animation pair, found with the asset graph, before a preset is built for it.
  - **Studio:** the Content tab already edits JSON; the inspector will show an enemy's resolved behaviour. A Map overlay of aggro and leash circles would make tuning visual.
