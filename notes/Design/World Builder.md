# World Builder

the owner's direction (2026-10-15): build the world **from inside the running game**. A builder places an NPC, gives it a new quest, links quests, and sends a quest to everyone online. Example: an admin scavenger hunt. The admin places an NPC, posts hint photos, and the first players to find it get goodies. Everything persists past a server restart.

Several builders walking the world could build it out together. Later, public sandbox instances would let open-source contributors fill the content gaps (quests, NPCs, items), "like Roblox Studio or Unity". Only trusted people can change or delete things.

This builds on the [[Studio Toolkit]]: placer, marks, terrain, hot reload and behaviour presets.

## What the client allows (checked 2026-10-15)

- **Quest text is locale ids only.** The client has no quest definition table. Titles, descriptions and objective text arrive as locale ids inside the server's quest records, and the client looks them up in `locale/en_us_data.dat`. An unknown id shows `<STRING n NOT FOUND>` ([[Build Guides/Gameplay/Quests and Objectives]], items 140 and 239). Many retail quest titles are missing from our client copy, so the original game must have shipped locale updates.
- **New text is possible, through a locale overlay.**
  - The client reads the loose files `./locale/en_us_data.dat` and `.dir` at startup (native `00e8e360`, `00e861e0`).
  - The MIT community tool FreeRealmsLocaleTools (pinned `ba2ede22`) adds entries and writes both files. It has already been used unchanged for the opening journal labels: 28 new ids, every original entry and id preserved, strict round trip ([[Research/Opening Journal Reconstructed Labels]]).
- **The consequence:** authored text appears for a player after their client restarts with the updated locale pair. It is not instant.
- **Live paths that need no restart** (both verified natively on 2026-10-15; see the native text check below):
  - Chat and system messages are literal text, enough for event announcements and hints.
  - AddNpc (35/2) has a literal `npc_name` field next to `name_string_id`. With the id at 0 the nameplate shows it: **verified natively 2026-10-15** (run 20261016T011257Z).
- **No Rust client is needed for this.** A replacement client would remove the restart step and allow new UI. It is a multi-month project with bigger reasons to exist, and this design does not depend on it.

## Layers

1. **Restoration content** (`data/content`, in Git, reviewed): the reconstructed Free Realms.
2. **Live world** (`--world DIR`, persisted, outside Git): everything built in-game. It holds placed NPCs, authored quests, authored text, marks and events, and is loaded on top of the restoration content. It can be switched off, and events can expire.
3. **Publishing:** `realm publish` (admin) exports chosen live-world pieces as content files for a Git pull request. That is the path from a sandbox into the real world: built by walking around, reviewed before it ships.

## Authority

Roles per account: **player**, **builder** and **admin**.
- **Builder:** place, edit and remove their own NPCs, quests, text and marks; test quests on themselves.
- **Admin:** everything a builder can do, plus editing or removing anyone's content, broadcasting quests and events to all players, publishing, and granting roles.
- **Storage:** `roles.json` in the live-world folder.
- **Local single-player:** `--dev-tools` keeps today's behaviour, where the local player is admin.
- **Audit:** every change records who made it and when, so mistakes can be traced and undone.

## Text

`realm text <words>` (or Studio) registers a string and returns its id. Ids come from a reserved pool that is unused in the stock locale and inside the community tool's proven id domain. Strings live in `world/text.json`.

The launcher builds each run's client locale pair from the stock files plus the overlays. A public server would serve the overlay through its patcher. The server warns when content uses text that players' clients won't have until they restart.

## Quests

Authored quests use the existing quest engine, whose objective kinds include talk to an NPC, defeat a kind of enemy, collect an item, enter an area, reach a point, finish a minigame, use an item, equip an item and complete a dungeon. Rewards are coins and items.
- **Commands** (Studio shows the same as a form):
  - `realm quest new <title text>` makes a quest with you as the author.
  - `realm quest giver <npc guid | nearest>`.
  - `realm quest goal talk|kill|collect|reach ...`.
  - `realm quest reward coins|item ...`.
  - `realm quest after <quest id>` links it to an earlier quest.
