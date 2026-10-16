# World Enemies

## Scope and evidence

Task W36 adds an instance-scoped enemy engine in `prototype/osfr-tools/src/enemies.rs`. It is not registered in `lib.rs`; `tests/enemies.rs` includes it with `#[path]` until lead integration.

At task start, the requested `notes/Research/World Enemies.md` did not exist. The existing `Actors and NPCs` guide has no narrative yet. The Combat and Abilities guide is the reviewed client map for combat results, while the startup-content note covers AddNpc. The reverse-engineering workflow was applied by checking the supplied native-map disassembly/labels before relying on packet readers. Game recon, web research and the asset pipeline were not needed for this code-only task.

## Client facts

- `Combat and Abilities.md`, Network and “Damage, healing and the health bar”: 32/7 is server-to-client. Reader `00a2a910` reads three GUID pairs (the second overwrites the first), three `i32`, two bytes, then two `i32`. Handler `00a2ba40` uses the first result integer as damage, the second as target max health, and the last integer as current target health. The code emits the same body layout as the accepted opening encounter.
- The same guide identifies 35/5 `UpdateHitpoints` reader `008d6f10` and its `u64 + 3*i32` body. 38/1 has current and maximum `i32` health; dispatcher `009392c0`, reader `0089d2b0`. 32/9 has a GUID and bool; reader `00a2a8a0`.
- `Startup World Content.md`, “NPC spawns, 35/2”: confirmed reader `00921a10` reads the AddNpc body. The module reuses `startup_world::build_add_npc` and unwraps its established tunnel prefix. Unnamed AddNpc field meanings remain reconstructed defaults.
- 35/8 animation is server-to-client and is accepted in the recorded native opening fights. Existing `EncounterEngine` uses animation ids 3 (run), 1 (idle) and 1001 (attack). The numeric AI choices are not proven original server choices.
- Existing `EncounterEngine` writes standalone opcode 125 movement as actor GUID, position xyz, heading xyz, and bytes `[1, 0]`. No dedicated client reader for this opcode was present in the named movement guide. W36 copies that exact accepted opening-writer layout; it does not claim original-server provenance for the flags.
- 35/3 actor removal, 32/7 results, 35/5 health and 32/9 boss display use the existing opening encounter's packet writers. The native client acceptance of those local messages proves compatibility for the tested flow, not historical server behavior.
- Native opening recordings show wolves running/lunging and Mac approaching with changing hammer poses. They do not establish original enemy AI thresholds or a low-health flee behavior. No flee state was added.

## Reconstructed policy (R)

All enemy AI and combat policy in W36 is local reconstruction: aggro radius 40, leash radius 40, straight-line movement at 3 units/sec, 100 ms minimum tick, 250 ms movement cap, 2.5-unit attack range, 1,000 ms attack cooldown, and 30,000 ms respawn. Leash return, nearest-player tie-breaking, patrol waypoint handling, private-instance GUID derivation, per-life reward keys, and no same-tick attack immediately after respawn are also (R). There is no pathfinding.

`enemies.json` retains the current local prototype's Hooligan Wolf, Gloam Creature, Mac, Gloamling and Gloam Overlord actor fields and spawn poses. Source file: `../lab/references/opening-journal-text-v2/content/encounters.json`, SHA256 `2d465c7e70edcd725345f512c26883fb0d31124e2af89d1b103e432927b5d9a4`. The existing `encounter.rs` header already labels placement, health, damage, timing and rewards as reconstructed. `dungeons/opening_cave.json` repeats four `native_config.enemies` rows with model, position, health and damage, but each row also marks its source as reconstructed from that same local encounter package. These are not original client/server facts. Across the 75 dungeon files, the 275 enemy/boss guide rows contain 234 named entries; the opening-cave actor rows are the only rows with model/health values. The NPC files have 82 nonempty descriptive `enemy_kind` rows across 74 names; none has a position. W36 leaves level null instead of inventing one.

W36 assigns one coin and one active-job XP per kill as a testable example reward. This is (R), not recovered retail loot/XP. Item grants are supported by the same `rewards::apply_grant` path but this content package invents no item drop. Quest progression is emitted as `QuestEvent::KillActorKind` once per death. `rewards::apply_grant` stages a per-life key; callers still own persistence and send-after-save ordering.

## Message sources used by W36

