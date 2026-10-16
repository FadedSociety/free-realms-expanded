# Content Data Model

Status: W12 schema and loader candidate, schema version 1. The loader and fixture tests verify the data shape and cross-file integrity; they do not verify gameplay or original-server policy.

## Scope and sources

`notes/Design/Build Ledger.md` item 5 identifies the missing general content model: the Content tree has 17 job pages, 74 dungeon pages, and 17 regional NPC pages. The content index sets provenance precedence to client, then video, then wiki. The gameplay data sections add an important boundary: the client does not ship unified quest, reward, item, dungeon, or NPC gameplay definitions. The pages are source-audited inventories, not proof that a server rule was recovered.

Reviewed sources:

- `notes/Build Guides/Content/Content.md` and representative pages `Jobs/Archer.md`, `Dungeons/Robgoblin Creek.md`, and `NPCs/NPCs - Highroad Junction.md`.
- `notes/Build Guides/Build Guides.md`, the feature-to-guide index and specification entry point.
- Data and formats sections of `Quests and Objectives.md`, `Items and Inventory.md`, `Combat and Abilities.md`, `Minigame Framework.md`, and `Rewards and Collections.md`.
- Typed startup rows in `src/startup_catalogues.rs`, `src/startup_messages.rs`, and `src/startup_world.rs`.
- `notes/Research/Startup World Content.md`, which records the startup capture export and native readers.

## Worker workflow

- Read `RTK.md`; normal PowerShell and `rg` output was usable, so no `rtk proxy` retry was needed.
- Applied Ponytail by keeping serde/serde_json dependencies, avoiding manifest or shared-module edits, and treating the startup structs as wire rows rather than duplicating their parser logic.
- Applied Caveman in progress updates while keeping this evidence note explicit.
- `game-recon`, `game-research-websearch`, `reverse-engineering`, and `asset-pipeline` were not needed: this task defined a validated data model from the existing guides and captured typed rows; it did not inspect a game install, recover new binary behavior, or parse/convert assets.

Recovered client facts used for alignment:

- `startup_catalogues::ItemDefinition` is only `{ id: i32, serialized_record: &[u8] }` (`src/startup_catalogues.rs`, `ItemDefinition`). It is a borrowed packet-builder input, not a useful JSON item record. The content `Item` preserves the compatible signed ID and adds locale/category fields from the item guide; it does not claim to decode the opaque runtime record.
- `startup_messages::ZoneDetails` carries name, zone type, tutorial flag, sky, zone ID, and geometry ID (`src/startup_messages.rs`, `ZoneDetails`). `Zone` uses these names where known and leaves unresolved values optional.
- Startup NPC message 35/2 uses reader `PlayerUpdatePacketAddNpc_ReadBody` at `00921a10`; the Rust wire row is `startup_world::NpcSpawn`. The reader confirms the field sequence and types, including actor ID, name/model IDs, float4 position and rotation, and scale. The content transform keeps xyz plus the raw fourth position value; its meaning, role, and quest associations are not proven by that reader (`notes/Research/Startup World Content.md`, “NPC spawns, 35/2”).
- Startup POI opcode 57 uses `PoiDefinitionRow_ReadFields` at `00b98290`; the reader proves the row widths/order, not the meaning of every column (`notes/Research/Startup World Content.md`, “POIs, opcode 57”). The Rust `startup_world::PointOfInterest` is the current packet-facing crosswalk; content `zone_id` is a normalized relationship, not an assertion that captured `location_id` means zone ID.
- The saved capture export `../lab/reports/worldcontent-20261013T203151Z/world-content.json` contains `npc_spawns[0]` (actor `2732686529148769137`, model `3311`, name string `432135`, position `[-2136.071,-32.9525,1038.784,1.0]`, rotation `[-0.02317995,0,0.9997313,0]`, scale `1.0`) and `points_of_interest[0]` (ID `3`, name string `3961`, location `281`, position `[51.391,30.987,363.156]`, heading `1.487`, icon `1345`). These are captured startup values. Fixture `zone_id`, NPC role, and synthetic quest linkage are separately marked `(R)`.

## Version 1 files and records

`src/content.rs` defines strict serde records. Each required file is an envelope with `schema_version: 1` and `records: [...]`; unknown envelope/record fields fail decoding. The required files are `zones.json`, `npcs.json`, `pois.json`, `items.json`, `quests.json`, `jobs.json`, `dungeons.json`, and `minigames.json`.

