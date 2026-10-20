---
type: ledger
updated: 2026-10-17
---

# Build Ledger

The verified state of every system as of **2026-10-17**, from the code at `0e5d122` (batches 6-9) and the newest native reviews. It was rebuilt from the code (what `src/gameplay.rs`, `src/gateway.rs` and `src/bin/osfr-gateway.rs` actually route) and the native reviews. It is not built from older summaries. Batches 6-9 are wired and suite-tested, with some bot coverage. No native acceptance records their new behaviour; rows distinguish that from mining's earlier native losses.

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

**Test suite:** 2747 passed, 0 failed at `0e5d122` (2026-10-17, clean review worktree, from the commit message; D01's full run also records 135 ignored).

## Platform and network

| System | Status | Evidence | Open gaps |
|---|---|---|---|
| SOE transport and sessions | Native-verified | `soe.rs`, `session.rs`; every native run | **Loopback only**: both servers refuse remote addresses (`osfr-gateway.rs:295,1759,1831`; `osfr-login.rs:82,468`). Single-threaded; it clones session state per message and has never been tested with 10 players. |
| Login and character select | Native-verified (one account, dev mode); key-pair mode built and bot-proven, never run natively | `login.rs`, `bin/osfr-login.rs`, `login_auth.rs`; LA, R4; `tests/login_broker.rs`, `tests/bot_broker_login.rs` | Dev mode (no `--account-store`) is unchanged: one account, login name = `--session`. **Key-pair mode (W94):** with `--account-store` and `--server-id-file`, the AUTH2 broker issues server-bound challenges, checks Ed25519 signatures and hands out one-use 60 s Session tokens; failure lockout and per-IP limits; non-enumerating replies. **Decision 2 (`34e8104`, 2026-10-16):** the broker may bind a private/VPN address with `--allow-lan` (public refused); the launcher accepts private broker endpoints and its join screen has a broker field. Launcher login and VPN bind never run natively. The list labels show "1"/"0" instead of names. |
| Character creation | Native-verified (old identity flow only) | Runs 20261008T065152Z and 072258Z | Creating through the account store has not been tested natively. Two-character create/list/select/delete is still open. |
| Account store and HMAC handoff | Wired | `account_store.rs`; `osfr-gateway.rs:742-814`; LA | LA used the legacy fixed-ticket bypass, so **the HMAC ticket path has never run natively**. Account ids and character GUIDs come from separate counters that both start at 1 (`account_store.rs:59-60`); **confirmed collision** in `players.rs` `guid_owners`, which mixes both namespaces (W103 ignored test `multiplayer.rs:540`). |
| Account-store handshake cost | Wired (suite-tested), never run natively | `bin/osfr-gateway.rs` `Session::new` clones one pre-login `Gateway` template; `3ae6d0c`; `tests/bot_acceptance_1016.rs` | Before `3ae6d0c`, every unauthenticated account-store handshake re-parsed all game content on the event loop (about 24 MB of NPC JSON, 2–3 s in a debug build), so handshakes piled past the 5 s login deadline. The template is built once and cloned; login still loads the character with current content. The commit measured handshakes at 3–4 ms. The end-to-end W154-F1 timing test is still `#[ignore]` at HEAD. |
| World entry (46 startup messages) | Native-verified | `gateway.rs:1366-1403`, built from `lab/content/startup-snapshot/startup.json`; SD (first fully built run) | The 46 captured templates are still loaded and hash-checked at startup, but they are no longer replayed. The snapshot keeps some opaque hex tails. |
| Self data (opcode 12) | Native-verified | `startup_selfdata.rs`; SD | It is still seeded from captured template #7. The action-bar rows are the captured ones. |
| Saves | Native-verified | `game_store.rs` (atomic replace), `characters/{guid}.json`; R5 runs 143737, 144157, 182347 | Each save rewrites the whole JSON file. **Pose-only changes are coalesced (`4ba97ad`):** the first is written at once, then at most every 5 s per session; tick and logout flush; gameplay changes still save immediately. A duplicate account or character selection is refused before any save is read (W103). After a client crash the player waits up to the 60 s idle timeout to log in again. No database. |
| Movement validation | Native-verified | `movement.rs` via `gateway.rs:957-1061`; R5 050310 | Limits are loose (R). Mount speed ×1 would snap a mounted player back. No collision. |
| Player registry and other-player visibility | Native-verified 2026-10-15 (bot + the owner) | `players.rs`; `Gateway::player_presentation`; `tests/multiplayer.rs` | Run 20261016T045046Z: the owner saw the bot spawn with its name, Capo's look and its chat, stand still, and despawn on timeout. Frida `add_pc` hooks: the client parses our AddPc and creates the actor. Fixed tonight: only GUID 1 could enter the world or basic-attack; one player's game exiting crashed the gateway (Windows 10054). Open: name ids, speed, wield type still (R); an earlier spawn/vanish was not seen (likely out of range); no two-real-client run yet. |
| Chat, friends, ignore, groups | Wired | `social_chat.rs`, `social_lists.rs` in the gateway binary | Two-bot say and tell delivered (W103). **Emotes (W139, `f1de554`):** animations come from the server's quick-chat table; Dance (3200) refused; mounted and moving-into-freeze refused (R); 2,469 of 2,470 real rows pass. **W173 (`0e5d122`):** 26/32 and 26/33 channel switches and 26/18 bubble colour persist per character in `<guid>.chat.json` beside the social list and load at join; the file and wire values are bounded (R). `tests/chat_settings.rs` and the gateway routing tests cover reads, writes and malformed input. Never tested natively. |
| Player click menu and Inspect (26/19, 26/10, 149) | Built and wired (W143, `4a7ab21`) | `player_radial.rs`, `Gameplay::public_look` | Inspect shows only unlocked jobs with ranks and the active job's worn items; resets on zoning; rate-limited. Never run natively. |
| Guilds and matchmaking | Built | `guilds.rs`, `matchmaking.rs` | Not routed. They wait on multiplayer. |
| Player trading | Wired, never run natively | `trade.rs`, `player_radial.rs`, `inventory.rs::adopt_state`, `bin/osfr-gateway.rs` trade commit; `tests/player_trade.rs`, `tests/trade_definition.rs`; `00bae61` | Trade is a server-built radial entry (locale 3299) chosen with 26/10. The live family-59 lifecycle uses 59/3 and 59/6–10: invites and offers are checked against each character's own inventory and coins, any offer change resets both locks and accepts, and distance, zoning, battle, disconnect, leaving or expiry cancels. Both accepts produce a settlement; the gateway rechecks both `before` states, stages both `after` states, writes a whole-or-absent `trade-<session>-<stamp>.trade-journal` beside the character saves, writes both saves, then removes it. Startup recovery finishes an interrupted commit; a stale settlement moves nothing. Two-client native check pending. |
| Bot client `osfr-bot` | Wired (gateway-level) | Scripts, re-zoning, battle loop (`bot_zones_into_battle_arena_and_wins`) | It skips the login server and has never run alongside a native client. |
| Developer authority | Enforced (tests and bot), never run natively | `account_store.rs` roles (W78), gateway/gameplay checks (W120); `tests/role_enforcement.rs` | Every account has a role (player, builder, admin; default player). **Decision 1 (`264db2a`):** on an account-store gateway, role-checked `realm` commands run without `--dev-tools`; diagnostics and wire experiments still need it. Refusals are logged as `DevCommandDenial`, Studio needs `--dev-tools` and an admin, `realm role` is admin-only. A loopback gateway without an account store is admin when `--dev-tools` is on. Bot check of a builder over the broker pending. |

## World and combat

| System | Status | Evidence | Open gaps |
|---|---|---|---|
| Open world zone 0 | Native-verified | `world.rs` `OPEN_WORLD_ZONE`; run 20261016T011257Z | The 17 `zones.json` entries are region labels, not instances. |
| Zone transfers (cave 411, arena 9600, kart) | Wired, native-verified per feature | `zone_transition.rs`; R5 battle runs | **`instance_id` is always 0** (`osfr-gateway.rs:455,1233`). The `world::World` instance coordinator exists but is unused. |
| Warpstones (56/57/90/122/58) | Built and wired (W142, `378a680`) | `restoration/piece_077.rs` | Nine curated warpstones, attune within 12 m, travel with cooldown, safe teleport to the nearest; friend teleport refused until wired to the registry. Never run natively. |
| Take Me There breadcrumb (98/1) | Wired, never run natively | `pathing.rs`, `gameplay.rs`; `tests/pathing.rs`; `3ae6d0c` | Answers the client's kind-1 98/1 request with 98/2: a straight line sampled every 5 m with terrain heights, at most 80 points, within the client's 300 m limit, at 4 answered requests/s per session. No obstacle avoidance; a missing grid gets the client's empty failure reply, and other kinds are left alone. Without a reply the client drops the target after 10 s. Native gate pending. |
| NPC population | Native-verified | `NpcVisibility` (100 m, 64 cap); R4 | 578 Legacy-placed regional NPCs plus 3,265 extra spawns. 621 regional rows are skipped because their positions are guesses. |
| NPC dialogue and quest offers | Native-verified | `npc_interaction.rs`; R5 212855, 215840, 230409 | Most dialogue bodies are blank. 26/20 is rejected. |
| Vendors (66/42 list, 66/43 buy, 165/5 sell) | Built, never run natively | `gameplay.rs` 66/43 route, `coin_shop.rs`, `npc_interaction.rs`, `vendors.json`; W148 (`e9088d3`) | 66/42 layout corrected to the client reader (lead-checked in IDA); 165/6 result first; native sells name merchant 0 = the open shop (R); refused sells answered. 62 of 192 vendors have fully grantable stock. |
| Coin store (165/4, 165/5) | Wired, never run natively | `coin_shop.rs`, `gameplay.rs` | The 10-15 "overwrites the live inventory" bug was already fixed at HEAD (live inventory synced before each purchase); W95 added a regression test whose canary re-breaks the sync. Purchases now grant through `Inventory::grant`, so catalogue items arrive with their definitions. |
| Terrain heights | Wired | `terrain.rs`, `lab/terrain` (zones 0 and 9600; median error 0.05 m); placer trolls stood on the ground in 011257 | No heights for bridges or buildings. Cave 411 has none (deliberately, its terrain is a flat placeholder). |
| Field enemies | Content placed, never run natively | `field_enemies.json`; R5 010757 (4 Hooligans, kill quest); W102 bot run | The 4 quest-gated Hooligans are unchanged. **W102 (2026-10-16):** 12 always-on enemies in four zone-0 regions, centres checked against the terrain grid; a scratch bot saw them spawn (AddNpc). Always-on loading and per-player respawn timers already exist. **W105:** home leash (drop target, walk home, heal), seeded idle roaming inside the spawn radius, and level scaling of HP and damage with the final-era helpers; a scratch bot dragged an enemy to 11.7 m of its 12 m leash and watched it return. Open (see [[Field Enemies]]): shared ownership between players (3.1), showing the level to the client (no level field found in AddNpc yet), native kill and respawn. |
| Combat power-ups | Wired, never run natively | `power_ups.rs`, `power_ups.json`, `encounter.rs`, `world.rs`, `bin/osfr-gateway.rs`; `tests/power_up_drops.rs`, `tests/power_up_group_heal.rs`; `b3dcbc9` | Every player who took part in a shared-field kill (damage within 30 s, or the enemy's target) gets their own roll and a pickup only their client sees; the optional participation block tunes the window and a group chance multiplier. Pickups clear on zone change, health and energy apply at once, and one kind is held. A big heart heals the rest of the picker's group in the same zone and instance once per kill, never a knockout. Key-3 use still waits on the owner's slot decision. |
| Free revive | Built (W147, `f24c3ba`) | `encounter.rs` 41/122, 41/125 | Held knockout, free revive to the checkpoint, paid refused, server auto-revive after 15 s. Never run natively. |
| Activity portal (167) | Built and wired (W150, `5f06115`) | `activity_portal.rs` | Field battles published at world entry; Play starts a battle after job/level checks; refresh throttled to 250 ms. Never run natively. |
| Friend map and group pings | Built and wired (W151, `86b1b0e`) | `social_lists.rs` | Mutual, online, not-ignored friends only; coordinates only in the same zone/instance. Never run natively. |
| Field battle 96 "Troll Summoner Madness!" | **Playable** | Full loop in R5 184205: marker, start screen, Go, arena, summons, win, loot wheel, return | Start-screen Switch Jobs is disabled. `required_combat_level` is not enforced. The spawner model 1917 doesn't render. Prizes are (R). |
| Opening cave 411 | **Playable** | Runs 081219, 114509 | "TOTAL SCORE 0" display. |
| Shared field enemies | Built and bot-proven (W127, `447e79b`) | `encounter.rs`, `world.rs`, `players.rs`; `tests/shared_field_interest.rs` | Players in one field instance see and fight the same enemies, kills, respawns and HP. Arena battles are still per player. Native two-client check pending. |
| Basic attacks per job | Partly | Brawler (43), Medic (11) and Archer (35) attack | **Ninja, Warrior and Wizard have no basic attack** in the runtime content. 2026-10-16: W85 staged Ninja/Warrior/Wizard attacks (bot-proven on scratch gateways), not yet in the play campaign; no weapon check. |
| Job abilities and level locks | Native-verified (Medic basics); 23 skills built, never run natively | `job_abilities.json`; R5 081330; W101 bot (level-5 Warrior bar shows Kick and Spinning Blade) | **W101 (2026-10-16):** 24 skills for six jobs with local (R) definition ids (definitions are server-sent, sub-opcode 0xd), client text, icon and effect ids, and effects for heal, damage over time, area damage, stun and buff; Shadow Minion (summon) is marked unavailable. Levels: Medic, Warrior and Wizard from guides; Ninja, Brawler and Archer use a 1/5/10/15 ladder (R) for the owner to correct. Each job beats a same-level enemy in tests. Ninja, Warrior and Wizard campaign bars still need W85 basic attacks merged. |
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
| Quest content (653 records) | Hollow quests hidden | `data/content/quests/*.json` | **W104 (2026-10-16):** the 631 quests that still have no objectives carry `"enabled": false` and are never offered (World Builder can flip them), so no offered quest completes on accept. W92's eight talk referrals are merged. **W110:** 12 quests enabled, every target resolving (3000020 newly enabled; 11 hidden again because a giver or target is not spawned); 15 more have staged objectives but stay hidden. See [[Research/Starting Quest Chains]]. Changing the objectives of a quest that is active in a save makes that save fail to load (counter-count check, `quest_engine.rs`); no current save is affected. Only 7 have rewards (coins only). 215 name a giver that isn't spawned. |
| Objective events | Wired, bot-proven (talk, collect) | `gameplay.rs` hooks, `quest_engine.rs`; W104 | Talk, kill, collect, equip, gather, craft, minigame, area and reach events reach the engine and only advance the player's active quests; area checks are indexed by zone. Bots completed talk quest 3000022 and a scratch collect quest. Not seen natively. |
| Completable quests | Native-verified (5) | Through play: 3000021, 3000059, 3000060, 3050011. With a dev offer: 3030007 | 3110002 Ninja "Stolen Away" (kill objective) and 7 talk quests are wired but untried. Carlos Brazenfist doesn't spawn, which blocks the Brawler chain. |
| Journal sync on relog | Wired | `active_login_packets` | A native relog check is pending. |
| Quest journal requests (47/12, 47/13, 49/15) | Built and wired (W145, `f4e397b`) | `quest_journal.rs` | Select/lock act only on the character's own active quests; breadcrumb starts wait for an original-id crosswalk; 47/14 tracked-target pin not built. |
| Inventory, categories, equip | Native-verified | [[Reviews/Native Inventory Repair 173814]]; SB; W138 | **Equip eligibility (W138, `44c7fd0`):** profile unlocked, members-only, gender, item and slot rank (Star rune 10, Moon rune 20), refusal chat line (R); rental expiry off until its unit is proven. Search: SB shows "hammer" no longer crashes, cause not proven for every query. |
| Item bar and potion | Built and wired (W146, `86b1b0e`) | `item_bar.rs` | Four saved slots, 42/6, 42/9, 38/25, bar casts, Small Health Potion (35 % heal, 10 s recast; R, tunable). Never run natively. |
| Item catalogue | Wired (loader and bundle gate), never run natively | `inventory.rs` merges `CONTENT/item-catalogue.json` under `inventory.json`; `scripts/content_check.py` item-coverage checks; `scripts/bundle/item_content.py`; `tests/inventory_catalogue.rs`; W84 plus `5d522cf` | The merged runtime set is **58,835 ids**, not 1,649: `inventory.json` alone has 1,649 captured ids, and the generated catalogue supplies the rest. Startup sends captured rows plus owned additions; a granted non-captured item sends its single-row batch first; 35/34 replies cover the merged set. Parsed content is cached per process (folder + file size/mtime), so logins and Coin Store purchases do not re-parse the 68 MB catalogue. `content_check` applies the loader's rules to item and ride consumers; the bundle gate refuses a missing or broken catalogue. Native grant/display remains unverified. |
| Coins and rewards | Native-verified | `rewards.rs`; R5 (359→364 from a quest, →384 from a battle) | The turn-in screen shows no reward line. |
| Crafting (cooking, smelting, forging) | Native-verified, opening tutorial only; recipe book built | 10-08/09 reviews; W144 (`4ba97ad`) | **Recipe book (26/24):** start a learned recipe at a station of the active job in range (R: the opening craft actors), exact inputs; completions no longer need the tutorial objective. Never run natively. Miner recipes have no ids yet. |
| Mystery chest | Native-verified (private branch) | Runs 194318, 194943 | Reconstructed chest; no normal way to get one. |
| Collections, achievements, titles | Built | `progression_extras.rs` | Not routed. |

## Minigames and lifestyle

| System | Status | Entry today | Open gaps |
|---|---|---|---|
| Kart | Native-verified (3 laps, 50 coins and 25 XP paid once, reload); NPC entry built | `realm race` (dev) or a gate NPC's menu (W149, `5f06115`) | Gate NPCs (R picks) and a saved, tunable reward cooldown; NPC entry never run natively. Solo only. Weapon effects partly done. Entering a race switches the job to Kart Driver (48); leaving restores the previous job (R). Kart Driver's native default outfit (4 items) is not in our starter content, so a Kart Driver shows no clothes. |
| Mining | Wired; native losses only | `realm mine` (dev) or a Copper Vein node (W171, `0e5d122`); `harvest_nodes.rs`, `mining_nodes.json`, `tests/gathering_nodes.rs` | No native win so far (runs 151307, 200750 lost). **W140 (`f4e397b`):** reconnect releases an unfinished run (unpaid run forfeited, R); once-only payout and Quit paths pinned by tests. **W171 (`0e5d122`):** a player clicks an in-range node in 26/8 to start the Miner run; a win pays 25 Miner XP (R), then the node hides after the results close and respawns after 60 s. The five Copper Vein placements, range, active cap and respawn interval are reconstructed. Whole-round native acceptance remains open. |
| Harvesting | Wired; native round unverified | `realm harvest` (dev) or a Mushroom node (W171, `0e5d122`); `harvest_nodes.rs`, `harvest_nodes.json`, `tests/gathering_nodes.rs` | Goal names show placeholders. W140 found no Quit/Leave defect in the live route and fixed reconnect; payout tested in-process, not natively. **W171 (`0e5d122`):** the ordinary node entry starts Harvesting111 for a Chef in range, pays 25 Chef XP on a win (R), and applies the same paid-hide/respawn path. Before starting, it refuses a full-quota payout that would not fit, with no state change. The Legacy-derived mushroom placements and policy are (R). Whole-round native acceptance remains open. |
| Minigame framework (`minigame.rs`) | Built | — | Unrouted modules use it. The combat encounter path is the proven one. |
| Fishing | Wired, never run natively | `fishing.rs`, `fishing_service.rs`, `fishing_spots.json`, `gameplay.rs`, `npc_interaction.rs`; `tests/fishing_entry.rs`; `3ae6d0c` | Six guide NPCs offer the trip, standing in for the boat (R). Admission checks the guide and busy state; start, catches and quit go through the server-owned minigame round and opcode 138. The round records its start zone and ends if the player changes zone or leaves the 50 m leash (R). A catch is paid once as its fish item plus 10 Fisherman XP when the job is unlocked (R); quit and expiry pay nothing, and reconnect releases the round. Native fishing processor and the guide menus have not been run natively. |
| Mounts and rides | Wired, never run natively | `mounts.rs`, `gameplay.rs`, `players.rs`, `bin/osfr-gateway.rs`; `tests/mount_wired.rs`, `tests/mount_bots.rs`, `tests/mount_follow.rs`, `tests/ride_sources.rs`; `3ae6d0c`, `b3dcbc9` | Owned rides use family 168 to spawn, board and put away; neighbours and late joiners get the ride actor, rider attachment and AddPc mount tail, and the ride receives a copy of seated rider movement. Rides end on zoning, battle, race or knockout, and mounted players cannot emote. The ride shop sells all 54 rides from Delvin (three also from Cragara); prices are Station Cash ×20 rounded to 50 coins, tunable and (R). Natively unseen; a vendor window with all 54 rides, bag-slot cost of ride items and duplicate purchases remain open. |
| Soccer, chess, derby, housing, pets | Built (registered, not routed) | none | Housing is also missing about 1,273 scene files. |
| Battle Mages, card game, ship combat, FRTW | Researched | none | — |
| Activity content | Mixed: private board packages, repository node tables | Opening, mining, harvesting and kart board content live in `lab/` packages, not in `data/content`; W171's reconstructed copper placements ship in `data/content/mining_nodes.json` and the mushroom placements in `harvest_nodes.json` | A playtest build must bundle one frozen board package and include both node tables. |

## Tooling

| Tool | Status | Notes |
|---|---|---|
| Studio (port 47800) | Native-verified, all 9 tabs plus Map | [[Design/Studio Toolkit]]. It acts on the first player in the world. |
| Terrain builder | Built and used | `scripts/build_terrain.py`; refuses flat placeholder terrain |
| World Builder: live world, NPC placer, literal and authored names, marks, text pool, locale build | Native-verified (names and text, run 011257) | [[Design/World Builder]]. Placements aren't sent to other sessions. No quest authoring or roles yet. |
| Content checker, asset index and graph | Built and used | 0 errors on the current content. The asset graph is private. |
| Debug harness (trace, Lua probe, Frida hooks) | Native-verified | [[Design/Debug Harness]] |
| Native function map | Mostly done, paused (Codex limit) | 830 of 901 shards accepted (2026-10-16; 61 pending, 5 failed, 4 queued, 1 in review). IDA cross-check of accepted shards: 99.5 % exact boundaries, 96.6 % callee agreement; review queue resolved (W115–W133). | Remaining shards wait for workers. IDA MCP works on `lab/working/ida/FreeRealms.exe.i64`. |
| Version control | In place | Private GitHub repo `FadedSociety/free-realms-expanded`; commit and push every milestone | Never commit `lab/`, saves or keys. |

## Integration backlog (built, not wired)

Value calls are from the 10-15 audit.

- **Worth routing for the playtest:** vendor buy.
- **Later:** pets, housing, collections and titles, chess.
- **After multiplayer works:** guilds, matchmaking.
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