| Message | Source and reader | W36 use |
|---|---|---|
| 35/2 AddNpc | `Startup World Content.md`, reader `00921a10`; writer `startup_world::build_add_npc` | Initial spawn and respawn |
| 35/3 actor removal | Existing `encounter.rs::remove`; dispatcher `0092f460`. No dedicated body reader was located in the named guides. | Death despawn; exact local opening-writer bytes reused |
| 35/5 UpdateHitpoints | reader `008d6f10` | Spawn/respawn health |
| 35/8 Animation | dispatcher `0092f460`; native-accepted opening writer (Combat guide) | Run, idle, attack poses; no separate body-reader address recorded |
| 32/7 AttackProcessed | reader `00a2a910`, handler `00a2ba40` | Enemy-to-player hit result; player abilities reuse `abilities::use_ability` |
| 32/9 boss display | reader `00a2a8a0` | Boss register and removal |
| 38/1 Hitpoints | dispatcher `009392c0`, reader `0089d2b0` | Player health after enemy attack |
| 125 movement | No dedicated reader identified in the named guides; exact existing `EncounterEngine::movement` writer | Straight-line movement and return; layout/flags remain an accepted local writer, not recovered client facts |
| 38/19 coin count | `Items and Inventory.md`, reader `0089e670`; dispatcher `009392c0` | Coin grant |
| 38/14 experience, 38/18 rank | `Rewards and Collections.md`, dispatcher `009392c0`; readers `008d86e0`, `008d8910` | Job XP and rank grant |
| 38/2 item add | `Items and Inventory.md`, reader `0089d410`; dispatcher `009392c0` | Optional item grant |
| 50/1 reward bundle | `Rewards and Collections.md`, dispatcher `00b8a640`, reader `00b8a300`, shared body reader `008e7930` | Reward display after grant |

## Lead integration steps

1. Register `enemies` in the shared library and load `data/content/enemies.json` through the normal content validation path.
2. Create one enemy state per `World::ZoneInstance`; pass only same-instance `PlayerRegistry` members to `tick`, and fan out resulting messages through existing interest tracking.
3. Enter/activate stage-gated groups after the client-ready gate. Send the current live enemy snapshot to late joiners; `enter_instance` itself is idempotent.
4. Route authorized ability targets through `use_ability_on_enemy`. Persist the returned `RewardState`, quest event result, and enemy life/reward key in the same character transaction before sending packets.
5. Run focused integration checks and lead-controlled native client acceptance. W36's tests prove local packet equality with `EncounterEngine`; they do not prove retail behavior, visual correctness, balance, collision, or release readiness.

## Verification record

- Read the three named Build Guides, `encounter.rs`, `abilities.rs`, `world.rs`, `players.rs`, `rewards.rs`, and `quest_engine.rs`.
- `python -I scripts/native_map/nmq.py --help` confirmed the read-only `brief` and `dis` interfaces.
- `python -I scripts/native_map/nmq.py dis 00a2a910 160`, `... dis 00a2ba40 260`, `... dis 008d6f10 120`, `... dis 00bf9430 120`, and `... brief 00a2a910 00a2ba40 008d6f10 0092f460 00bf9430` checked the packet and movement-related labels/bytes. Unknown handler labels were not treated as confirmed facts.
- Python read-only audits covered 75 dungeon JSON files and 17 NPC JSON files. They found 275 enemy/boss guide rows (234 named), five model-bearing rows including a duplicate boss listing, and 82 NPC enemy-kind records with no positions. `opening_cave.json` repeats the four cave actor rows, all with reconstructed provenance.
- Formatting: private `rustfmt.exe --edition 2024` and its `--check` passed on both owned Rust files after the final code edits.
- Focused test attempt 1: `scripts/cargo.ps1 test --test enemies` compiled and ran six tests; five passed and the exact 32/7-byte assertion failed because its expected hex had two extra zero bytes. The expected vector was corrected to the 50-byte layout emitted by `EncounterEngine::attack_processed` and `abilities::build_attack_processed`.
- Focused test retry: the same requested command stopped before the test target ran because the concurrently changing, unowned `src/bin/osfr-gateway.rs` fails compilation (`NameData` missing at line 205; `request` and `gateway` used after move at lines 400 and 408). Attempts with `--bin soe-probe` and `--lib --test enemies` reached the same errors. No passing result is claimed for the corrected six-test suite; the lead can rerun after the gateway build is repaired.

## W58 dungeon enemy definitions (2026-10-13)

