---
updated: 2026-10-17
---

# Project Map

Where everything lives, for agents. It's a map, not a status report. Status lives in [[Design/Build Ledger]] (verified state per system) and [[Design/Build Plan]] (what's next and the update log). Before claiming something is missing, check the code and the newest `Reviews/Native *.md` ([[Start Here]]).

Refreshed by the lead after each integration batch. If you add, rename or retire a module, update its row here in the same change.

## Top level

| Path | What it is |
|---|---|
| `project/` | Git repo (private, `FadedSociety/free-realms-expanded`). Commit only your own files; never commit `lab/`, saves or keys. |
| `project/prototype/osfr-tools/` | The Rust server crate: library (`src/`), binaries (`src/bin/`), 92 restoration pieces (`src/restoration/`), about 250 test files (`tests/`). |
| `project/launcher/` | Standalone Rust launcher (egui, controller support, patching, key-pair login, join-a-realm). Build with `launcher/build.ps1` (target `x86_64-pc-windows-gnullvm`; `getrandom` stays 0.2). |
| `project/scripts/` | Python and PowerShell tooling (below). `scripts/cargo.ps1` selects the private Rust toolchain in `lab/tools`. |
| `project/notes/` | This Obsidian vault. |
| `lab/` | Everything not in Git: original client, working copies, captures, references, IDA database, build dirs, worker runs, reports. |

## Server binaries (`src/bin/`)

| Binary | Role |
|---|---|
| `osfr-gateway` | The game server. Owns sessions, SOE transport, application dispatch, the shared world, saves and security limits. `Server::step` drains datagrams; world and session ticks run at 20 Hz. Studio is loopback-only and needs `--dev-tools` plus an admin. |
| `osfr-login` | Native login and character select, plus the AUTH2 key-pair broker (`--auth-bind`; a private/VPN address with `--allow-lan`). |
| `osfr-account` | Host-only account provisioning: create accounts, add launcher public keys, set roles. |
| `osfr-bot` | Headless bot client (direct or broker login) for tests and stress. |
| `osfr-terrain` | Builds ground-height grids from client terrain chunks. |
| `soe-probe` | First transport-slice probe (historical). |

## Server library (`src/`) by system

The request path runs packet → `gateway.rs` (per-character application gateway; `application_handler` labels each family/subtype) → `gameplay.rs` (in-world dispatch; stages changes on a clone, so rewards commit together) → the system modules below. `osfr-gateway.rs` handles cross-player routing: chat, social, shared fields, inspect.