| File / record | Fields and references |
|---|---|
| Zone | Stable `id`, name, optional geometry/type/tutorial/sky fields. Startup `ZoneDetails` mapping. |
| NPC spawn | `actor_id` key, `zone_id`, `model_id`, `name_string_id`, `transform`, one or more `role` values (`vendor`, `quest_giver`, `ambient`), and `quest_ids`. |
| POI | `id`, normalized `zone_id`, `name_string_id`, `transform`, optional icon, and optional opcode 57 crosswalk fields (`location_id`, notification/sub-name IDs, breadcrumb/teleport IDs, atlas name, and raw unknown dwords). `Rotation` preserves either a heading or quaternion without converting between them. |
| Item | Signed definition `id`, locale name/description IDs, optional category. Runtime item bytes remain outside this content schema. |
| Quest | `id`, optional title locale ID, required `giver_npc_id`, prerequisite quest IDs, typed goals/objectives, reward grants. |
| Job | Stable slug `id`, optional client profile ID, display name, combat/crafting/freestyle category, max level, members flag, trainer NPC IDs, quest IDs, and levelled ability IDs. |
| Dungeon / encounter | Dungeon slug, optional name/zone, typed entry requirements, nested encounters, goals, and rewards. Encounter IDs are scoped to their dungeon. |
| Minigame row | Row ID, client game type ID, optional name ID and linked dungeon, rewards. This records a catalogue link; it does not implement the minigame lifecycle. |

All top-level records and nested source-bearing records require a non-empty `source` list. Each tag is `{ "kind": "client" | "video" | "wiki" | "reconstructed", "evidence": "..." }`. Evidence must be non-empty; `reconstructed` evidence must start with `(R)`. A row may carry multiple tags when different fields have different evidence. A missing tag, invalid tag, unsupported schema version, duplicate ID, or unresolved reference returns a typed error naming the file/record/field/target.

The loader checks unique IDs within each dataset; NPC/POI/dungeon zone links; NPC quest links; quest giver and prerequisite links; objective NPC/item/POI/zone links; job trainer/quest links; dungeon entry requirement job/quest links; item rewards; and minigame-to-dungeon links. These are (R) content-package integrity rules. They are not recovered server authorization, quest, or payout rules.

`RewardGrant` carries coins, item rows (`definition_id`, count), and job XP keyed by client profile ID, matching the data fields of `src/rewards.rs::RewardGrant` and `ItemGrant` after an adapter supplies runtime display context. Job XP IDs must resolve to a job's `profile_id`; item rows must resolve to item definitions. The client guide recovers reward bundle layout and display consumers, but not historical payout policy or complete reward tables.

## Conversion batches

1. Convert known startup zone, NPC, and POI rows first, keeping native values and unresolved mappings tagged separately. Then convert the 17 regional NPC pages in small region batches; do not turn a name-only row into a verified placement or service.
2. Convert the 17 job pages by combat, crafting, and freestyle families. Create quest rows alongside the job rows only when the page supplies a stable ID and a source-backed giver/prerequisite/objective relationship. Do not join locale strings by wording without an `(R)` tag.
3. Convert the 74 dungeon pages by region/type. Keep entry restrictions, goals, enemy counts, payout, and prize-wheel values omitted or explicitly `(R)` when the page records a gap or wiki-only claim.
4. Convert minigame catalogue rows independently from activity behavior. Validate each batch with `ContentDb::load`, retain the exact source references, and report unresolved joins instead of inventing IDs.

## Limits and integration

- The fixture combines synthetic rows with two parsed startup rows; it is a schema/loader check, not complete content extraction.
- Startup NPC capture proves wire fields, not the content role or quest link. The fixture labels those additions `(R)`. Startup POI `location_id` semantics remain unresolved; fixture zone mapping is `(R)`.
- Job abilities currently store only IDs and levels. They are not complete ability definitions, combat behavior, or balance.
- The minigame type ID is a client catalogue field. This model does not recreate the 783-row registry, matchmaking, launch protocol, activity state machine, or results processing.
- Content source tags do not certify game behavior. The schema is not registered in shared `lib.rs` and has no production consumer yet; lead integration and gameplay/native gates remain separate.