`data/content/enemies.json` preserves the five W36 definitions and adds 26 dungeon enemy/boss definitions with local IDs `930000..930025` (`w58_*`). The exact per-field evidence, unresolved model gaps, source-file hashes, and ability strings are in `data/content/enemies_unresolved.json`. These are additions to the current prototype data, not recovered retail enemy records.

### Client and Legacy provenance

- Each added `name_string_id` resolves to the displayed `kind` in `lab/reports/locale-id-to-text.json`.
- The numeric `model_id` and ADR filename are paired from the starting crosswalk `lab/references/FreeRealms-Legacy/src/Resources/Npcs.json`; the exact ADR exists in `lab/extracted/client-assets/catalog.tsv`. The catalog has no numeric model-ID column, and the Legacy pair has not been independently verified against a native `AddNpc` for these dungeon actors. The table below therefore records a Legacy candidate cross-checked against the client asset name, not a recovered native numeric mapping.
- Shady Smuggler uses `TextureAlias: "03"` from Legacy NPC id `2834` (NameId `140959`, ModelId `669`). The other 25 selected Legacy rows have no `TextureAlias` value; their empty strings are (R), with native client aliases unverified.
- `lab/content/startup-snapshot/startup.json` was inspected for `AddNpc` fields and relevant world NPCs; it contains no matching dungeon enemy actor. Its SHA256 is recorded in the sidecar.
- Pinned wiki revisions are recorded per difficulty source in the sidecar. The wiki pages supply encounter difficulty/role context; they do not supply the numeric per-enemy health, damage, speeds, or loot values below.

| Definition ID | Name | Name string ID | Model ID | Client ADR checked | Level (R) | HP (R) | Damage (R) | Role |
|---:|---|---:|---:|---|---:|---:|---:|---|
| 930000 | Bigfoot | 428930 | 3001 | `sasquatch_bigfoot_costume.adr` | 5 | 540 | 30 | Boss |
| 930001 | Brutus the Brute | 116426 | 786 | `robgoblin_m_brute.adr` | 13 | 340 | 31 | Enemy |
| 930002 | Changeling Heartbreaker | 427522 | 2041 | `changeling_emo_f.adr` | 13 | 340 | 31 | Enemy |
| 930003 | Chugawump | 72862 | 338 | `chugawug_m_sergeantcargug.adr` | 13 | 1020 | 62 | Boss |
| 930004 | Counselor Edmond | 428934 | 3019 | `fairy_m_freestyle_Edmond.agr` | 5 | 180 | 15 | Enemy |
| 930005 | Counselor Jake | 428935 | 3022 | `human_m_freestyle_Jake.agr` | 5 | 180 | 15 | Enemy |
| 930006 | Counselor Kaela | 428931 | 3017 | `fairy_f_freestyle_Kaela.agr` | 5 | 180 | 15 | Enemy |
| 930007 | Counselor Tyra | 428932 | 3020 | `human_f_freestyle_Tyra.agr` | 5 | 180 | 15 | Enemy |
| 930008 | Cray Snapper | 91930 | 470 | `cray_ocean.adr` | 5 | 180 | 15 | Enemy |
| 930009 | Evil Postman | 384822 | 881 | `human_m_courier_evil.agr` | 5 | 180 | 15 | Enemy |
| 930010 | Fairy Cultist | 441992 | 500 | `sprite_cultist_m.adr` | 3 | 140 | 11 | Enemy |
| 930011 | Grave Elemental | 115851 | 767 | `graveelemental_m_skullwgraves.adr` | 5 | 180 | 15 | Enemy |
| 930012 | Hooligan Archer | 5100400 | 520 | `human_m_hooligan_archer_02.agr` | 3 | 140 | 11 | Enemy |
| 930013 | Lazy Swooper | 93049 | 750 | `birds_seagull.adr` | 5 | 180 | 15 | Enemy |
| 930014 | Necronomicus | 440729 | 4220 | `necronomicus_m.adr` | 13 | 1020 | 62 | Boss |
| 930015 | Nettleseed Nibbler | 45945 | 3250 | `maneatingplant.adr` | 5 | 180 | 15 | Enemy |
| 930016 | Nettleseed Pest | 73988 | 160 | `maneatingfungus.adr` | 13 | 340 | 31 | Enemy |
| 930017 | Sandscale Hunter | 5102869 | 1528 | `manguana_m_base.adr` | 3 | 140 | 11 | Enemy |
| 930018 | Sandscale Scavenger | 403236 | 3993 | `manguana_m_scavenger.adr` | 3 | 140 | 11 | Enemy |
| 930019 | Scavenger Swooper | 5101508 | 750 | `birds_seagull.adr` | 5 | 180 | 15 | Enemy |
| 930020 | Shady Smuggler | 140959 | 669 | `human_blackspore_thief_m.adr` | 9 | 260 | 23 | Enemy |
| 930021 | Stealthy Despoiler | 93120 | 945 | `human_m_ninja_ghost.adr` | 9 | 260 | 23 | Enemy |
| 930022 | Stone Heart | 427068 | 2055 | `stoneheart_m.adr` | 13 | 1020 | 62 | Boss |
| 930023 | Surfmonger Arthur | 139701 | 559 | `human_m_freestyle_06.agr` | 9 | 780 | 46 | Boss |
| 930024 | The Pumpkin Prince | 414863 | 1702 | `pumpkinprince_m_boss.adr` | 5 | 540 | 30 | Boss |
| 930025 | Troll Bone Shaman | 72269 | 168 | `troll_shaman.adr` | 1 | 100 | 7 | Enemy |

