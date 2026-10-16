---
updated: 2026-10-16
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
| Transport and session | `soe.rs`, `soe_send.rs` (SOE framing and reliable channel), `session.rs`, `net_policy.rs` (bind and peer rules, token buckets), `trace.rs` |
| Login, accounts and security | `login.rs`, `login_auth.rs` (AUTH2 key-pair), `account.rs`, `account_store.rs` (roles, public keys), `character_store.rs`, `identity.rs`, `security_log.rs` |
| World entry | `startup_snapshot.rs`, `startup_world.rs`, `startup_selfdata.rs`, `startup_catalogues.rs`, `startup_messages.rs`, `zone_transition.rs` |
| Shared world | `players.rs` (sessions, interest grid, nearest-64 fanout), `world.rs` (zone instances, membership, transfers), `movement.rs` (pose validation), `terrain.rs` |
| Gameplay core | `gameplay.rs`, `game_store.rs` (save envelope), `game_rules.rs` (membership policy), `rewards.rs` |
| Combat and enemies | `encounter.rs` (encounters, shared field fights, HP, revive), `enemies.rs`, `combat.rs`, `abilities.rs` (36/10 casts, 36/12–13 definitions), `opening_abilities.rs` (job bars) |
| Quests and story | `quest_engine.rs` (definition-driven quests), `quest_journal.rs` (47/12, 47/13, 49/15), `quests.rs`, `opening.rs`, `opening_journal.rs`, `opening_world.rs`, `profiles.rs` (jobs/classes) |
| NPCs and vendors | `npc_interaction.rs` (menus, quest offers, merchant 66/42), `coin_shop.rs` (165/4 buy, 165/5 sell, 165/6 result) |
| Items | `inventory.rs` (state, equip with eligibility), `inventory_v2_fixes.rs`, `item_categories.rs`, `item_bar.rs` (four-slot bar, potion), `mystery_chest.rs`, `mystery_chest_loot.rs` |
| Crafting and gathering | `recipes.rs`, `recipe_book.rs` (26/24 at a station), `cooking.rs`, `smelting.rs`, `forging.rs`, `mining.rs`, `harvesting.rs`, `minigame.rs` (shared run lifecycle), `mining_lifecycle.rs`, `harvesting_lifecycle.rs`, `harvest_nodes.rs` |
| Social | `chat.rs`, `social_chat.rs` (chat policy, quick chat and emotes, per-account action budget), `social_lists.rs` (friends, ignore, groups), `guilds.rs`, `trade.rs`, `player_radial.rs` (click menu, Inspect) |
| Travel | `restoration/piece_077.rs` (warpstones: 56/57/90/122/58), `dev_teleport.rs` |
| Minigames and activities | `kart_*.rs` (8 files: entry, lobby, racing, service, pickups, weapons, rewards, progress), `soccer*.rs`, `derby.rs`, `chess.rs`, `fishing*.rs`, `matchmaking.rs`, `activity_portal.rs` (167) |
| Housing, pets, mounts, progression | `housing*.rs`, `pets.rs`, `mounts.rs` (not routed yet), `progression_extras.rs` |
| Content loading | `content.rs` (content JSON schema and cross-checks), `world_text.rs` |
| Developer tools | `studio.rs` (loopback HTTP), World Builder layer (`--world`), `realm` commands (by role on account-store servers) |
| Restoration pieces | `src/restoration/piece_NNN.rs`: recovered native codecs and rules, one per work item (`notes/WorkItems/NNN.md`). Some are included by a host module via `#[path]` instead of `lib.rs`. |

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
| Bundles and playtest | `bundle/`, `stress/run.py` (bots, `--broker-login`), `run-local-baseline.py`, `drive-local-game.py` |
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
