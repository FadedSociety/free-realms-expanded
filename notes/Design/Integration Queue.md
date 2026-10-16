# Integration Queue

This note lists the work integration builders must do after the scoped builders finish. Each round owns `lib.rs`, the gateway and the bins, and ends with a full-suite gate and a native test.

## Done

- Round 3 (W55): account-store login with HMAC handoff; chess, guilds and matchmaking registered. Natively verified in run 023623, after lead launcher and gateway fixes.
- Round 4 (W59): `game_rules`, `npc_interaction`, `progression_extras`, `recipes` and `harvest_nodes` registered; the `legacy-data` source tag; zone NPC visibility (100 m, nearest 64 (R)); regional NPC menus and vendor stock; account info from `GameRuleSet`.
- Lead fixes after W55 and W59 (2026-10-14):
  - `pets.json` source tags renamed from `"R"` to `reconstructed`.
  - The `npc_interaction` merchant test no longer assumes an empty stock.
  - W59 had replaced the character-list row payload with a base64 name. That dropped the appearance record the native character select draws the model from. It is restored via `login::appearance_payload`, which `selection_reply` shares, and `tests/login.rs` now expects it.
  - `tests/piece_012.rs` gained W60's `experience_points` field.
  - Backups are in `lab/snapshots/` (`login.rs.pre-payloadfix-20261014`, `pets.json.pre-tagfix-20261014`, `run-local-baseline.py.pre-handoff-20261014`).

## Next round (5)

| Item | Source | Work |
| --- | --- | --- |
| Movement | W53/W53b | `pub mod movement` (ready: 14,982 real updates from 59 runs replay with 0 rejects and 0 clamps). Validate opcode 125 before committing a position. The server issues discontinuity tokens on teleport (`dev_teleport`), relocation and zone change, and ignores stale reports. |
| Recipes on the wire | W51b/W60 | After login, send `0x3f` operation 5 (replace-all) with the known recipes, and operation 1 when one is learned. Track learned recipes in the save. Gameplay passes the learned state, rank and inventory to `begin_recipe` instead of `begin(profile)`. Crafting XP is 0 (R) until evidence exists. |
| Quests from NPCs | W59 gaps | Build the typed quest catalogue from the split regional content (`npcs/*.json`, `quests/*.json`). Route quest accept 49/2 into `quest_engine`, put its state in `GameplayState` and save it. Keep the opening flow byte-for-byte. |
| Vendor purchases | W59 gaps | Map 66/43 fields to CoinShop transactions, or keep vendors on family 165 buy/sell. This needs native capture evidence first. |
| Extra NPCs | W56/W59 | `npc_spawns_extra.json` has no zone field (W59 assigns the nearest regional anchor, and long distances may be wrong) and no interaction keys. |
| Enemies | W58/W58b | `EnemyDefinition` gets typed `abilities`, `loot_table_refs` and per-field sources. Spawn enemies from content via `enemies.rs`; the dungeon `dungeon_data.enemy_refs` crosswalk exists. |
| Game rules | W54 | Self-data flags come from `GameRuleSet`. Member gates apply in the coin shop, housing, mounts and pets. |
| Matchmaking | W52 | Route families 141 and 175; the minigame handoff replaces the per-game lobbies. |
| Guilds | W48 | Route family 129; guild chat channel; save the guild store. |
| Progression | W49b/W57 | Collections, achievements and titles messages (families 150 and 152); counters hook into job rank and quest completion. |
| Harvest nodes | W57 | Spawn nodes; loot through `rewards`. |
| Character-select labels | runs 011719/023623 | The native list shows "1"/"0" instead of the name. This predates W55. The row payload is the appearance record, so find which field the label reads (`00fa4370` `ReadEntityRecord`; the leading empty string in the appearance record is a candidate) and test it natively. Never drop the appearance. |