### Reconstructed combat values and unsupported fields

Every added numeric combat value is (R). Wiki dungeon difficulty maps to a minimum level as `1→1`, `2→3`, `3→5`, `4→9`, `5→13`; page-specific difficulty/revision evidence is kept per definition. Normal health is `80 + 20×level`, boss health is three times that. Normal damage is `5 + 2×level`, boss damage is twice that. Boss status follows the named guide role, not the array a row appears in; this corrected Brutus the Brute's “Not the boss” row to enemy health/damage and `boss: false`.

Movement and combat policy reuses the existing W36 reconstruction: speed `3.0`, aggro/leash `40.0`, attack range `2.5`, cooldown `1000 ms`, and respawn `30000 ms`. New definitions grant zero coins, XP, and items because no original per-enemy reward or numeric loot-table reference was established. These are deliberately conservative local rules, not recovered balance.

The sidecar records 28 client-string/dev-string evidence entries across seven definitions, including 26 guide `loc` references. Bigfoot, Chugawump, Grave Elemental, Necronomicus, Surfmonger Arthur, The Pumpkin Prince, and Stone Heart have names or ability-related strings. Some are dev labels, descriptions, or combat flags; they do not prove that a named ability is live or define its mechanics. The `loc` values are documented string references, not server ability IDs. No numeric loot-table refs were recovered.

The strict `EnemyDefinition` in `src/enemies.rs` uses `deny_unknown_fields` and has no `abilities`, `loot_table_refs`, or structured per-field source members. The Rust source and schema were left untouched. Its exact required change is: add a typed `abilities` member with source/reference validation; add a typed vector of `loot_table_refs` and validate each target; add a per-field source map with `kind` and `evidence`, replacing or supplementing `source: String`; give the added collections defaults for existing content. Until then, abilities and per-field provenance live in `enemies_unresolved.json`, while the loader-compatible `source` string summarizes the definition source. Exact field-level sources are recorded for all 31 definitions.

Four of the five preserved W36 model IDs lack an independently checked ID-to-ADR pair; Hooligan Wolf's `3284 → wolf_generic.adr` is cross-checked. Those four remain listed in `definition_field_gaps` and are not represented as client-catalog-verified.

World coverage remains incomplete: 82 `enemy_kind` rows in 74 regional NPC files (70 unique description texts in this pass) remain unresolved. They are descriptive leads without a hostile actor/model/stat record or position, and the inspected startup snapshot did not provide a matching `AddNpc`. Each row has its NPC content key and reason in `world_rows`; none was promoted to a combat definition based on prose alone.

### W58 verification

- `catalog.tsv` SHA256 `12BFD0F81236FFC02E6A9A661E1FFB14890A2BD4AC2306C14B20E9CBE445B1C9`; locale map SHA256 `F1F22A30107C824AF4B3021C8C7C323C927D3C6A9B82EFC387E9EABA6AE62844`.
- Legacy NPC source SHA256 `F8F224482A022FBB6C7D96280BF977ED0A648135BA965B51FB854BB75D69CFF2`; startup snapshot SHA256 `DA3A4AC1654522E212E7364FCF3870F5315E055A45B66F7ECDB116C904C0F276`; pinned wiki database SHA256 `B9D7B9F1ADABB42DB54C3253BF43FDDC249D6D70E3F2075B134A9441E2FCD911`.
- `content_enemies` checks all names against the locale map, all non-gap numeric model/ADR pairs against Legacy and the client catalog, and requires explicit reasons for model gaps. The native `AddNpc` numeric mapping remains unverified for the 26 additions.
## W58b enemy coverage and model binding audit (2026-10-13)