- **Testing:** `realm quest test <id>` offers it to yourself.
- **Releasing** (admin): `realm quest give <id> all` offers it to everyone online, with a chat announcement.

## Events

An event bundles NPCs, quests, a start, an end and a winner limit. **Scavenger hunt:**
1. The admin places the hidden NPC and gives it a "find me" quest.
2. `realm event start hunt` announces it in chat and offers the quest to everyone online.
3. The first N players to talk to the NPC get the prize. The rest get a thank-you line.

The event ends after N winners or at its end time. The NPC can stay or vanish.

**Built so far (T418):** a quest marked `live_event` in content (the hunt, 3000200) is offered only while an admin has it open. The admin commands are `realm event open <id>`, `realm event close <id>` and `realm event list`. They keep `<live_dir>/events.json`, and every session sees a change within about a second. They do not announce anything: the operator sends the announcement with `realm broadcast` (script in `notes/Autopilot/T281.md`). There is no `realm event start` yet, and no start or end time, winner limit or per-event NPC placement.

## Order of work

1. Native text check, one session: an NPC with a literal `npc_name`; an NPC and a quest using overlay text ids. *(needs the game window)*
2. Live-world layer: move the placer and marks there, add `--world`, and persist everything.
3. Text pool plus the launcher overlay build.
4. Quest authoring commands, then the Studio quest form.
5. Roles, permissions and the audit trail.
6. Broadcast and events (scavenger hunt).
7. Publish to a content pull request; sandbox instances for public builders.

## Status

- **Built 2026-10-15 (unit-tested):**
  - **Live world** (`--world`, the launcher uses `lab/world`): placed NPCs and marks are saved there. They survive restarts and stay out of `data/content`. Studio reads them from `/api/world/`.
  - **Literal NPC names:** `realm place <model> ... named <Any Name>` keeps capitals and sends the name in AddNpc's `npc_name`.
  - **Text pool:** `realm text <words>` returns a stable id from 7,000,000 upward and saves it to `world/text.json`. Identical text reuses its id, and ids whose locale hash is a stock row are skipped (`--client-locale` loads the stock keys). `realm texts` lists recent entries. The Rust hash matches the Python and community-tool values.
  - **Locale build:** `scripts/build_world_locale.py` adds the world text to a locale pair with the pinned MIT tool, at the server's ids.
    - Tested on a copy of the stock pair: 2 rows were added, all 113,286 stock rows were preserved, a strict re-read passed, and a second build was a no-op with an identical file hash.
    - The launcher applies it to every dev run's client copy.
  - **(R) to verify natively:** ids above the tool's search limit (7,000,000+) should work, because the client looks rows up by hash.
- **Outside work seen:** another fan's closed-source 3D editor (Explorer, Properties, move and drop-to-ground, quests, text, publish), built on the existing community ForgeLight tools. It can't be used, but it proves the target. Our plan is a native Rust editor whose renderer later becomes the Rust client, built after the server-side authority.

## Native text check (passed 2026-10-15, run client-baseline-20261016T011257Z)

Three troll NPCs were placed with `realm place` and shown in the real client. Screenshot: `lab/runs/client-baseline-20261016T011257Z/world-text-nameplates.png`.
- **Stock name id 15623:** "Troll Summoner" (the control).
- **Authored text id 7,000,000:** "Bob the Lost". The locale overlay works, and ids above the community tool's search limit work, because the client looks rows up by hash.
- **`name_string_id` 0 with a literal `npc_name`:** "Bob Literal Name". **NPC names need no restart**, so a scavenger-hunt NPC can be named live.

Quest titles and descriptions resolve through the same locale lookup. They need a client restart after the text is authored.

## Open questions

- Can the locale be reloaded without restarting the client? No evidence either way; assume not.
- How to keep a public sandbox clean: per-builder instances, rate limits, and a review queue in Studio.
