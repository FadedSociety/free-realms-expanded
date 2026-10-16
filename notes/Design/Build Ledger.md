---
type: ledger
updated: 2026-10-15
---

# Build Ledger

The verified state of every system as of **2026-10-15**. It was rebuilt from the code (what `src/gameplay.rs`, `src/gateway.rs` and `src/bin/osfr-gateway.rs` actually route) and the newest native reviews. It is not built from older summaries.

The plan for what comes next is [[Build Plan]]. **Update this ledger whenever a system changes state**, and give the evidence (code path, test, run id). The 2026-10-13 ledger it replaces was badly stale: for example, it said world entry replayed 46 captured messages, which was fixed on 10-14. The old copy is kept outside the vault in `lab/snapshots/notes-20261015/`.

**Status words** (use exactly these):

- **Researched:** notes only.
- **Built:** code and tests, but not routed.
- **Wired:** routed in the live gateway.
- **Native-verified:** seen working in the original client (cite the run).
- **Playable:** a normal player can reach it and finish it without dev commands.

**Review shortcuts:**

- **SB:** [[Reviews/Native Startup Builders 215401]]
- **SD:** [[Reviews/Native SelfData 011719]]
- **LA:** [[Reviews/Native Login Accounts 023623]]
- **R4:** [[Reviews/Native Round4 NPCs 035300]]
- **R5:** [[Reviews/Native Round5 050310]] (a running log through 10-15)

**Test suite:** 2065 passed, 0 failed (2026-10-15, after World Builder text).

## Platform and network

| System | Status | Evidence | Open gaps |
|---|---|---|---|
| SOE transport and sessions | Native-verified | `soe.rs`, `session.rs`; every native run | **Loopback only**: both servers refuse remote addresses (`osfr-gateway.rs:295,1759,1831`; `osfr-login.rs:82,468`). Single-threaded; it clones session state per message and has never been tested with 10 players. |
| Login and character select | Native-verified (one account) | `login.rs`, `bin/osfr-login.rs`; LA, R4 | **One account and one session at a time.** There is no password; the login name must equal `--session` (`login.rs:314`). A second handshake replaces the first (`osfr-login.rs:459-483`). The list labels show "1"/"0" instead of names. |
| Character creation | Native-verified (old identity flow only) | Runs 20261008T065152Z and 072258Z | Creating through the account store has not been tested natively. Two-character create/list/select/delete is still open. |
| Account store and HMAC handoff | Wired | `account_store.rs`; `osfr-gateway.rs:742-814`; LA | LA used the legacy fixed-ticket bypass, so **the HMAC ticket path has never run natively**. Account ids and character GUIDs come from separate counters that both start at 1 (`account_store.rs:59-60`). |
| World entry (46 startup messages) | Native-verified | `gateway.rs:1366-1403`, built from `lab/content/startup-snapshot/startup.json`; SD (first fully built run) | The captured templates are a fallback only, enabled with `OSFR_GATEWAY_STARTUP_TEMPLATE_FALLBACK=1`, but they are still loaded and hash-checked at startup. The snapshot keeps some opaque hex tails. |
| Self data (opcode 12) | Native-verified | `startup_selfdata.rs`; SD | It is still seeded from captured template #7. The action-bar rows are the captured ones. |
| Saves | Native-verified | `game_store.rs` (atomic replace), `characters/{guid}.json`; R5 runs 143737, 144157, 182347 | Each save rewrites the whole JSON file. A rejected duplicate login may save a stale snapshot (inferred, untested). No database. |
| Movement validation | Native-verified | `movement.rs` via `gateway.rs:957-1061`; R5 050310 | Limits are loose (R). Mount speed ×1 would snap a mounted player back. No collision. |
| Player registry and other-player visibility | Wired, never seen with two clients | `players.rs`; `osfr-gateway.rs:450-530,1222-1249`; `tests/multiplayer.rs` 1 | **No native run has ever had two players.** The AddPc message is a stub (model 0, empty name and appearance, `players.rs:301-360`). Position relays drop the state bytes. Account ids and character GUIDs share one key space, so they can collide. |
| Chat, friends, ignore, groups | Wired | `social_chat.rs`, `social_lists.rs` in the gateway binary | **Probable bug:** with `--content`, social packets are dropped unless `--trace` is on (`gateway.rs:887,1128-1156`). Inferred from code, untested. Never tested natively. |
| Guilds, trade, matchmaking | Built | `guilds.rs`, `trade.rs`, `matchmaking.rs` | Not routed. They wait on multiplayer. |
| Bot client `osfr-bot` | Wired (gateway-level) | Scripts, re-zoning, battle loop (`bot_zones_into_battle_arena_and_wins`) | It skips the login server and has never run alongside a native client. |
| Developer authority | Wired | `--dev-tools` | It is one gateway-wide flag, so **every player gets the cheats**. There are no player, builder or admin roles. |