W58b expanded the 31 W58 definitions with 24 definitions based on new model, locale, capture, and pinned wiki evidence. The `enemies_unresolved.json` sidecar now stores one model binding per definition, exact Models.txt row numbers, catalog paths and hashes, ADR Toolkit closure reads, matched tokens or exact NameId reference joins, captured AddNpc tuples, and remaining candidate alternatives.

### Coverage

| Measure | Before W58b | After W58b |
|---|---:|---:|
| Dungeon named occurrences resolved | 45/367 | 74/367 |
| Unique dungeon names resolved | 32/250 | 46/250 |
| Regional `enemy_kind` rows fully resolved | 0/82 | 36/82 |
| Regional rows partially resolved | 0 | 3 |
| Definitions | 31 | 55 |

All 367 dungeon occurrences still have exactly one `enemy_refs` entry. The regional sidecar retains all 82 source rows with `resolved`, `partial`, or `unresolved` status and linked definition ids. Partial aggregate rows identify their remaining unmatched names; they do not count as fully resolved.

Dungeon matches added include Banshee, Bats, Black Widows, Ghost Dwarves/Ghost Dwarf, Ghost Miner, Shambler Miner, Bergram Stumpfinger, Ed Sullenfangs, Jake Blackforest, Ozymandias, and Taka the Shadow. The matching definitions use exact locale names where available. The regional crosswalk additionally links the corresponding exact NPC rows for Cakeling/Cakenstein, Cray Marauder, Grave Elemental, Mac, Pumpkin Prince, Chugawump, Counselor Edmond, Necronomicus, Hogosh, Jarvus, Zandro Rageracer, One/Two of Three, Stompenmugg, Tormented Spirit, Queen Verda, Collin, Mudshell, and other exact-name definitions.

### Client model facts and captured tuples

`Models.txt` SHA256 `0bad5a154c3ef855744349eb03eb3ab02e7d8dfc3d29fd0f601257fce95076be` contains 3,743 rows. Each definition's numeric model id maps to the ADR/AGR filename recorded with its source line in `model_asset_bindings`. `catalog.tsv` SHA256 `12bfd0f81236ffc02e6a9a661e1ffb14890a2bd4ac2306c14b20cbe445b1c9` contains 53 of the 55 selected files. The two missing extracted assets are model 3933 `gloam_m_construct.adr` and model 3956 `gloam_m_boss.adr`; their Model.txt rows are present, so the numeric mapping is known, but their asset bodies remain uninspected. The previous four model-id field gaps are closed by the client model table; the two missing catalog payloads remain explicit asset gaps.

Five hostile AddNpc tuples are recorded as client facts:

| Name string id | Model id | Alias | Actor scale | Capture evidence |
|---:|---:|---|---:|---|
| 441457 | 4201 | empty | 1.0 | `client-baseline-20261008T073831Z/frames/002418.bin`, SHA256 `0842ff7e012002cb308e0fbce46a181497b7d0062d7e9ce1388127177fe6271a` |
| 5100344 | 3897 | empty | 1.0 | `client-baseline-20261008T193318Z/frames/008521.bin`, SHA256 `cafc42c34472b9afc0a3945787fc68011604a9367c30b493a0f95e1f90d6f21e` |
| 5101505 | 3956 | empty | 1.0 | `client-baseline-20261008T073831Z/frames/002425.bin`, SHA256 `d472fff8d5ee353ce31a057f207c6acb5e35665ea6b3506e8d404db96f2f51e2` |
| 432135 | 3311 | empty | 1.0 | `rust-gateway-reference/0010-ready.bin`, SHA256 `ecdb77aa3b6ec56b9afcf843c18face8a7834fb81ff026cdabbeafab797720f4` |
| 432094 | 3296 | empty | 1.0 | `rust-gateway-reference/0011-ready.bin`, SHA256 `e08629bf1990d55333922115c2d53b9967e327a349ac2fb237c56bc61d8c252b` |