| System | Modules |
|---|---|
| Transport and session | `soe.rs`, `soe_send.rs` (SOE framing and reliable channel), `session.rs`, `net_policy.rs` (bind and peer rules, token buckets), `trace.rs` (library module; the one packet-redaction rule every tracer uses) |
| Login, accounts and security | `login.rs`, `login_auth.rs` (AUTH2 key-pair), `account.rs`, `account_store.rs` (roles, public keys), `character_store.rs`, `identity.rs`, `security_log.rs` |
| World entry | `startup_snapshot.rs`, `startup_world.rs`, `startup_selfdata.rs`, `startup_catalogues.rs`, `startup_messages.rs`, `zone_transition.rs` |
| Shared world | `players.rs` (sessions, interest grid, nearest-64 fanout, AddPc mount tail), `world.rs` (zone instances, membership, transfers, group-heal claims), `movement.rs` (pose validation), `terrain.rs` |
| Gameplay core | `gameplay.rs` (hub: state, load, request and tick dispatch only), `gameplay/occupation.rs` (**the busy table**: one row per activity, one column per gate; add new activities here), `game_store.rs` (save envelope), `game_rules.rs` (membership policy), `rewards.rs`. Inline tests live in `gameplay/<module>.rs` with unchanged names |
| Combat and enemies | `encounter.rs` (encounters, HP, revive, kill credits, pickups, solo field path), `encounter/skills.rs` (job abilities, DoTs, buffs, mitigation, energy, crits), `encounter/shared.rs` (`SharedFieldState`: one enemy simulation per instance), `encounter/field.rs` (field enemies and battle markers: loading, reloads, wander/chase AI, battle start screen), `enemies.rs`, `combat.rs`, `abilities.rs` (36/10 casts, 36/12–13 definitions), `opening_abilities.rs` (job bars), `power_ups.rs` (drop tables and participation roll) |
| Quests and story | `quest_engine.rs` (definition-driven quests), `quest_journal.rs` (47/12, 47/13, 49/15, 47/14 tracked-target pin), `quests.rs`, `opening.rs`, `opening_journal.rs`, `opening_world.rs`, `profiles.rs` (jobs/classes) |
| NPCs and vendors | `gameplay/npc.rs` (the NPC router: 26/x, 45/7, 49/x, 165/x, 66/x, plus dialogue/merchant/quest-offer builders), `gameplay/npc_quests.rs` (quest glue: catalogue, breadcrumbs, eligibility), `npc_interaction.rs` (menus, quest offers, merchant 66/42), `coin_shop.rs` (165/4 buy, 165/5 sell, 165/6 result) |
| Items | `inventory.rs` (state, equip with eligibility, `adopt_state` for trade commits), `inventory_v2_fixes.rs`, `item_categories.rs`, `item_bar.rs` (four-slot bar, potion), `mystery_chest.rs`, `mystery_chest_loot.rs` |
| Crafting and gathering | `recipes.rs`, `recipe_book.rs` (26/24 at a station), `cooking.rs`, `smelting.rs`, `forging.rs`, `mining.rs`, `harvesting.rs`, `minigame.rs` (shared run lifecycle), `mining_lifecycle.rs`, `harvesting_lifecycle.rs`, `harvest_nodes.rs` (W171 per-player ore and mushroom node actors; 26/8 starts the matching Type8 run; placements in `mining_nodes.json` and `harvest_nodes.json`) |
| Social | `chat.rs`, `social_chat.rs` (chat policy, quick chat and emotes, per-account action budget, W173 persisted channel switches and bubble colour), `social_lists.rs` (friends, ignore, groups), `guilds.rs`, `trade.rs` (radial invite; 59/3 and 59/6–10 lifecycle; validated settlement), `player_radial.rs` (click menu, Inspect, Trade). The commit in `bin/osfr-gateway.rs` writes `trade-<session>-<ns>.trade-journal` beside character saves and recovers it before accepting logins. |
| Travel | `restoration/piece_077.rs` (warpstones: 56/57/90/122/58), `dev_teleport.rs`, `pathing.rs` (98/1 Take Me There request to 98/2 terrain-sampled path) |
| Minigames and activities | `kart_*.rs` (8 files: entry, lobby, racing, service, pickups, weapons, rewards, progress; NPC gates and reward cooldown in `kart_gateway.rs` + `kart_gates.json`), `soccer*.rs`, `derby.rs`, `chess.rs`, `fishing.rs` + `fishing_service.rs` (guide entry from `fishing_spots.json`; minigame 21 and opcode 138), `matchmaking.rs`, `activity_portal.rs` (167) |
| Housing, pets, mounts, progression | `housing*.rs`, `pets.rs`, `mounts.rs` (168 spawn/board/put-away, ride actor and rider attachment; fanout in `bin/osfr-gateway.rs`; ride catalogue and shop rows in `mounts.json`, `stores.json`, `vendors.json`), `progression_extras.rs` |
| Content loading | `content.rs` (content JSON schema and cross-checks), `world_text.rs`; item definitions are the union of `inventory.json` and `item-catalogue.json` (58,835 ids in the shipped packages) |
| Developer tools | `studio.rs` (loopback HTTP), World Builder layer (`--world`), `realm` commands (by role on account-store servers) |
| Restoration pieces | `src/restoration/piece_NNN.rs`: recovered native codecs and rules, one per work item (`notes/WorkItems/NNN.md`). Some are included by a host module via `#[path]` instead of `lib.rs`. |

## The spine (2026-10-18)

The four hubs (`gameplay.rs`, `bin/osfr-gateway.rs`, `encounter.rs`, `gateway.rs`) are dispatch only. A feature lives in its owning module and the hub gets the few lines that call it; the autopilot refuses a builder diff that grows a hub by more than 40 net lines. Charter: the owner's tables-and-registry vision (values in content tables with atomic reload; features slot in; one feature's failure never takes down another; idle systems cost nothing).

