---
updated: 2026-10-27
---

# Project Map

**Last refreshed 2026-10-27**, against the checkout at tip `0db6412d`. Counts below were taken from that tree and are marked "counted" so you can redo them.

Where everything lives, for agents. It's a map, not a status report. Status lives in [[Design/Build Ledger]] (state per system) and [[Design/Build Plan]] (what's next). Before claiming something is missing, check the code and the newest `Reviews/Native *.md` ([[Start Here]]). Real-client runs since 2026-10-20 are not in `Reviews/` yet; the Ledger cites their run ids.

Refreshed by the lead after each integration batch. If you add, rename or retire a module, update its row here in the same change.

## Top level

| Path | What it is |
|---|---|
| `.` (repository root) | Git repo `FadedSociety/free-realms-expanded`, public. Commit only your own files; never commit `lab/`, saves, keys, client files or decompiler output (the publishing rules are in `AGENTS.md`). Root files: `README.md` (what's new), `ROADMAP.md` (what is unfinished, known bugs, how to help), `AGENTS.md`, `MODLOG.md` (project journal), `NOTICE`, `LICENSE-MIT`, `LICENSE-APACHE`, `recovery-manifest.json` and `setup-manifest.json` (2026-10-08 intake records), `docs/images/banner.png`. |
| `prototype/osfr-tools/` | The Rust server crate `osfr-tools` (edition 2024): library `src/` (counted 121 module files at the top level: 119 declared in `lib.rs`, plus `mining_lifecycle.rs` and `harvesting_lifecycle.rs`, which are `#[path]` children of `mining.rs` and `harvesting.rs`), subfolders `src/gameplay/`, `src/gateway/`, `src/encounter/`, `src/world_events/`, 92 restoration pieces (`src/restoration/`), binaries (`src/bin/`), content (`data/`), the Studio page (`studio/index.html`, `studio/assets/`) and 464 Rust test files (`tests/`). Dependencies: `miniz_oxide`, `serde`, `serde_json`, `sha2`, `ed25519-dalek`, `getrandom` (stays 0.2), `subtle`. Dependencies build at opt-level 2 in dev and test builds (L063). |
| `scripts/` | Python, PowerShell and Java tooling (below). `scripts/cargo.ps1` selects the maintainers' private Rust toolchain; outside the maintainers' machine use plain `cargo`. |
| `tests/` | Nine stdlib Python checks for the scripts (asset delivery, bundle gates, client overlay, live log, minigame and quest coverage audits, stress harness, wiki prices). They are not Rust tests; see "Tests and gates". |
| `notes/` | This Obsidian vault. |
| `docs/images/` | README banner only. |
| `lab/` | **Not in Git.** Everything private: original client, working copies, captures, references, the IDA database, build dirs, worker runs, reports, the autopilot pipeline's queue and logs. Paths under `lab/` below are for maintainers; none of it is published. |
| (launcher) | **The launcher is its own repository**: <https://github.com/FadedSociety/free-realms-launcher> (egui, key-pair login, patching, join-a-realm). The old `launcher/` folder was removed from this tree on 2026-10-26 (L067). What stays here: `scripts/launcher/friend_launcher.py` (a stdlib script that prepares a supplied client for a LAN server) and `scripts/launcher/gamepad_bridge.py` with `gamepad.json`. |
| (removed) | `scripts/pieces/` and `tests/pieces/` no longer exist. The restoration pieces are `prototype/osfr-tools/src/restoration/piece_NNN.rs` with matching `prototype/osfr-tools/tests/piece_NNN.rs`. |

## Server binaries (`src/bin/`)

| Binary | Role |
|---|---|
| `osfr-gateway` | The game server. Owns sessions, SOE transport, application dispatch, the shared world, saves and security limits; single-threaded (no thread is spawned in production code). `Server::step` drains datagrams; world and session ticks run at 20 Hz. Options include `--account-store`, `--handoff-key-file`, `--content`, `--world`, `--terrain`, `--security-log`, `--tick-stats-seconds`, `--dev-tools`, `--studio-port` (default 47800) and `--allow-lan`. It binds loopback; a private or VPN address is accepted only with `--allow-lan` (`net_policy::allowed`), and startup refuses a non-loopback bind unless an account store is set and `--dev-tools` is off. Studio is loopback-only and needs `--dev-tools` plus an admin. When the card content names a port, `osfr_gateway/card_listener.rs` runs the Card Duelist TCP service beside the UDP socket. |
| `osfr_gateway/` (children of `osfr-gateway.rs`) | Production modules: `admin_move`, `ambient`, `boombox_routing`, `card_listener`, `client_area_routing`, `delivery` (application framing), `encounter_invites`, `moderation`, `paths`, `t006_guilds` (family 129), `tick_stats` and `world_event_routing`. Everything else in that folder is `#[cfg(test)]` (`tests.rs` and the per-task routing suites such as `t020_group_routing`, `w163_trade_routing`, `l042_moderation`, `isolation`, `dormant_fields`). |
| `osfr-login` | Native login and character select, plus the AUTH2 key-pair broker. Defaults: login on `127.0.0.1:20042`, broker (`--auth-bind`) on `127.0.0.1:20043`. Key-pair mode needs `--account-store` and `--server-id-file`; a private/VPN address needs `--allow-lan`. |
| `osfr-account` | Host-only account provisioning: `create`, `add-key`, `remove-key`, `set-role` (player, builder, moderator, admin), `member` (on or off) and `list`. Takes only public keys. |
| `osfr-bot` | Headless bot client for tests and stress: scripts of `chat`, `move`, `attack`, `ability`, `raw`, `expect` steps; direct login or the broker (`--broker-profile`, `--allow-lan`). |
| `osfr-terrain` | Builds ground-height grids from client terrain chunks (uses `restoration/piece_098.rs`). |
| `soe-probe` | First transport-slice probe (historical). |
| `osfr-tools` (default binary, `src/main.rs`) | Pack tools: `catalog` and `extract` over the supplied client's asset packs. |

## Server library (`src/`) by system

The request path runs packet → `gateway.rs` (per-character application gateway; `application_handler` labels each family/subtype and is also what routing uses, so behaviour never depends on `--trace`) → `gameplay.rs` (in-world dispatch; stages changes on a clone, so rewards commit together) → the system modules below. `bin/osfr-gateway.rs` handles cross-player routing: chat, social, shared fields, trade commit, guilds, boomboxes, events, inspect. `gateway/` holds the session-gateway helpers moved out of the hub (`admin_move`, `application_preload`, `companions`, `encounter_join`, `staff`, `startup_stream`, `trade_bridge`).

| System | Modules |
|---|---|
| Transport and session | `soe.rs`, `soe_send.rs` (SOE framing and reliable channel), `session.rs`, `net_policy.rs` (bind and peer rules, token buckets), `trace.rs` (the one packet-redaction rule every tracer uses), `jump_relay.rs` (opcode 164 jump update), `bundle_budget.rs` (outbound reply budget per client message) |
| Login, accounts and security | `login.rs`, `login_auth.rs` (AUTH2 key-pair), `account.rs`, `account_store.rs` (roles, public keys, membership), `character_store.rs`, `identity.rs`, `security_log.rs`, `moderation.rs` (mute, kick, ban stored per account) |
| World entry | `startup_snapshot.rs`, `startup_world.rs`, `startup_selfdata.rs`, `startup_catalogues.rs`, `startup_messages.rs`, `zone_transition.rs`, `gateway/startup_stream.rs`, `game_clock.rs` (the time-sync reply, so the sky runs) |
| Shared world | `players.rs` (sessions, interest grid, nearest-64 fanout, AddPc mount tail), `world.rs` (zone instances, membership, transfers, group-heal claims), `movement.rs` (pose validation), `terrain.rs`, `world_regions.rs` (region and place table with a position lookup), `client_areas.rs` (opcode 89 areas), `world_events.rs` and `world_events/schedule.rs` (event NPCs, timelines, wall-clock schedule and season gates), `world_notice.rs` (`realm broadcast`), `npc_ambient.rs` (idle and emote animations) |
| Gameplay core | `gameplay.rs` (hub: state, load, request and tick dispatch only), `gameplay/occupation.rs` (**the busy table**: one row per activity, one column per gate; add new activities here), `gameplay/content_reload.rs` (atomic developer `realm reload` transaction), `gameplay/jobs.rs` (job selection, trainer learning, starter outfits), `gameplay/profile_outfit.rs`, `gameplay/progression.rs`, `gameplay/live_events.rs` (admin-opened events such as the scavenger hunt), `gameplay/placement_watch.rs`, `game_store.rs` (save envelope), `game_rules.rs` (membership policy), `rewards.rs`, `gameplay/encounter_rewards.rs`. Inline test modules live beside them (`gameplay/w64_regional_quest_tests.rs`, `t013_objective_events.rs` and others); they are `#[cfg(test)]` children of `gameplay.rs` |
| Combat and enemies | `encounter.rs` (encounters, HP, kill credits, pickups, solo field path), `encounter/skills.rs` (job abilities, DoTs, buffs, mitigation, energy, crits), `encounter/shared.rs` (`SharedFieldState`: one enemy simulation per instance), `encounter/field.rs` and `field_content.rs` (field enemies and battle markers: loading, staged reloads, wander and chase AI, start screen), `encounter/field_rewards.rs`, `encounter/arena.rs` (one battle run's enemy simulation), `encounter/battle_drops.rs`, `encounter/combat_scale.rs` (tunable damage, health and tier numbers from `combat_scale.json`), `encounter/revive.rs`, `encounter/power_up_use.rs`, `encounter_invite.rs` (group battle invitation), `gameplay/field_battle.rs`, `gameplay/knockout.rs`, `gameplay/power_up_use.rs`, `power_ups.rs` (drop tables and participation roll), `abilities.rs` (36/10 casts, 36/12–13 definitions), `opening_abilities.rs` (job bars), `combat.rs`, `enemies.rs` (older static definitions, superseded by field enemy content) |
| Quests and story | `quest_engine.rs` (definition-driven quests), `quest_journal.rs` (47/12, 47/13, 49/15, 47/14 tracked-target pin), `quest_markers.rs` (35/10 and 35/11 markers over NPCs), `job_browser.rs` (the 147/1 Jobs-window list), `gameplay/quest_pickups.rs` (quest-only pickup nodes), `quests.rs`, `opening.rs`, `opening_journal.rs`, `opening_world.rs`, `profiles.rs` (jobs/classes) |
| NPCs and vendors | `gameplay/npc.rs` (the NPC router: 26/x, 45/7, 49/x, 165/x, 66/x, plus dialogue, explainer-page, merchant and quest-offer builders), `gameplay/npc_quests.rs` (catalogue, breadcrumbs, eligibility), `gameplay/npc_turnin.rs`, `npc_interaction.rs` (menus, quest offers, greeting and explainer data), `coin_shop.rs` (165/4 buy, 165/5 sell, 165/6 result, 165/10 coin merchant window), `station_cash.rs` (wallet, Marketplace catalogue, ride orders), `gameplay/sc_earnings.rs` (Station Cash earned from verified completions), `fotomat.rs` (156/2 portrait request) |
| Items | `inventory.rs` (state, equip with eligibility, `adopt_state` for trade commits), `inventory_v2_fixes.rs`, `item_categories.rs`, `item_bar.rs` (four-slot bar, potion), `gameplay/item_use.rs`, `gameplay/item_requests.rs`, `consumable_effects.rs` (timed per-player effect stack: treats, auras, transformations), `boombox.rs` (boombox catalogue and placed boxes), `style_cards.rs` with `gameplay/style_card_use.rs`, `mystery_chest.rs`, `mystery_chest_loot.rs` |
| Crafting and gathering | `recipes.rs`, `recipe_book.rs` (26/24 at a station), `cooking.rs`, `smelting.rs`, `forging.rs`, `mining.rs`, `harvesting.rs`, `gameplay/crafting.rs` (the crafting and gathering run machinery moved out of the hub), `gameplay/movie.rs` (the five-movie gate order, static refusals, 250 ms clock rule), `minigame.rs` (shared run lifecycle), `mining_lifecycle.rs`, `harvesting_lifecycle.rs`, `harvest_nodes.rs` (per-player ore and mushroom node actors; 26/8 starts the matching run; placements in `mining_nodes.json` and `harvest_nodes.json`) |
| Social | `chat.rs`, `social_chat.rs` (chat policy, quick chat and emotes, per-account action budget, persisted channel switches and bubble colour), `social_lists.rs` (friends, ignore, groups), `guilds.rs` (family 129), `matchmaking.rs` with `gameplay/matchmaking_route.rs` (list replies and the empty in-progress list), `trade.rs` (radial invite; 59/3 and 59/6–10 lifecycle; validated settlement), `player_radial.rs` (click menu, Inspect, Trade). The trade commit in `bin/osfr-gateway.rs` writes `trade-<session>-<ns>.trade-journal` beside character saves and recovers it before accepting logins. |
| Travel | `restoration/piece_077.rs` (warpstones: 56/57/90/122/58, included by `gameplay.rs`), `dev_teleport.rs`, `pathing.rs` (98/1 request to 98/2 terrain-sampled straight path; kinds 1 and 2), `gameplay/admin_move.rs`, `gateway/admin_move.rs` and `bin/osfr_gateway/admin_move.rs` (admin `realm move`) |
| Minigames and activities | `kart_*.rs` (nine files: entry, gateway, lobby, racing, service, pickups, weapons, rewards, progress; NPC gates and reward cooldown in `kart_gateway.rs` + `kart_gates.json`), `soccer.rs`, `soccer_entry.rs`, `soccer_match.rs`, `soccer_service.rs`, `derby.rs`, `derby_gateway.rs` with `gameplay/derby_session.rs` and `gameplay/derby_rows.rs` (which MiniGameData row an arena driver enters), `chess.rs` and `chess_service.rs`, `fishing.rs` + `fishing_service.rs` + `gameplay/fishing_scene.rs` (guide entry from `fishing_spots.json`; minigame 21 and opcode 138), `card_game.rs`, `card_service.rs`, `gameplay/card_launch.rs` (Card Duelist rows, wire codec, launch ticket), `activity_portal.rs` (167) |
| Housing, pets, mounts, progression | `housing.rs`, `housing_native.rs`, `housing_rating.rs`, `housing_service.rs`, `gameplay/housing.rs` (family 127 in the world session); `pets.rs`, `pet_follow.rs`, `gameplay/rides.rs` (ride and pet glue), `gateway/companions.rs`; `mounts.rs` (168 spawn/board/put-away, ride actor and rider attachment; fanout in `bin/osfr-gateway.rs`; ride catalogue and shop rows in `mounts.json`, `stores.json`, `vendors.json`); `progression_extras.rs` (achievements, titles, collections), `collection_catalogue.rs`, `collection_coins.rs`, `gameplay/collection_pickup.rs`, `gameplay/coin_authoring.rs` (`realm coin`) |
| Content loading | `content.rs` (content JSON schema and cross-checks), `world_text.rs`; item definitions are the union of the private package files `inventory.json` and `item-catalogue.json` (58,835 ids in the shipped packages; neither file is in this checkout, so that figure is carried, not recounted) |
| Developer tools | `studio.rs` (loopback HTTP), World Builder layer (`--world`), `realm` commands (by role on account-store servers) |
| Restoration pieces | `src/restoration/piece_NNN.rs` (92 files, `piece_008` to `piece_099`): recovered native codecs and rules, one per work item (`notes/WorkItems/NNN.md`). None is a `lib.rs` module. Most are compiled only by their own `tests/piece_NNN.rs` (a `#[path]` include); a few are also included by a host module: 025 (`housing_service.rs`), 046 and 048 (`item_bar.rs`), 052 (`world_events.rs`), 056 (`abilities.rs`), 074 (`progression_extras.rs`), 077 (`gameplay.rs`), 081 (`inventory.rs`), 093 (`social_chat.rs`), 096 (`startup_catalogues.rs`), 098 (`osfr-terrain`). |

## Content (`prototype/osfr-tools/data/`)

Content is JSON, loaded at startup and reloadable with `realm reload` where the loader supports it. Counts are from this tree. Rows carry provenance fields; numbers we had to invent are marked **(R)**. Large private inputs (the item catalogue, startup snapshot, captured templates, terrain grids) are packaged outside Git; `scripts/bundle/` builds the playtest bundle.

| Area | Files in `data/content/` |
|---|---|
| NPCs and population | `npcs/*.json` (17 zone files, 1,209 regional NPC rows; 177 have a recovered greeting line, 194 have the vendor role), `npc_spawns_extra.json` (2,770 extra placements), `npc_spawns_retired.json` (632 community-pruned legacy spawns kept for the record), `npcs_unresolved.json`, `npc_ambient.json`, `zones.json` (17 region labels), `regions.json` (10 regions, 19 places, client name ids), `client_areas.json` |
| Quests and jobs | `quests/*.json` (23 files, 678 records, 93 not marked `enabled: false`), `breadcrumb_quests.json`, `quest_nodes.json`, `quest_markers.json`, `jobs.json` (17), `job_browser.json`, `job_trainers.json`, `staff_jobs.json` |
| Combat and enemies | `combat_scale.json`, `job_abilities.json` (six jobs), `opening-abilities.json`, `behaviours.json`, `field_enemies.json` (13 spawn groups, 49 enemy rows), `field_kill_rewards.json`, `field_battles.json` (11 battles, all in arena zone 9600), `encounters.json` with `encounter-template.bin`, `power_ups.json`, `enemies.json` (107 older definitions) and `enemies_unresolved.json`, `dungeons/` (75 research records) and `dungeons_unresolved.json` |
| Shops and economy | `vendors.json` (100 vendors with stock), `vendors_unresolved.json` (192 unresolved definitions), `stores.json` (17,477 shop and Marketplace rows), `sc_rewards.json` (Station Cash earn amounts), `consumables.json` (94), `recipes.json` (32) and `recipes_unresolved.json`, `crafting_stations.json` |
| Gathering | `mining.json`, `mining_nodes.json`, `harvesting.json`, `harvest_nodes.json` |
| Collections and progression | `collections.json`, `collection_catalogue.json` (253 collections), `collection_coins.json` (121 placed coins), `collection_nodes.json` (12 node types, 34 pools, 847 spawn points; data only), `achievements.json`, `titles.json` |
| Minigames | `kart_entry.json`, `kart_lobby.json`, `kart_gates.json`, `derby_entry.json`, `derby_gates.json`, `derby_rows.json` (extra hosted rows), `derby_minigames.json` (their TYPE-11 proof), `soccer.json`, `chess.json`, `fishing.json`, `fishing_spots.json` (6 spots), `card_game.json` |
| Housing, pets, rides, music | `houses.json` (31 lots, 23 enabled), `housing_fixtures.json` (1,186 fixtures), `pets.json` (113 species), `mounts.json` (89 rides), `boomboxes.json` (15) |
| World and moderation | `world_events.json` (3 events: two concerts and the Halloween dance-off), `game_clock.json`, `moderation.json` |

`data/reference/` holds three reference tables (`attacks.json`, `item_abilities.json`, `weapons.json`) built from the wiki harvest and checked against the client. `studio/` is the Studio web page the gateway serves with `--dev-tools`.

## The spine (as of 2026-10-27)

The four hubs (`gameplay.rs`, `bin/osfr-gateway.rs`, `encounter.rs`, `gateway.rs`) are dispatch only. A feature lives in its owning module and the hub gets the few lines that call it; the autopilot refuses a builder diff that grows a hub by more than 40 net lines. Charter: the owner's tables-and-registry vision (values in content tables with atomic reload; features slot in; one feature's failure never takes down another; idle systems cost nothing). The deletion phase (D001–D010, 2026-10-18 to 10-20) is merged; reports are in `notes/Autopilot/`.

- **Busy rules:** `gameplay/occupation.rs`. A new activity adds an `Occupation` variant; the build fails until it decides every gate (start, warp, ride, pets, location) and its detection.
- **Find the owner first:** the code graph (codebase-memory, indexed on the integration worktree and re-indexed after every integration) answers "who owns X" and "who calls X"; read the owning module and the hub's call site, not the whole hub.
- **Proof for moves:** a pure move keeps `cargo test -- --list` identical; the integrator also refuses any change that loses a named passing test (`lab/autopilot/protected-pass.txt`, about 5,900 names at this tip: counted from the private file, not re-run for this refresh).
- **Isolation boundary:** `contain` in `bin/osfr-gateway.rs` turns a panic in one session's packet handling, tick or end-of-session save into an `internal_error` disconnect for that player alone; a faulted shared field freezes (`faulted_fields`). Test: `osfr_gateway/isolation.rs`.
- **Spine ratchet:** `tests/spine_rules.rs` with ceilings in `tests/spine_rules.ceilings` (lead-only). It covers hub sizes, plain `fs::write`, hex outside `trace.rs`, and hand-read busy state outside `occupation.rs`. At this tip the hubs measure 5,143 (`gameplay.rs`), 4,631 (`bin/osfr-gateway.rs`), 2,976 (`encounter.rs`) and 2,125 (`gateway.rs`) lines. The ceilings only go down; the integrator lowers them and appends `lab/autopilot/logs/drift.log` after each green batch.
- **Dormancy:** a shared field nobody is in settles once, then sleeps (`dormant_fields` in `Server::tick_shared_fields`; test `osfr_gateway/dormant_fields.rs`). For the next slices, see [[Build Playbook]] §4.

## Tests and gates

- **Full suite:** `cargo test --manifest-path prototype/osfr-tools/Cargo.toml --no-fail-fast` (maintainers: `scripts/cargo.ps1 test ...`). The README says about ten minutes; not re-timed for this refresh. Lead reviews run it in a clean worktree holding only the change under review, with a shared build dir under `lab/build/`. The tree has 464 test files in `tests/`, inline `#[cfg(test)]` modules in `src/`, and counted 4,832 `#[test]` attributes (a static count; macro-generated tests are not in it) with 153 `#[ignore]` attributes. The last full-suite figure written in the Ledger is from 2026-10-17 (2,747 passed at `0e5d122`). **This refresh did not run the suite, so the pass count at this tip is unverified.**
- **Shared fixtures:** `tests/common/*.rs` (NPC greeting harness, vendor and shop fixtures, field-battle and field-enemy fixtures, packet readers), `tests/fixtures/`. Tests that need private lab data are `#[ignore]`d, usually naming the environment variable they read.
- **LAN gate (lead only):** `cargo test --test login_broker -- --ignored` for any change to login, login_auth, net_policy or osfr-login. It binds this PC's LAN address, which prompts Windows Firewall, so ordinary suites skip it.
- **Coverage gate:** `tests/app_fuzz.rs`. Every newly routed family/subtype in `gateway.rs::application_handler` needs a seed, or the coverage test fails.
- **Spine ratchet:** `tests/spine_rules.rs` (see above).
- **Content gate:** `scripts/content_check.py --strict` (0 errors expected; its own regression is `scripts/test_content_check.py`).
- **Script gates (Python, stdlib):** `tests/test_*.py` (nine files) and the checks beside their scripts (`scripts/test_prepare_opening_base.py`, `scripts/launcher/test_*.py`, `scripts/native_map/test_*.py`). Run each from the repository root; each file's header says how.
- **Formatting:** `rustfmt --edition 2024 --check` on touched files. Preserve each file's CRLF or LF style.
- **Native gate (lead only):** a change the client can see is checked in the real client with `scripts/run-local-baseline.py` (launch against a disposable copy), `scripts/drive-local-game.py` (input) and `scripts/native_hook.py` (Frida hook points). Needs the private client; results go to `lab/autopilot/native-notes/` and then `notes/Reviews/`.
- **Disk:** each full-suite build dir is about 28 GB. Build with `CARGO_PROFILE_DEV_DEBUG=0` and delete `debug/` when done.

## Scripts (`scripts/`)

| Area | Files |
|---|---|
| Native function map | `native_map/` (30 files; `nmq.py` query, `ingest.py`, `recheck.py`, `ida_check.py`, `build_guides.py`, `dispatch.py` and the mapper and reviewer prompts) and the Ghidra export scripts `ExportNative.java`, `ExportNativeMap.java`, `InspectNative.java`, `DefineAndExportNew.java`, `DefineGapFunctions.java`, `FixNoReturn.java` |
| Content checks and audits | `content_check.py`, `audit-quest-coverage.py`, `audit-minigame-coverage.py`, `quest_audit.py`, `locale_id_map.py`, `wiki-research.py` (with `wiki-requirements.txt`), `world_objects.py` |
| Content builders (write `data/content/`) | `build_boomboxes.py`, `build_collection_coins.py`, `build_collection_nodes.py`, `build_consumables.py`, `build_housing_fixtures.py`, `build_item_catalogue.py`, `build_sell_prices.py`, `build_weapon_reference.py`, `build_wiki_prices.py`, `pets_import.py`, `quest_talk_objectives.py`, `quest_text_backfill.py`, `extract_job_outfits.py`, `build_world_locale.py` |
| Private package preparation | `prepare-*.py` (opening, cooking, smelting, forging, gameplay inventory, combat and quests, item categories, journal text, gateway reference), `migrate-*-campaign.py`, `check-*.py` migration and asset checkers, `cooking_campaign_proof.py`, `forging_campaign_proof.py`, `anvil_placement.py` |
| Bundles and playtest | `bundle/` (`make_bundle.py`, `check_bundle.py`, `client_overlay.py`, `item_content.py`, `bundle_common.py`), `stress/run.py` (bots, `--broker-login`), `run-local-baseline.py`, `drive-local-game.py`, `bot/*.bot` (three job attack scripts), `fixtures/starter_items.json` |
| Debug harness | `native_hook.py`, `live_capture.py`, `live_log.py`, `lua_probe_overlay.py`, `journal_text_overlay.py`, `verify-soe-capture.ps1`, `collect-legacy-oracle.py` |
| Assets | `build_asset_graph.py`, `build_asset_index.py`, `asset_delivery.py`, `build_terrain.py`, `image_search.py`, `image_sheets.py` |
| LAN friend tools | `launcher/` (`friend_launcher.py`, `gamepad_bridge.py`, `gamepad.json` and their tests). The real launcher is the separate repository above. |

## Lab (`lab/`, private, not in Git)

| Path | Contents |
|---|---|
| `originals/`, `working/` | Hash-verified original archives and working copies, including the client locale and the IDA database of the 2014 client (open it through the IDA MCP with `run_auto_analysis=false`). |
| `native-map/` | Native function map (names, strings, callees, rules). Query it with `scripts/native_map/nmq.py`. |
| `references/` | Pinned community repos (Legacy, Sanctuary, locale tools; behaviour reference only, no AGPL code copy) and the gateway reference templates. |
| `content/`, `reports/`, `terrain/`, `bundles/` | Private content packages, catalogues, worker reports, ground-height grids, playtest bundles. |
| `captures/` | Packet captures. Some are of the community Legacy server, not SOE, so say which. |
| `autopilot/` | The build pipeline: `queue/`, `merged/`, `reviews/`, `native-notes/` (drafts of real-client run notes), `logs/`, the worker rules. |
| `runs/` | Worker reports and real-client run folders (`runs/claude/subagent-rules.md` holds the worker rules; Codex-era runs are under `runs/luna/`). |
| `build/` | Cargo target dirs (large; clean `debug/` after use). |

## Notes (`notes/`)

| Folder or file | Use |
|---|---|
| `Design/` | Build Ledger, Build Plan, Build Playbook, Security Model, Public Transport, Shared World, Station Cash, Playtest Bundle, Playtest Host Runbook, Rust Launcher (design history; the launcher now lives in its own repository), Studio, World Builder, Field Enemies, Battle Template, worker contract |
| `Autopilot/` | A report for most integrated tasks: `T` (feature), `F` (review fix), `L` (lead patch; a few have none), `D` (deletion step); 570 files counted. Newest evidence for any module is usually here. |
| `Research/` | Topic research (protocols, content, Unrouted Client Requests survey, Dungeon Instances, Card Game, Zones and Regions) |
| `Research/Pieces/`, `WorkItems/` | Per-piece restoration notes |
| `Native Map/` | Shards, classes, IDA cross-checks (the coverage figures on its front page date from 2026-10-12) |
| `Build Guides/` | One guide per feature with every mapped function |
| `Reviews/` | Native acceptance runs and independent reviews (newest first by name). The newest `Native *.md` there dates from before the 2026-10-20 runs. |
| `Journal/`, `Templates/` | Setup journal and note templates |
| `Start Here.md`, `Tasks.md`, `Current Progress.md`, `MMO Work Queue.md`, `Handoff *.md` | Older orientation notes (2026-10-08 to 10-17). They predate most of the code; trust the Ledger and the code over them. |
