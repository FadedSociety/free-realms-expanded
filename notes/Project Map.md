# Project Map

Generated from the code at `5a07ba04` by `scripts/project_map.py`; rerun it after a merge instead of editing by hand.
System status (what works in the client) is not here: it lives in the per-game checklist sheets and native runs.

Columns: module, lines, used by the gateway/gameplay hub/a binary, test files naming it, purpose (its first doc line).

## src (crate root)

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `abilities.rs` | 991 | yes | 39 | Small, data-driven ability resolver. |
| `account.rs` | 528 | yes | 40 | Account ownership and pre-game character request handlers. |
| `account_store.rs` | 1036 | yes | 80 | Versioned account/character JSON storage. |
| `activity_portal.rs` | 892 | yes | 12 | W150: the activity portal list (167/1/1), its refresh (167/1/4) and the Play choice (167/1/2). |
| `area_banner.rs` | 120 |  | 1 | T900: the area-enter banner. |
| `base_ui.rs` | 76 |  | 0 | L952: BaseUi (opcode 47) packets the server sends to the client's UI. |
| `boombox.rs` | 3279 | yes | 8 | The boombox catalogue (`boomboxes.json`, T459), one row per client boombox item, and the boxes pressed from the item bar (T461): a [`Boomboxes`] set of live box |
| `bundle_budget.rs` | 61 | yes | 0 | (R) Outbound application-reply budget for one client message (F179). |
| `card_game.rs` | 560 | yes | 8 | T504: the card-game activity rows (Trading Card Game, category 11, and Treasure War, category 26) the activity portal publishes, loaded from `data/content/card_ |
| `card_service.rs` | 752 |  | 2 | T763: the Card Duelist (TCG) service wire, decoded from `tcg/FreeRealmsTCG.dll` (lab/content/tcg/analysis/lead-login-decode-2026-10-26.md): the compact integer  |
| `character_store.rs` | 224 | yes | 153 | One local character snapshot. |
| `chat.rs` | 189 | yes | 129 | The original client's chat request and system-message wire shapes. |
| `checkers.rs` | 879 | yes | 1 | L104: checkers rules and movie text for client minigame type 7 (MiniGameData 21, the live Games window row 514), played against the server. |
| `chess.rs` | 2267 |  | 13 | Chess table service for client minigame type 6. |
| `chess_engine.rs` | 368 |  | 0 | L105: an optional UCI chess engine (the official, unmodified Stockfish binary) for the computer side of a chess table. |
| `chess_service.rs` | 744 | yes | 4 | Chess routing through the shared minigame lifecycle. |
| `client_areas.rs` | 325 | yes | 8 | T457: opcode 89 `PacketSetClientArea`, the server's switch for the client areas that FabledRealmsAreas.xml ships with `AreaEnabled="0"`: always-on areas, month/ |
| `coin_shop.rs` | 1556 | yes | 16 | Coin Shop purchases and the Station Cash Marketplace handshake. |
| `collection_catalogue.rs` | 407 |  | 7 | Collection catalogue taken from our own login capture (T447). |
| `collection_coins.rs` | 478 |  | 6 | Which collection entry each placed exploration coin ticks (T449) and the live authoring overlay (T525). |
| `combat.rs` | 440 | yes | 77 | A deliberately small, original sandbox encounter using verified NPC framing. |
| `consumable_effects.rs` | 517 | yes | 9 | T473: the per-player timed effect stack behind treats, cakes and transformations. |
| `content.rs` | 1463 | yes | 407 | Versioned offline content records. |
| `cooking.rs` | 1156 | yes | 60 | Original Cooking GFx protocol with reconstructed six-phase tuning. |
| `daily_wheel.rs` | 253 | yes | 1 | T560: the daily rewards wheel ("Spin For The Win!"), part 1: offer the spin and open the movie. |
| `derby.rs` | 2396 |  | 25 | Native Demo Derby packets and a bounded server-authoritative bout candidate. |
| `derby_gateway.rs` | 987 | yes | 9 | Optional ordinary-entry binding for the native demolition derby. |
| `dev_teleport.rs` | 753 | yes | 3 | Developer-only `realm goto` relocation to a named actor in the world zone. |
| `encounter.rs` | 2926 | yes | 101 | Local opening encounters over community-documented native packets. |
| `encounter_invite.rs` | 294 |  | 4 | T440 (Shared World slice 5): the ordinary group-encounter invitation. |
| `enemies.rs` | 823 |  | 73 | Instance-scoped enemies. |
| `fishing.rs` | 911 |  | 21 | Free Realms fishing (`BaseFishingPacket`, opcode 138): wire codec and session tracker. |
| `fishing_service.rs` | 2031 | yes | 8 | Server-authoritative fishing service for native minigame type 21. |
| `forging.rs` | 1501 | yes | 24 | Forging139 drives the original match-three movie with a server-owned board. |
| `fotomat.rs` | 247 | yes | 2 | Fotomat portraits: the client's 156/2 `PacketPortraitDataRequest` and the 156/3 `PacketPlayerImageData` reply, plus the quiet 115 portrait CRC. |
| `game_clock.rs` | 209 | yes | 2 | T469: the game clock in the opcode 52 time-sync reply, so the client's sky runs. |
| `game_rules.rs` | 315 | yes | 1 | Account membership and local game-rule policy. |
| `game_store.rs` | 192 | yes | 65 | Version-seven character, gameplay, housing, pet and recipe snapshots, committed together. |
| `gameplay.rs` | 5018 | yes | 262 | Local gameplay integration. |
| `gateway.rs` | 2127 | yes | 224 | Per-character application gateway backed by a private, verified reference. |
| `gm_panel.rs` | 238 |  | 1 | L072: the referee (GM) panel's chat protocol, for the client mod from Sanctuary PR #163. |
| `guilds.rs` | 2261 | yes | 11 | World-server guild state and family-129 request decoding. |
| `harvest_nodes.rs` | 1026 | yes | 22 | Harvest-node content imported from the pinned Legacy data tables and the community collection tables (T523), and (W171) the ordinary way into mining and harvest |
| `harvesting.rs` | 1364 | yes | 46 | Harvesting (original MatchThree "Farming" movie, MiniGameData type 8). |
| `harvesting_lifecycle.rs` | 524 |  | 0 | Harvesting adapter for the shared minigame run lifecycle. |
| `hash.rs` | 37 |  | 45 | One SplitMix64 mixer for every seeded stream. |
| `housing.rs` | 1569 | yes | 27 | Housing / property / furniture placement state for one restored server. |
| `housing_native.rs` | 1304 |  | 6 | Native housing placement codecs. |
| `housing_rating.rs` | 227 |  | 1 | Housing rating directory stub (T517): the client's 174/16 `CandidateInfoRequest` probe on every house entry, answered with an empty 174/17 `CandidateInfoReply`. |
| `housing_service.rs` | 2715 | yes | 4 | Housing integration service: the callable seam between the recovered native housing requests, the restored `housing` state machine and the live character `Inven |
| `identity.rs` | 565 | yes | 77 | One native-created local character. |
| `inventory.rs` | 1874 | yes | 206 | Typed local inventory state and evidenced inventory packet shapes. |
| `inventory_item_table.rs` | 150 |  | 0 | T877: the release item table, `item-table.json` beside `inventory.json`. |
| `inventory_v2_fixes.rs` | 126 |  | 5 | Narrow inventory wire fixes and a fail-closed startup catalogue encoder. |
| `item_bar.rs` | 1121 | yes | 10 | W146/T510: the four-slot item bar and the data-driven consumable rows (potions, food auras, transformation treats and size effects). |
| `item_categories.rs` | 135 | yes | 2 | Item category reference data, using the community 44/2 wire description. |
| `job_browser.rs` | 161 |  | 4 | The Jobs window's Become-a list: the 147/1 `ActivityProfileList` the client fills its `ActivityFinderDataSource` from, so `JobBrowser:StartJobQuest` (`ScriptsBa |
| `jump_relay.rs` | 254 | yes | 0 | Decode and relay the native client's separate jump update (opcode 164). |
| `kart_entry.rs` | 769 |  | 13 | Native kart race-entry handler: the tunneled inner-opcode-33 register exchange. |
| `kart_gateway.rs` | 910 | yes | 9 | Optional race entry binding. |
| `kart_lobby_native.rs` | 601 | yes | 3 | Native kart lobby definition codec: opcode 102 sub 2. |
| `kart_pickups.rs` | 764 |  | 3 | Kart pickup set: native `33/15` create, `33/16` destroy and `33/18` grant, plus the bounded positional authority a caller runs *after* its own claim validation. |
| `kart_progress_native.rs` | 298 |  | 5 | Native kart progress payload codecs for the vehicle-race family (inner opcode 33). |
| `kart_racing.rs` | 734 |  | 9 | Kart Racing (original lobby game type 1) bounded session contract. |
| `kart_rewards.rs` | 353 |  | 2 | Kart reward registration and the one-shot payout seam for one finished race. |
| `kart_rows.rs` | 142 |  | 4 | L930: the MiniGameData rows the kart race serves (`kart_rows.json`, beside `kart_gates.json`). |
| `kart_service.rs` | 2675 | yes | 5 | Kart race service: server-side lifecycle, control, gate validation and finish. |
| `kart_weapons.rs` | 1376 |  | 1 | Kart weapon packet codecs and a bounded local visual lifecycle. |
| `le.rs` | 42 | yes | 1 | Little-endian field reads that cannot panic (L109, hygiene pass). |
| `lib.rs` | 148 |  | 77 |  |
| `login.rs` | 807 | yes | 135 | Minimal loopback external-login protocol (LoginUdp_6). |
| `login_auth.rs` | 450 | yes | 6 | (R) Launcher key-pair authentication, not recovered Free Realms protocol. |
| `loot_tables.rs` | 354 |  | 1 | L084 (Sanctuary port P3): keyed loot tables of weighted entries. |
| `mail_sorting.rs` | 64 | yes | 2 | L103: Mail Sorting, the Postman's minigame, on the shared Type8 engine. |
| `main.rs` | 307 |  | 30 |  |
| `matchmaking.rs` | 1605 |  | 3 | Reconstructed transport-free queue and match coordination. |
| `minigame.rs` | 952 | yes | 59 | Shared, persistent lifecycle for one player's active minigame run. |
| `mining.rs` | 2525 | yes | 53 | Mining18 (TYPE8) shared match-three engine for the original `game_mining.gfx` movie. |
| `mining_lifecycle.rs` | 505 |  | 0 | Mining adapter for the shared minigame run lifecycle. |
| `moderation.rs` | 236 | yes | 4 | L042 staff moderation: mute, kick and ban, stored per account in the account store as unix seconds (ids only, never names). |
| `mounts.rs` | 1671 | yes | 15 | Session-scoped mount ownership and actor lifecycle. |
| `movement.rs` | 407 | yes | 48 | Transport-free validation for client-reported player positions. |
| `music_game.rs` | 863 | yes | 2 | L123: "Feeling the Beat!", client minigame type 9 (MiniGameData 201-204, the Games window rows 511-513 and 568): the music conducting movie `game_music.gfx`, re |
| `mystery_chest.rs` | 222 |  | 3 | Mystery Chest protocol codec. |
| `mystery_chest_loot.rs` | 296 | yes | 2 | Local loot bindings; the original drop distribution was not supplied. |
| `net_policy.rs` | 198 | yes | 1 | Which addresses the servers may listen on and accept packets from (Security Model L1). |
| `npc_ambient.rs` | 399 | yes | 1 | T507/T552: per-model ambient animations for passive content NPCs. |
| `npc_interaction.rs` | 1847 | yes | 72 | Content-backed NPC interaction state for the restoration server. |
| `npc_notifications.rs` | 105 |  | 1 | Actor notifications: the 35/10 `PlayerUpdatePacketAddNotifications` packet and the content row behind it (a minimap indicator and an overhead icon for an actor) |
| `npc_presentation.rs` | 238 |  | 1 | T899: what an NPC's name id says about how it looks and answers a hover (`npc_presentation.json`): the hover cursor (35/12), the shop or combat marker rows (35/ |
| `obstacles.rs` | 1074 | yes | 2 | Zone obstacle meshes from the client's placed `CDTA` collision assets (T534). |
| `opening.rs` | 902 | yes | 167 | Bounded, persistent opening-story progression from private recovered content. |
| `opening_abilities.rs` | 516 | yes | 9 | Basic native hotbars from private, pinned community metadata. |
| `opening_journal.rs` | 609 | yes | 2 | Native opening quest journal, derived from the preserved client readers. |
| `opening_world.rs` | 102 | yes | 2 | Native actors and interaction menus for the bounded opening content. |
| `pathing.rs` | 254 | yes | 2 | Answer to the client's path request (98/1): the objective breadcrumb and Take Me There. |
| `pet_follow.rs` | 873 |  | 2 | T455: server-owned pet following. |
| `pets.rs` | 3784 | yes | 57 | Saved pet roster and bounded client-shaped pet state. |
| `placed_cakes.rs` | 511 | yes | 3 | T556: placed birthday cakes, part 1 of 2. |
| `player_gm_menu.rs` | 227 |  | 0 | L081 (owner 2026-10-31): the admin-only GM wheel on another player's click wheel. |
| `player_radial.rs` | 489 | yes | 3 | Player radial menu and Inspect (W143, Unrouted Client Requests brief #13). |
| `players.rs` | 1063 | yes | 48 | Live player sessions and deterministic area-of-interest message fanout. |
| `portrait_upload.rs` | 322 |  | 1 | L071: the Fotomat portrait upload. |
| `power_ups.rs` | 439 | yes | 5 | Combat power-up drop tables: content schema, bounded loader, pure roll. |
| `profiles.rs` | 948 | yes | 150 | Native class selection backed by verified private profile payloads. |
| `progression_extras.rs` | 1762 | yes | 9 | Collections, achievements, titles and claim-window state. |
| `quest_dialogue.rs` | 299 | yes | 0 | T543: multi-line quest talk conversations with player responses and retries. |
| `quest_engine.rs` | 3869 | yes | 110 | Definition-driven quest state and protocol sequence candidate. |
| `quest_journal.rs` | 377 | yes | 9 | Quest journal requests from the client: select a tracked quest (47/12), lock the selection (47/13) and start a breadcrumb quest (49/15). |
| `quest_markers.rs` | 181 | yes | 4 | Quest markers: the 35/10 and 35/11 packets and the rule that picks each NPC's marker. |
| `quests.rs` | 714 | yes | 156 | Persistent local quest counters and a reconstructed checkpoint activity. |
| `recipe_book.rs` | 408 | yes | 3 | Recipe-book start (client request 26/24) at a real crafting station, and the recipe window a station interaction opens (server command 26/25, [`open_window`]). |
| `recipes.rs` | 734 | yes | 19 | Crafting recipe content. |
| `redeem_codes.rs` | 408 | yes | 1 | L081: redeem codes (owner 2026-10-31: kept for handing out items, including the realm's own assets later). |
| `rewards.rs` | 419 | yes | 117 | Pure reward transaction staging over the existing inventory and profile snapshots. |
| `road_graph.rs` | 321 | yes | 1 | FabledRealms road graph: the client's own `FabledRealms.map`, for 98/1 path replies. |
| `security_log.rs` | 568 | yes | 8 | Bounded security audit (Security Model L9). |
| `session.rs` | 110 | yes | 197 | Session/control framing verified against the supplied SOE implementation and pinned Sanctuary Multi/ClockSync handlers. |
| `ship_combat.rs` | 1404 | yes | 2 | L963: the ship combat packet family (Pirate's Plunder), client family 94. |
| `smelting.rs` | 1075 | yes | 22 | Original smelting GFx protocol with explicitly reconstructed phase tuning. |
| `soccer.rs` | 1073 |  | 29 | Soccer packet layouts recovered from the native client readers and writers. |
| `soccer_ball.rs` | 228 |  | 2 | L955: the soccer ball in flight and on the turf: gravity, air drag, bounce and grass friction, stepped in 10 ms substeps. |
| `soccer_bots.rs` | 971 |  | 2 | L932: the server's soccer players (owner 2026-11-02: "Soccer needs enemy ai"). |
| `soccer_entry.rs` | 973 | yes | 5 | Native soccer entry through the activity portal and the shared minigame lifecycle. |
| `soccer_eval.rs` | 118 |  | 1 | L956: how a bot reads the pitch around the ball: the odds a pass gets through, how pressed a player is, and how good a pass to each team-mate is. |
| `soccer_match.rs` | 1214 | yes | 6 | Server-owned soccer match authority for the MMO005 restoration. |
| `soccer_pickups.rs` | 189 |  | 1 | L942: soccer power-ups as the live server ran them (capture `packets_soccer_minigame.pcapng`; client HUD switch at 0xb81414 and `Resources/CodeStringMappings.tx |
| `soccer_service.rs` | 2133 |  | 2 | Bounded native soccer lifecycle/wire service for the MMO005 restoration. |
| `social_chat.rs` | 1475 | yes | 11 | Chat packet handling and server-side chat policy. |
| `social_lists.rs` | 1774 | yes | 14 | Per-character friends and ignore lists, presence fanout, and volatile groups. |
| `soe.rs` | 575 | yes | 15 | Bounded legacy SOE framing and ordered receive state for reliable channel 0. |
| `soe_send.rs` | 309 | yes | 7 | Bounded channel-zero SOE reliable sending, without sockets or clock access. |
| `startup_catalogues.rs` | 268 |  | 2 | Pure builders for startup catalogue and status packets. |
| `startup_messages.rs` | 130 |  | 1 | Builders for startup packets whose client-facing layouts are currently understood well enough to serialize without consulting captured packet bytes. |
| `startup_selfdata.rs` | 1522 | yes | 12 | Opcode 12 (`SendSelfToClient`) reader and serializer. |
| `startup_snapshot.rs` | 987 | yes | 13 | Typed, capture-derived startup content. |
| `startup_world.rs` | 1081 |  | 69 | Typed builders for the captured world-content startup messages. |
| `station_cash.rs` | 1768 | yes | 25 | Station Cash wallet, Marketplace catalogue and ride orders (T230). |
| `studio.rs` | 122 | yes | 6 | Studio: a tiny loopback-only HTTP/1.1 endpoint for developer tooling. |
| `style_cards.rs` | 409 | yes | 4 | Style cards (item type 12): the 42/10-42/12 requests, the card-to-record rule, the 35/39 and 35/65 customization packets and the client tables a card resolves t |
| `terrain.rs` | 411 | yes | 32 | Ground heights from the client's terrain chunks, for developer teleports, NPC placement and enemy ground-following. |
| `tower_defense.rs` | 853 | yes | 1 | L962: Tower Defense, client minigame type 14 (MiniGameData 150-161, 250-261 and 350-361; Games window rows 195-206, 520-540 and 549-551): the movie `towerdefens |
| `trace.rs` | 267 | yes | 21 | Opt-in JSONL trace output shared by the local gateway and login binaries. |
| `trade.rs` | 1661 | yes | 12 | Player-to-player trade session service. |
| `world.rs` | 2222 | yes | 227 | Content-backed zone instances, player membership, and transfer staging. |
| `world_events.rs` | 3343 | yes | 10 | T458: world events, part 1. |
| `world_notice.rs` | 160 | yes | 1 | `realm broadcast <text>`: one server-owned system line for every online session. |
| `world_regions.rs` | 516 |  | 2 | T753: the shared world zone/region/place table (`data/content/regions.json`) and the position -> region/place lookup later zone slices call. |
| `world_text.rs` | 275 | yes | 11 | World Builder text: strings authored in-game get stable locale ids. |
| `zone_transition.rs` | 273 | yes | 12 | Original world scene transition packet (opcode 31, `ClientBeginZoning`). |

## bin

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `bin/osfr-account.rs` | 314 | yes | 3 | (R) Host-only account provisioning. |
| `bin/osfr-bot.rs` | 1167 | yes | 12 | Headless bot client: speaks the SOE UDP protocol to a local osfr-gateway, logs in like the native client (login, 13 ready, 10 loaded) and plays a scripted flow. |
| `bin/osfr-gateway.rs` | 4638 | yes | 19 | One local character and zone, using private reference initialization data. |
| `bin/osfr-login.rs` | 1193 | yes | 8 | Explicitly local native character creation/selection service. |
| `bin/osfr-terrain.rs` | 640 | yes | 0 | Build a zone's ground-height grid and, with `--obstacles`, its placed-object collision mesh from the client's terrain chunks. |
| `bin/osfr_gateway/admin_move.rs` | 168 | yes | 0 | L043: carry out a committed admin `realm move`. |
| `bin/osfr_gateway/ambient.rs` | 196 | yes | 13 | T507/T552: the per-tick ambient animation pass for passive content NPCs. |
| `bin/osfr_gateway/boombox_routing.rs` | 1066 | yes | 1 | T461: the gateway side of `osfr_tools::boombox`. |
| `bin/osfr_gateway/cake_routing.rs` | 452 | yes | 0 | T557: the gateway side of `osfr_tools::placed_cakes`. |
| `bin/osfr_gateway/card_listener.rs` | 1391 | yes | 0 | T770: the Card Duelist card service, a TCP listener beside the UDP gateway. |
| `bin/osfr_gateway/client_area_routing.rs` | 478 | yes | 1 | T457: the gateway side of `osfr_tools::client_areas`. |
| `bin/osfr_gateway/delivery.rs` | 394 | yes | 20 | Application-delivery framing for the gateway: one place that wraps an inner packet in the 5/1 tunnel the client decodes, builds a chat refusal, fans a server no |
| `bin/osfr_gateway/dormant_fields.rs` | 43 | yes | 0 | Dormant by default: a shared field with nobody in it ticks once more, then sleeps. |
| `bin/osfr_gateway/encounter_invites.rs` | 195 | yes | 0 | T440 (Shared World slice 5): the group encounter invitation route. |
| `bin/osfr_gateway/f120_fishing_hour_reconnect.rs` | 608 | yes | 0 |  |
| `bin/osfr_gateway/f160_field_views.rs` | 482 | yes | 0 | F160: shared-field view reconciliation around readiness/world-load loss, and the per-recipient delivery isolation that keeps one full queue out of the tick. |
| `bin/osfr_gateway/f161_pet_adoption_persist.rs` | 455 | yes | 1 | F161: a pet adoption spends an inventory certificate and adds a pet in one session commit. |
| `bin/osfr_gateway/f261_pickup_reload.rs` | 235 | yes | 0 | F261: every opcode-31 begin-zoning makes the client reload its scene, so `Session::commit_outcome` passes that fact into `Gateway::bind_run_instance` and the st |
| `bin/osfr_gateway/f276_field_session_save.rs` | 208 | yes | 1 | F276: a refused field action through the real session writes no character save. |
| `bin/osfr_gateway/fishing_fixtures.rs` | 74 | yes | 3 | Wire builders shared by the gateway fishing regressions. |
| `bin/osfr_gateway/fotomat_routing.rs` | 296 | yes | 1 | T519: the Fotomat 156/2 and 115 through `Session::dispatch_message`, the same staging/commit/route path a socket datagram takes. |
| `bin/osfr_gateway/gm_menu_routing.rs` | 214 | yes | 0 | L081: carry out an admin's GM wheel choice (`osfr_tools::player_gm_menu`) once the admin's message commits. |
| `bin/osfr_gateway/gm_panel_routing.rs` | 257 | yes | 0 | L072: the referee panel's `!gm` lines, answered here because only the gateway sees every player. |
| `bin/osfr_gateway/group_arena.rs` | 419 | yes | 0 | T441 (Shared World slice 6): grouped players in one battle run share one arena. |
| `bin/osfr_gateway/house_visits.rs` | 172 | yes | 0 | T872 house visits: another player's lot, served to a visitor and kept live. |
| `bin/osfr_gateway/isolation.rs` | 215 | yes | 7 | The isolation boundary: one session's panic ends that session alone. |
| `bin/osfr_gateway/l011_vendor_save_failure.rs` | 538 | yes | 0 | L011 (Security Model S12/L8): a vendor purchase whose character save fails changes nothing and ends only that session. |
| `bin/osfr_gateway/l039_login_pose.rs` | 146 | yes | 0 | L039: a save whose open-world pose is off the map or deep under the terrain loads at the nearest warpstone through the production load (`Session::new` -> `load_ |
| `bin/osfr_gateway/l042_moderation.rs` | 703 | yes | 0 | L042 moderation through the production path: account-store sessions bound by a real handoff ticket, `realm` chat through `Session::dispatch_message` (gateway ro |
| `bin/osfr_gateway/l043_admin_move.rs` | 731 | yes | 0 | L043: admin `realm move` through the production path, in account-store mode: a real `Server` (its security log opened by `Server::new`), sessions bound by login |
| `bin/osfr_gateway/lead_trade_commit.rs` | 189 | yes | 0 |  |
| `bin/osfr_gateway/moderation.rs` | 101 | yes | 4 | L042: the host half of moderation. |
| `bin/osfr_gateway/paths.rs` | 30 | yes | 67 | Local gateway file layout beside the character save: Studio savestates, the social list, the W173 chat settings and the per-character state files. |
| `bin/osfr_gateway/player_menu_routing.rs` | 67 | yes | 0 | W143/W163: the player click wheel (radial menu) on the shared gateway. |
| `bin/osfr_gateway/portrait_http.rs` | 399 | yes | 0 | L071: the portrait upload listener. |
| `bin/osfr_gateway/quest_reward_save_failure.rs` | 1169 | yes | 0 | T258 (Security Model S12/L8): a quest-reward turn-in whose character save fails changes nothing and ends only that session. |
| `bin/osfr_gateway/spine_pins.rs` | 69 | yes | 0 | Bodies of the W127 directed-field and shared combat-budget pins. |
| `bin/osfr_gateway/startup_source.rs` | 34 | yes | 0 | T807: where a run takes its startup base from. |
| `bin/osfr_gateway/style_card_use.rs` | 403 | yes | 2 | T478: a style card through the gateway's real dispatch and commit path (`Session:: dispatch_message`): the player and the viewers in range get one 35/39, viewer |
| `bin/osfr_gateway/t006_guild_routing.rs` | 550 | yes | 0 | T006: the family-129 route through the real session dispatch and save path. |
| `bin/osfr_gateway/t006_guilds.rs` | 177 | yes | 0 | T006: family-129 guild routing. |
| `bin/osfr_gateway/t020_group_routing.rs` | 1235 | yes | 1 |  |
| `bin/osfr_gateway/t199_world_notice.rs` | 620 | yes | 1 | T199: `realm broadcast <text>` fanout through the real gateway dispatch and commit path. |
| `bin/osfr_gateway/t458_world_events.rs` | 1086 | yes | 0 | T458: world events through the gateway host. |
| `bin/osfr_gateway/t507_npc_ambient.rs` | 681 | yes | 0 | T507: passive content NPCs play the per-model ambient animation row, and every viewer that knows the actor receives the same 35/8 (client header reader `0x008d6 |
| `bin/osfr_gateway/t852_content_presence.rs` | 252 | yes | 0 | T852: a release server starts with the repository's `data/content` alone. |
| `bin/osfr_gateway/t889_shared_gateway.rs` | 337 | yes | 0 | T889: what two clients on one shared gateway get from a field enemy click, from the derby trainer and from the kart race NPC. |
| `bin/osfr_gateway/test_support.rs` | 865 | yes | 0 | Shared setup for the gateway binary's `#[cfg(test)]` session modules: one scratch directory, one `Options` initializer and one socket-less session fixture, so a |
| `bin/osfr_gateway/tests.rs` | 1882 | yes | 264 |  |
| `bin/osfr_gateway/tick_stats.rs` | 274 | yes | 0 | Gateway tick timing for the stress gate (T431, plan 3.6). |
| `bin/osfr_gateway/w163_trade_routing.rs` | 513 | yes | 0 |  |
| `bin/osfr_gateway/w173_chat_settings_routing.rs` | 590 | yes | 0 |  |
| `bin/osfr_gateway/world_event_routing.rs` | 238 | yes | 0 | T458: the gateway host's side of world events (`osfr_tools::world_events`). |
| `bin/soe-probe.rs` | 81 | yes | 0 | First transport slice only: the legacy SOE session handshake, on loopback. |

## encounter

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `encounter/arena.rs` | 1053 |  | 33 | One battle run's enemy simulation: attack clocks, walk set and skittish kiting. |
| `encounter/battle_drops.rs` | 199 |  | 0 | T658: chance item drops on a field-battle win (`drops` rows in `field_battles.json`). |
| `encounter/battle_items.rs` | 213 |  | 0 | L088 (Sanctuary port P4c): battle items, the orbs and spheres thrown from the item bar. |
| `encounter/combat_scale.rs` | 156 |  | 27 | The combat scale: how hard players hit, how tough they are, which levels open each weapon tier, and how enemy rows follow. |
| `encounter/field.rs` | 825 |  | 247 | Overworld field enemies and battle markers at runtime: wander and chase AI and the battle start screen. |
| `encounter/field_camps.rs` | 199 |  | 1 | T892: camp enemies. |
| `encounter/field_content.rs` | 418 |  | 1 | Field content loading, validation and staged reloads (`field_enemies.json`, `field_battles.json`), and the quest pin's lookups over the loaded rows. |
| `encounter/field_marker.rs` | 59 |  | 0 | T879: the combat marker an overworld enemy or battle spawner shows on the minimap and as an on-screen label: a full 35/10 notification of kind 3 ("Combat"), sen |
| `encounter/field_rewards.rs` | 624 |  | 2 | Field-kill rewards: the bounded level-band table, the one coin/job-XP payout and the one quest event each credited player present at the claim receives for a se |
| `encounter/power_up_use.rs` | 150 |  | 0 | T222: the held combat power-up on its own action-bar key. |
| `encounter/restore.rs` | 54 |  | 33 | L085 (Sanctuary port P4a): a potion's heal over time. |
| `encounter/revive.rs` | 173 |  | 8 | Knockout and revive: the respawn window, the client's resume request and the reconstructed revive rules (moved out of `encounter.rs` by T402). |
| `encounter/shared.rs` | 697 |  | 224 | Shared field state: one overworld enemy simulation per instance, shared by every participant, and its engine entry points. |
| `encounter/skills.rs` | 592 |  | 7 | Job skills: abilities, damage over time, buffs, stuns, mitigation, energy and critical hits. |
| `encounter/threat.rs` | 95 |  | 4 | L083 (Sanctuary port P2): a field enemy's threat table. |

## gameplay

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `gameplay/admin_move.rs` | 270 | yes | 0 | L043: `realm move <character> <warpstone name / x z / x y z>`, for admins only. |
| `gameplay/card_launch.rs` | 63 | yes | 1 | T765: Trading Card Game launch glue for `Gameplay`; the run and ticket rules live in `crate::card_game::Launches`. |
| `gameplay/checkers_table.rs` | 55 | yes | 0 | L104: the checkers table's glue for `Gameplay`: Play on the Games window row offers the table, and its lifecycle and movie packets route to it. |
| `gameplay/coin_authoring.rs` | 480 | yes | 0 | T525: live coin authoring (`realm coin place/move/unbind/list`), Admin-only by the existing realm role gate (`account_store::can_run_realm_command`, unknown com |
| `gameplay/collection_pickup.rs` | 470 | yes | 0 | T450: exploration coins. |
| `gameplay/content_presence.rs` | 217 | yes | 0 | T852: a gameplay load starts with the content its directory actually carries. |
| `gameplay/content_reload.rs` | 369 | yes | 1 | Developer hot reload of authored content: battles, enemies, abilities. |
| `gameplay/crafting.rs` | 690 | yes | 13 | Crafting and gathering: the five movie gate, cancel and tick paths, the gather start and the server-side appliers for their staged outcomes. |
| `gameplay/daily_wheel.rs` | 54 | yes | 1 | T560: daily wheel glue for `Gameplay`; the rules, the 143/1 spin count and the run live in `crate::daily_wheel`. |
| `gameplay/dance_off.rs` | 83 | yes | 1 | T538: the dance-off's gameplay side. |
| `gameplay/derby_rows.rs` | 31 | yes | 6 | Derby row selection at ordinary NPC entry (T791). |
| `gameplay/derby_session.rs` | 350 | yes | 0 | Derby gameplay integration: job switching, completion payout, cancellation and session ticking. |
| `gameplay/developer_teleport.rs` | 626 | yes | 2 |  |
| `gameplay/encounter_rewards.rs` | 124 | yes | 2 | Encounter payout application: one reward at a time, with the live wallet and active job deciding whether the pair can be represented. |
| `gameplay/field_battle.rs` | 181 | yes | 1 | Field-battle start screen, prize preflight and arena entry. |
| `gameplay/fishing_scene.rs` | 130 | yes | 2 | T514: a fishing round at a hole whose scene the client has (`client_scene_cached`) is fished in that scene, in the housing pattern (`gameplay/housing.rs`). |
| `gameplay/group_arena.rs` | 65 | yes | 0 | T441: a group's shared battle run, as one member's gameplay sees it. |
| `gameplay/housing.rs` | 412 | yes | 27 | Player housing in the world session (lead, 2026-10-20, slice 1): the client asks for its houses (127/23 grant, 127/14 list), enters one (127/19) and leaves (127 |
| `gameplay/item_requests.rs` | 59 | yes | 0 | Mystery chest requests, answered before the shared 42 inventory branch, and the 35/6 an equip change sends neighbours (moved out of the `gameplay.rs` hub). |
| `gameplay/item_use.rs` | 408 | yes | 1 | The item bar and the consumable effect stack on `Gameplay` (W146, T474). |
| `gameplay/jobs.rs` | 440 | yes | 68 | Job selection, trainer learning and starter outfits. |
| `gameplay/kart_rows.rs` | 24 | yes | 4 | L930: kart row selection at entry (the derby's `derby_rows.rs`, for the kart race). |
| `gameplay/knockout.rs` | 50 | yes | 12 | Knockout on the gameplay side: field damage that can knock the player out, and the paid revive back up. |
| `gameplay/live_events.rs` | 289 | yes | 3 | T418: live events an admin opens and closes while the world runs. |
| `gameplay/matchmaking_route.rs` | 34 | yes | 0 | Matchmaking list/stat replies, the empty in-progress list, and the lobby close request. |
| `gameplay/minigame_credit.rs` | 43 | yes | 0 | (R) T787: one settled server minigame reports its result facts to the quest engine. |
| `gameplay/movie.rs` | 225 | yes | 44 | The six crafting and gathering movies (L103: Mail Sorting the sixth): gate order, busy row and fixed refusals. |
| `gameplay/music_table.rs` | 69 | yes | 0 | L961 (L123's engine): Feeling the Beat!'s glue for `Gameplay`: Play on a Games window row offers that game, and its lifecycle and movie packets route to it. |
| `gameplay/npc.rs` | 1967 | yes | 146 | NPC interaction router: the 26/x, 45/7, 49/x, 165/x and 66/x families, NPC visibility, and the dialogue, merchant and quest-offer packet builders. |
| `gameplay/npc_quests.rs` | 667 | yes | 63 | NPC quest glue: catalogue binding, breadcrumbs, eligibility and quest events for NPC interactions. |
| `gameplay/npc_turnin.rs` | 57 | yes | 0 | Saved turn-in replay: a pending 49/13 end screen is re-sent only in a batch that also carries its opener's 35/2 AddNpc, on both the per-session and shared-field |
| `gameplay/occupation.rs` | 348 | yes | 9 | What a player is doing right now, and what each activity blocks. |
| `gameplay/ordinary_quit_routing.rs` | 87 | yes | 0 |  |
| `gameplay/pet_quest.rs` | 26 | yes | 0 | T788: the accepted pet trick feeds the quest engine. |
| `gameplay/pirates_arena.rs` | 192 | yes | 0 | L963: Pirate's Plunder's match shell for `Gameplay` (slice 1). |
| `gameplay/placement_watch.rs` | 451 | yes | 0 | Per-session watch of the live-world placement table. |
| `gameplay/portal_games.rs` | 219 | yes | 0 | L095/L097: Games window glue for `Gameplay`: the live rows over the assembled catalogue, the per-player lists, and Play for the kart race and the derby. |
| `gameplay/power_up_use.rs` | 143 | yes | 0 | T222: the held combat power-up's use request and its own key slot. |
| `gameplay/profile_outfit.rs` | 45 | yes | 0 | Public appearance updates after a profile or outfit change. |
| `gameplay/progression.rs` | 147 | yes | 16 | Progress-driven achievements over authoritative gameplay state. |
| `gameplay/quest_delivery.rs` | 218 | yes | 1 | T786 (R): a Talk step that hands items to an NPC. |
| `gameplay/quest_pickups.rs` | 64 | yes | 0 | T503: quest-only pickup nodes. |
| `gameplay/quest_use.rs` | 54 | yes | 0 | T789: accepted ability and item uses feed the quest engine. |
| `gameplay/redeem.rs` | 85 | yes | 4 | L081: `realm redeem [code]` for every player and `realm code ...` for admins: the GM panel's `!redeem` and `!code` (Sanctuary PR #163 protocol). |
| `gameplay/reward_adoption.rs` | 224 | yes | 0 | D004: minigame reward adoption regressions. |
| `gameplay/rides/pet_treasure.rs` | 90 | yes | 0 | T533: the owner's pet treasure chest in `Gameplay`: the world tick of the hunt and the click (26/8) that opens the chest. |
| `gameplay/rides.rs` | 775 | yes | 15 | Ride and pet glue for `Gameplay`; the rules live in `crate::mounts` and `crate::pets`. |
| `gameplay/s15_npc_transactions.rs` | 833 | yes | 0 | T254 / restructure S15 step 0: one 26/10 NPC menu choice is one transaction. |
| `gameplay/s6_characterization.rs` | 803 | yes | 1 | Private fixture checks that need server-internal movie clocks or private appliers. |
| `gameplay/sc_earnings.rs` | 189 | yes | 0 | T229: Station Cash earned by server-verified completions. |
| `gameplay/seeds.rs` | 24 | yes | 16 | (R) Deterministic seams for tests and offline fixtures. |
| `gameplay/soccer_session.rs` | 73 | yes | 0 | Starting a soccer match: the Games window row, a soccer gate, or the row an open quest names (L932). |
| `gameplay/startup_health.rs` | 31 | yes | 0 | The active job's level sets the encounter's max health, and a world reload's startup stream (index 33, 38/1) reports that health instead of the captured 400/400 |
| `gameplay/style_card_use.rs` | 171 | yes | 2 | Style cards: 42/10 UseStyleCard and 42/12 UseStyleCardByItemRecord change the character's look, consume one card, and tell the player and everyone nearby (35/39 |
| `gameplay/style_card_use_tests.rs` | 483 | yes | 0 | T478: 42/10 and 42/12 through `Gameplay::handle_in_world`, the router the gateway calls, on both content shapes: the open world (`Gameplay::handle`) and the ope |
| `gameplay/t013_objective_events.rs` | 2455 | yes | 2 |  |
| `gameplay/t018_station_refresh.rs` | 223 | yes | 0 |  |
| `gameplay/tower_defense_table.rs` | 68 | yes | 0 | L962: Tower Defense's glue for `Gameplay`: Play on a Games window row offers that game, and its lifecycle and movie packets route to it. |
| `gameplay/w104_quest_tests.rs` | 441 | yes | 0 |  |
| `gameplay/w64_regional_quest_tests.rs` | 3141 | yes | 0 |  |
| `gameplay/w95_vendor_coin_store_regressions.rs` | 175 | yes | 0 |  |

## gateway

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `gateway/admin_move.rs` | 84 | yes | 0 | L043: the moved character's side of an admin `realm move`. |
| `gateway/application_preload.rs` | 101 | yes | 0 | Application requests handled around the gateway's pre-world gate: W174's accepted and discarded quiet list, and the login-time Fotomat 156/2 that the client sen |
| `gateway/companions.rs` | 70 |  | 2 | Ride and pet glue on a session gateway (W162/W177), moved verbatim from the hub (T458). |
| `gateway/encounter_join.rs` | 50 |  | 0 | T440: the invited member's side of a group encounter. |
| `gateway/group_arena.rs` | 65 | yes | 0 | T441: a group's shared battle run, as one session's gateway sees it: lend the World's arena to the engine around one staged action, and the member-side steps of |
| `gateway/staff.rs` | 93 |  | 5 | Staff `realm` words on a session gateway: `realm role` and the L042 moderation commands. |
| `gateway/startup_source.rs` | 142 | yes | 0 | Where the gateway's startup base comes from: the validated capture folder (`--reference`, lab use) or the repo's own `<content>/startup/` files, which need no c |
| `gateway/startup_stream.rs` | 147 |  | 0 | The login/ready startup stream: the typed snapshot's 46 messages with their live substitutions. |
| `gateway/trade_bridge.rs` | 36 |  | 0 | Trade bridges on a session gateway (W163/W172). |

## housing_service

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `housing_service/creative.rs` | 102 |  | 2 | T870 creative decorate catalog: every row of `housing_fixtures.json` is placeable without owning it, and a piece placed that way never becomes an item. |

## npc_ambient

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `npc_ambient/schedule.rs` | 239 |  | 17 | T552: the schedules fixed at load from the placed content actors: conversation groups, idle loops and look-at actors, and the `due` rule of each. |
| `npc_ambient/tests.rs` | 411 |  | 264 |  |
| `npc_ambient/wire.rs` | 60 |  | 176 | T507/T552: the three ambient wire bodies and the update that owns them. |

## pets

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `pets/grooming.rs` | 384 |  | 3 | T488: pet grooming and petting. |
| `pets/idle.rs` | 494 |  | 54 | T531: pet idle life and thought bubbles. |
| `pets/treasure.rs` | 591 |  | 3 | T533: pet treasure. |
| `pets/tricks.rs` | 271 |  | 9 | T489: pet tricks. |

## world_events

| Module | Lines | Hub | Tests | Purpose |
|---|---:|---|---:|---|
| `world_events/schedule.rs` | 793 |  | 17 | T462: world events, part 3. |

## Elsewhere

- `src/restoration/`: 92 restoration pieces (client behaviour recovered piece by piece).
- `prototype/osfr-tools/tests/`: 594 test files; `tests/spine_rules.rs` holds the hub line budgets.
- `prototype/osfr-tools/data/content/` and `data/release/`: the gameplay content the gateway loads.
- `scripts/`: harness, release, audit and data tools (`run-local-baseline.py`, `lua_bridge.py`, `asset_delivery.py`).
- `notes/Native Map/`: the decoded client functions; `notes/Research/` and `notes/Design/`: evidence and design references (snapshots; verify against the code).