- **Busy rules:** `gameplay/occupation.rs`. A new activity adds an `Occupation` variant; the build fails until it decides every gate (start, warp, ride, pets, location) and its detection.
- **Find the owner first:** the code graph (codebase-memory, project `C-Users-the owner-Documents-Projects-CartoonMMO-autopilot-int`, re-indexed after every integration) answers "who owns X" and "who calls X"; read the owning module and the hub's call site, not the whole hub.
- **Proof for moves:** a pure move keeps `cargo test -- --list` identical; the integrator also refuses any change that loses a named passing test (`lab/autopilot/protected-pass.txt`).

## Tests and gates

- **Full suite:** `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --no-fail-fast`. About 10 minutes, about 2,400 tests. Lead reviews run it in a clean worktree (`CartoonMMO/review`) holding only the change under review, with the shared build dir `lab/build/review`.
- **Coverage gate:** `tests/app_fuzz.rs`. Every newly routed family/subtype in `gateway.rs::application_handler` needs a seed, or the coverage test fails.
- **Other gates:** `scripts/content_check.py --strict` (0 errors expected), and `rustfmt --edition 2024 --check` on touched files. Preserve each file's CRLF/LF style.
- **Disk:** each full-suite build dir is about 28 GB. Build with `CARGO_PROFILE_DEV_DEBUG=0` and delete `debug/` when done.

## Scripts (`project/scripts/`)

| Area | Files |
|---|---|
| Native function map | `native_map/` (`nmq.py` query, `validate.py`, `recheck.py`, `xreview.py`, `ida_check.py`, `build_guides.py`, `dispatch.py`) |
| Content | `content_check.py`, `build_item_catalogue.py`, `quest_audit.py`, `prepare-*.py`, `wiki-research.py`, `world_objects.py` |
| Bundles and playtest | `bundle/` (including `item_content.py`: loader-rule item, ride and consumer-coverage gate; refuses a missing or broken `item-catalogue.json`), `stress/run.py` (bots, `--broker-login`), `run-local-baseline.py`, `drive-local-game.py` |
| Assets | `build_asset_graph.py`, `build_asset_index.py`, `asset_delivery.py`, `build_terrain.py`, `locale_id_map.py` |
| Launcher | `launcher/` |

## Lab (`lab/`)

| Path | Contents |
|---|---|
| `originals/`, `working/` | Hash-verified original archives and working copies. Client locale: `working/OSFR.Client/OSFR Client/locale/en_us_data.dat`. |
| `working/ida/FreeRealms.exe.i64` | IDA database of the 2014 client. Open it through the IDA MCP with `run_auto_analysis=false`. |
| `native-map/run/map.sqlite` | Native function map (names, strings, callees, rules). Query it with `nmq.py`. |
| `references/` | Pinned community repos (Legacy, Sanctuary, locale tools; behaviour reference only, no AGPL code copy) and `rust-gateway-reference`. |
| `content/`, `reports/` | Private content packages, catalogues, worker reports. |
| `captures/` | Packet captures. Some are of the community Legacy server, not SOE, so say which. |
| `runs/claude/WNNN/` | Claude worker reports. `runs/claude/subagent-rules.md` holds the worker rules. Codex-era runs are under `runs/luna/`. |
| `build/` | Cargo target dirs (large; clean `debug/` after use). |

## Notes (`project/notes/`)

| Folder | Use |
|---|---|
| `Design/` | Build Ledger, Build Plan, Security Model, Shared World, Public Transport, Playtest runbook, Studio, World Builder, worker contract |
| `Research/` | Topic research (protocols, content, Unrouted Client Requests survey and briefs, Owner Field Notes) |
| `Research/Pieces/`, `WorkItems/` | Per-piece restoration notes |
| `Native Map/` | Shards, classes, IDA cross-checks |
| `Build Guides/` | One guide per feature with every mapped function |
| `Reviews/` | Native acceptance runs and independent reviews (newest first by name) |