## World and combat

| System | Status | Evidence | Open gaps |
|---|---|---|---|
| Open world zone 0 | Native-verified | `world.rs` `OPEN_WORLD_ZONE`; run 20261016T011257Z | The 17 `zones.json` entries are region labels, not instances. |
| Zone transfers (cave 411, arena 9600, kart) | Wired, native-verified per feature | `zone_transition.rs`; R5 battle runs | **`instance_id` is always 0** (`osfr-gateway.rs:455,1233`). The `world::World` instance coordinator exists but is unused. |
| NPC population | Native-verified | `NpcVisibility` (100 m, 64 cap); R4 | 578 Legacy-placed regional NPCs plus 3,265 extra spawns. 621 regional rows are skipped because their positions are guesses. |
| NPC dialogue and quest offers | Native-verified | `npc_interaction.rs`; R5 212855, 215840, 230409 | Most dialogue bodies are blank. 26/20 is rejected. |
| Vendors (66/42 list, 66/43 buy) | Wired, never run natively | `gameplay.rs:1213,1340`, `vendors.json` | 66/43 is parsed but **not acted on**. Only 62 of 192 vendors have stock that exists in the catalogue. |
| Coin store (165/4, 165/5) | Wired, never run natively | `coin_shop.rs`, `gameplay.rs:1278` | **Bug:** the shop keeps a copy of the inventory from login and overwrites the live inventory on purchase (`gameplay.rs:503-511,1318`). A refusal sends no reply. |
| Terrain heights | Wired | `terrain.rs`, `lab/terrain` (zones 0 and 9600; median error 0.05 m); placer trolls stood on the ground in 011257 | No heights for bridges or buildings. Cave 411 has none (deliberately, its terrain is a flat placeholder). |
| Field enemies | Native-verified | `field_enemies.json`; R5 010757 (4 Hooligans, kill quest) | **Only 4 enemies exist, and only while quest 3030007 is active.** Zone 0 has no free-roaming enemies. |
| Field battle 96 "Troll Summoner Madness!" | **Playable** | Full loop in R5 184205: marker, start screen, Go, arena, summons, win, loot wheel, return | Start-screen Switch Jobs is disabled. `required_combat_level` is not enforced. The spawner model 1917 doesn't render. Prizes are (R). |
| Opening cave 411 | **Playable** | Runs 081219, 114509 | "TOTAL SCORE 0" display. |
| Encounter instancing for groups | Not built | Each player's `Gameplay` owns its own enemies | Players in the same arena see each other but **fight separate enemy copies**. |
| Basic attacks per job | Partly | Brawler (43), Medic (11) and Archer (35) attack | **Ninja, Warrior and Wizard have no basic attack** in the runtime content. |
| Job abilities and level locks | Native-verified (Medic) | `job_abilities.json` (Medic Triage and First Aid); R5 081330 | The other five combat jobs have no abilities. The green heal number hasn't been seen. |
| Job XP and levels | Wired | `award_job_xp`; battles give 50 XP (R) | A level-up from combat has not been seen natively. |
| Final-era balance | Wired (R) | `encounter.rs` scaling | Not checked against original numbers. |
| Behaviour presets | Built | `behaviours.json`, `realm reload behaviours` | No content uses them yet (battle 96 still uses the old `skittish` flag). |
| `enemies.rs` / `enemies.json` | Built, superseded | 55 definitions, not read at runtime | Data to reuse for field enemies. |
| Dungeons beyond the cave | Researched | 74 files in `data/content/dungeons/`. Only 1 has a numeric zone id or scene name; 293 of 367 enemy references are unresolved; goals are prose | None are playable. The template for building one is battle 96 (see [[Build Plan]]). |
| Opening (17 stages) | Native-verified | [[Reviews/Native Opening 17 103605]] | — |

## Quests and economy