The prepared reference manifest ties the last two messages to original capture directory `client-baseline-20261008T020205Z`, capture-manifest SHA256 `f731d2ae67bbfc25cd46f390a202eeaa8d14ba63da42a48e80b831ed31081ebf`, and packet-analysis SHA256 `348ce44a0362d5ad4d413d239cc07f2e548dcf23d2fa71df4b986b24a6fb8ae7`. These captures establish only their AddNpc tuple fields; they do not recover combat stats or AI.

Selected ADRs were read with the unmodified FR ADR Toolkit revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72`. For example, Banshee model 5 has a banshee skeleton and mesh, while toy model 4677 has no skeleton; Bats model 28 is marked `Combat NPC` and uses a bat skeleton, while model 3848 is explicitly a bat mount; Cakeling model 152 and Cakenstein model 22 have actor skeleton/mesh closures, unlike their toy or exploded-piece candidates. Model 10's ghost-dwarf miner body supports the Ghost Dwarf/Miner family. AGR character files were mapped by Models.txt and the asset catalog; the ADR reader was not applied to AGR.

Locale source SHA256 is `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`. Exact locale ids and duplicate-name candidates are retained per definition in `definition_field_sources` and `name_model_candidates`. The read-only Legacy NPC table was used only for exact NameId/model candidate joins, then checked against Models.txt and the client asset catalog; it was not copied as source code.

Pinned wiki database SHA256 is `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911`. Relevant revisions include Banshee 50200 (`ac7b93e51222f3e2288e5c62428b69ad710cdd4561193a6af694dfde5652d222`), Black Widow 48237 (`6b95d632b73ca12d125b5f792d5fc56fcbc601fec41d3369f9bea5352b35bd57`), Cakeling 49041 (`14c42ad7f51f04fab6f4e033d19424fed508f8703b4af29c26285464af2d95a3`), Cakenstein 49987 (`d3dcbf5d95677575871b5e7cb30c80749c31f17ace294f89ac4ce6c084302f22`), Bergram Stumpfinger 49191 (`894f23a85153891fb7409bad6dea4b2a0fff48baeb0b39598419005acfc78ece`), Mudshell 30403 (`e41af3ab81eb54bdd05192182365b7ec1fd165123861c5cc075debec6c8d92fb`), Queen Verda 50205 (`516977ef79f0dcf68b37c381adad0477d5069ce8d4e2f5f95a87b0c95a740f2d`), Collin 46624 (`54deec77ad95798dedc5c7892afe783df091ba65fbe1e64f2dadef8006f94ec7`), and One/Two of Three 50193/50194 (`ed4b3db7e1218d072a6394dccca2d55b7bb55b4e9de304cfb45f754c0402a9a1`, `1d9ee3146e4e4a5497bc7861c406c5b4dfa859754d2fae6e3e79d98c467bb0`).

### Ambiguity and reconstructed policy

Ambiguous candidates remain unresolved in the sidecar: generic Cray; Tarantula versus baby Tarantula; three Necrowart Zombie ADR variants; Frostfang Bat variants; Raven's raven/harpy forms; Snowface's ice tree-soldier file with two numeric model ids; and generic troll candidates for General Grexan. Whirly Willis remains unresolved as a hostile actor because the pinned page describes a helpful Robgoblin supplier; a coarse regional `enemy` role does not prove disposition.

All new levels are null. Health, damage, speed, aggro/leash, attack range/timing, respawn, boss combat policy, and rewards remain reconstructed (R) defaults; no original per-enemy balance or loot table was recovered. Actor scale comes from AddNpc only for the five captured tuples; ADR model scale is kept separately and is not treated as runtime actor scale. The native game was not launched for this scoped task.

### W58b verification

- Read-only asset audit checked every definition's numeric id against Models.txt and catalog presence; selected ADR body closures use the pinned toolkit.
- Crosswalk audit recalculates 74/367 dungeon occurrences and 46/250 unique names resolved, plus 36/82 regional rows fully resolved and 3 partial rows.
- `rustfmt --edition 2024 tests/content_enemies.rs` exited 0. With `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W58b`, `scripts/cargo.ps1 test --test content_enemies` passed 2/2 tests. The `-- --nocapture` run printed the 45/367 and 32/250 to 74/367 and 46/250 dungeon change, 0/82 to 36/82 regional change (3 partial), and 55/55 Models.txt plus 53/55 catalog pairs. Compilation emitted seven existing dead-code warnings in unowned chess, housing, and piece_074 sources; no errors.