| System | Status | Evidence | Open gaps |
|---|---|---|---|
| Regional quest engine (49/1 offer, 49/2, 49/3, 49/13, 49/14) | Native-verified | `quest_engine.rs`; R5 143124, 212855, 215840 | `player_level` is hard-coded to 1 (`gameplay.rs:737`). Quests loaded from content get empty requirements, so no gate applies. |
| Quest content (653 records) | Mostly hollow | `data/content/quests/*.json` | **Only 14 have objectives; about 639 complete the moment they are accepted.** Only 7 have rewards (coins only). 215 name a giver that isn't in the NPC content. |
| Objective events | Partly wired | Only `TalkToNpc` and `KillActorKind` reach the engine | Collect, gather, craft, minigame, area, reach and equip events are never sent. |
| Completable quests | Native-verified (5) | Through play: 3000021, 3000059, 3000060, 3050011. With a dev offer: 3030007 | 3110002 Ninja "Stolen Away" (kill objective) and 7 talk quests are wired but untried. Carlos Brazenfist doesn't spawn, which blocks the Brawler chain. |
| Journal sync on relog | Wired | `active_login_packets` | A native relog check is pending. |
| Inventory, categories, equip | Native-verified | [[Reviews/Native Inventory Repair 173814]]; SB | Only 1,649 item definitions. Search: SB shows "hammer" no longer crashes, but the cause isn't proven for every query. |
| Item catalogue | Gap | The server only has the 1,703 captured definition rows. Legacy `ClientItemDefinitions` has 58,795. | 16,551 of 18,017 content item references (mostly `stores.json`) can't be granted. Ninja, Wizard and Warrior get **no starter weapon**. |
| Coins and rewards | Native-verified | `rewards.rs`; R5 (359→364 from a quest, →384 from a battle) | The turn-in screen shows no reward line. |
| Crafting (cooking, smelting, forging) | Native-verified, opening tutorial only | 10-08/09 reviews | No world crafting stations. W60's recipe changes haven't been re-checked natively. |
| Mystery chest | Native-verified (private branch) | Runs 194318, 194943 | Reconstructed chest; no normal way to get one. |
| Collections, achievements, titles | Built | `progression_extras.rs` | Not routed. |

## Minigames and lifestyle

| System | Status | Entry today | Open gaps |
|---|---|---|---|
| Kart | Native-verified (3 laps, 50 coins and 25 XP paid once, reload) | `realm race` (dev only) | No NPC or menu entry. Solo only. Weapon effects partly done. |
| Mining | Wired | `realm mine` (dev) | No native win so far (runs 151307, 200750 lost). Dead board. W38's lifecycle adapter isn't called. |
| Harvesting | Wired | `realm harvest` (dev) | Goal names show placeholders. Leave/Quit fails. No payout. |
| Minigame framework (`minigame.rs`) | Built | — | Unrouted modules use it. The combat encounter path is the proven one. |
| Fishing, soccer, chess, derby, housing, pets, mounts | Built (registered, not routed) | none | Housing is also missing about 1,273 scene files. |
| Battle Mages, card game, ship combat, FRTW | Researched | none | — |
| Activity content | Private packages only | Opening, mining, harvesting and kart content live in `lab/` packages, not in `data/content` | A playtest build must bundle one frozen package. |

## Tooling

| Tool | Status | Notes |
|---|---|---|
| Studio (port 47800) | Native-verified, all 9 tabs plus Map | [[Design/Studio Toolkit]]. It acts on the first player in the world. |
| Terrain builder | Built and used | `scripts/build_terrain.py`; refuses flat placeholder terrain |
| World Builder: live world, NPC placer, literal and authored names, marks, text pool, locale build | Native-verified (names and text, run 011257) | [[Design/World Builder]]. Placements aren't sent to other sessions. No quest authoring or roles yet. |
| Content checker, asset index and graph | Built and used | 0 errors on the current content. The asset graph is private. |
| Debug harness (trace, Lua probe, Frida hooks) | Native-verified | [[Design/Debug Harness]] |
| Native function map | Paused 10-14 (rate limit) | 824 of 897 shards accepted, 99,133 labels. 6 shards are stuck and need a reset before restarting. |
| Version control | **Missing** | The project git repository has **no commits**, so there is no change history for 74 workers and the lead. |

## Integration backlog (built, not wired)

Value calls are from the 10-15 audit.

- **Worth routing for the playtest:** minigame lifecycle adapters (mining/harvesting win and payout), vendor buy, fishing.
- **Later:** pets, mounts, housing, collections and titles, harvest nodes, chess.
- **After multiplayer works:** trade, guilds, matchmaking.
- **Restoration pieces worth integrating:**
  - 046 consumables
  - 073 points of interest
  - 077 warpstones
  - 075 postman
  - 093 emotes
  - 057/058 loot and death, merged into encounters
  - 032–043 job progressions, feeding job abilities
  - 052/053 patrol and aggro, feeding behaviour presets
- **Obsolete pieces:** about 40 are superseded (the list is in the 10-15 audit). Don't integrate them.
