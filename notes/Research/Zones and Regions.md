---
updated: 2026-10-26
status: research
---

# Zones and Regions

Research for T749 (2026-10-26), no code change. Scope: find the client's real zone and
region identifiers, the world's region bounds, the "Now entering" banner and chat trigger,
and a position -> region rule, then give an ordered build plan. The commit is this note.
**F752 (review fix) corrected this note:** the 15/2 header is `u16 15, u16 2` (not a `u8`
subtype), the live NPC path already spawns every drawn NPC in zone 0, the group counts are
the shipped T753 590-sphere grouping and the 707-vs-697 difference is the 10 box rows, the
10 m grid is dropped, and the 15/2 display frame is cited. Evidence is in
`notes/Autopilot/F752.md`.

Evidence labels used below: **client** = first-party 2014 client code/data; **(R)** =
reconstructed or inferred by this research; **community** = a pinned community project,
never used over client evidence. All raw client data stays in the lab; this note only
quotes small values and paths.

## Answer in one page

1. **The client zones by scene name, not by a numeric zone id.** The 2014 client ships
   174 `*.gzne` zone definitions in its pack catalogue. The scene name in
   `PacketClientBeginZoning` (`opcode 31`) is the `.gzne` stem, e.g. `FabledRealms`,
   `sg_kt_track`, `hsg_emptylot_seaside_beach_01`. The client has no numeric scene-id
   table. The packet's `type`, instance `id` and `geometry id` are separate fields; the
   `geometry id` is used by the client to select a **journal region**, not a scene.
2. **The current server tables are not client zone tables.** `data/content/zones.json`
   has 17 rows whose ids 1..17 are reconstructed *package/region labels* from the regional
   NPC guides; the rows are not `.gzne` scene names and the ids are not client region ids.
   The open world is zone **0** (`world::OPEN_WORLD_ZONE`, `dev_teleport::WORLD_ZONE`) and
   the captured world scene is **`FabledRealms`** with `geometry id 2`. `encounters.json`
   currently gives the world `geometry_id: 214` and the one dungeon scene also 214, which
   contradicts the capture; that is a data defect, not a client table.
3. **The client has ten region ids used by the banner and journal:** 1 = default/starting
   region (the only `RegionOverride` in `FabledRealmsAreas.xml` is `Newbiezone`, id 1), and
   2..10 = Blackspore, Briarwood, Merry Vale, Sanctuary, Seaside, Shrouded Glade, Snowhill,
   Wugachug, Sunstone Valley. The Lua banner table in `ScriptsBase.lua` fixes the names and
   the `*_notification.swf` movies. Client numeric name ids (locale `ucdt`) are recovered:
   3500 Blackspore, 18140 Briarwood, 4428 Merry Vale, 3499 Sanctuary, 3501 Seaside,
   3502 Shrouded Glade, 3961 Snowhill, 3505 Wugachug, 3793 Sunstone Valley, 20776 The
   Wilds, 18899 Lakeshore, 18142 Cobblestone Village, 428995 Greenwood Forest,
   103715 Highroad Junction, 103701 Stillwater Crossing, 18735 Farnum Farmstead,
   7262 Queensfields, 4029 Briarheart.
4. **The on-screen banner is client-side presentation triggered by a server UI packet.**
   Client `BaseUiPacketDispatch` (`0x00a91bf0`) case **family 47, subtype 9** calls Lua
   `NotificationHandler:LoadingZone(zoneId)`, which plays
   `playZoneAnimation(Ui.GetString("note_nowEntering"), t[zoneId] .. ".swf", "0xCCCCFF")`
   and sound 16125. The packet reader (`0x00a8da50` -> `0x00a8b360`) reads two
   length-prefixed strings; the first is the zone id passed to the Lua function. The client
   does not derive the banner from position by itself, except through the optional
   `CurrentPOI` path (below).
5. **The "Now entering <area>" chat line is a different server packet:** client
   `BaseChatPacket` **family 15, subtype 2** (`ChatPacketEnterArea`). The receive
   dispatcher `0x00b14350` first calls the shared base reader `0x00b0f550`, which consumes
   **two `u16` header words** (family, subtype), then case 2 calls `0x00b109b0` ->
   `0x00b10580`, which consumes one `u32` area id. The display path `0x00b10250` looks up
   the locale key `EnteringChatArea` and formats the client string **numeric id 3604**,
   `Now entering [*area*].`, binding the packet value as the `area` parameter. The packet
   carries an **area name string id**, not a region id. A `u8` subtype is wrong: the
   reader's second header word is a `u16`.
6. **The banner region bounds can be computed two ways:** the recovered nine-region
   `AreaDefinitions` table already in `piece_076` (community `FabledRealms.json`: four
   rectangles and five circles) and the client's own `FabledRealmsAreas.xml` area spheres
   in 19 shipped place groups plus named beds. The recovered shapes are the coarse banner
   regions; the first-party area groups are the place bounds and a refinement. The note
   gives both, their overlaps and gaps. The combined rule covers 1091/1209 NPC rows
   (90.2%); 118 rows remain unplaced, grouped below.
7. **Build order:** (a) one `regions.json` + loader + `RegionTable` lookup, **shipped by
   T753**, (a2) role-based NPC placement on the accepted map fits (after the region table;
   only Seaside currently qualifies), (b) a data pass giving every location row `zone_id 0`
   (world) or instance zone + `region_id` + `place_id`, (c) teleport/`realm goto` through
   the same lookup, (d) the 47/9 banner and 15/2 chat line on change, (e) dungeon zone ids.
   Each step's acceptance test and native check is in the build plan. The existing 17-row
   `zones.json` should stop being loaded as `Zone`; the world `Zone` is 0/FabledRealms and
   instance zones are `.gzne` scenes.

## 1. What the client calls a zone

### 1.1 Scene list: 174 `.gzne` definitions

The client's zone definition file is a `GZNE` file, named by the scene stem and carried in
the pack catalogue `lab/extracted/client-assets/catalog.tsv` (columns `pack`, `entry`,
`name`, `offset`, `bytes`, `crc32`, `extracted_relative_path`). The world scene and every
captured transition scene resolve to a `.gzne` entry:

| Scene (BeginZoning name) | `.gzne` entry | Captured numeric fields |
| --- | --- | --- |
| `FabledRealms` | `Assets_004.pack` entry 6041 (`pack_16/006041.gzne`, 29,181 B) | `id 0`, `geometry 2` (piece_078 capture) |
| `sg_random_encounter_clearing` | `Assets_009.pack` entry 3215 (`pack_21/003215.gzne`, 2,944 B) | `id 0`, `geometry 96` |
| `sg_kt_track` | `Assets_001.pack` entry 3217 (`pack_13/003217.gzne`, 342 B) | `id 0`, `geometry 101`, `type 2` |
| `hsg_emptylot_seaside_beach_01` | `Assets_009.pack` entry 2672 (`pack_21/002672.gzne`, 2,096 B) | `id 1`, `geometry 214` |
| `sg_npx_cavern_01` | `Assets_001.pack` entry 1613 (`pack_01/001613.gzne`, 2,428 B) | content `zone 411`, `geometry 214` (not captured) |

The `.gzne` header is `GZNE` version 2 or 3. It has no scene-name field and no numeric
scene id; the name is the pack entry name. The header-only stubs are 84/56 bytes; they are
not usable zones (e.g. `sg_haunted_mines.gzne`, `mv_den_of_secrets.gzne`,
`pv_palescale_pit.gzne`).

The full 174-scene list follows; kind is our grouping for this note, not a client enum.
The same stem can serve more than one activity (e.g. `sg_soccer`).

**World / region scenes (2).** `FabledRealms`, `sunstone_valley`.

**Housing, lots and farms (41).** `briarwood_party_zone`, `farming_briarwood_farmstead_01`,
`farming_prototype`, `farming_wilds_farmstead_01`, `farming_wilds_farmstead_02`,
`hsg_emptylot_bixie_field_01`, `hsg_emptylot_blackspore_01`, `hsg_emptylot_boat_01`,
`hsg_emptylot_bog_shore_01`, `hsg_emptylot_briarfalls_01`, `hsg_emptylot_briarwood_01`,
`hsg_emptylot_bw_towers_01`, `hsg_emptylot_castle_01`, `hsg_emptylot_gl_staircase_01`,
`hsg_emptylot_gl_valley_01`, `hsg_emptylot_island_01`, `hsg_emptylot_laketree_01`,
`hsg_emptylot_merryvale_01`, `hsg_emptylot_seaside_beach_01`,
`hsg_emptylot_seaside_cliffs_01`, `hsg_emptylot_sh_batcave_01`,
`hsg_emptylot_sh_crystalmines_01`, `hsg_emptylot_shrouded_gloam_01`,
`hsg_emptylot_snowdays_01`, `hsg_emptylot_snowhill_01`, `hsg_emptylot_vale_steam_01`,
`hsg_emptylot_wilds_halflot_01`, `hsg_emptylot_wilds_rapids_01`,
`hsg_emptylot_wugachug_01`, `hsg_hum_blackspore_01`, `hsg_hum_chromakey_01`,
`hsg_hum_condo`, `hsg_hum_condo_w_lot`, `hsg_hum_deluxe_01`, `hsg_hum_economy_01`,
`hsg_hum_economy_decorated_01`, `hsg_hum_seaside_beach_house_01`,
`hsg_hum_sg_night_club_01`, `hsg_hum_snowhill_01`, `hsg_hum_xmas_01`,
`ranching_wilds_ranch_01`, `ssv_emptylot_wilds_01`, `ssv_rumbledome_lot_01`.

**Kart / racing (10).** `bw_kt_track`, `gw_racetrack_01`, `sg_kt_track`,
`sg_kt_track2`, `sg_racing_garage`, `sh_kt_track`, `ss_kt_track`, `ss_pro_track`,
`ss_pro_track2`, `test_designtrack_bbe`.

**Soccer (3).** `bw_soccer`, `sg_soccer`, `sh_soccer`.

**Arena / combat instances (10).** `bw_dm_arena`, `gw_rumbledome`, `sg_dm_arena`,
`sg_dm_arena2`, `sg_mongo_combat`, `sg_newbiezone_showdown`, `sh_dm_arena`,
`sh_iso_arena`, `sh_snowball_battle`, `ss_pirate_arena2`.

**Random / quest encounters (23).** `bs_random_encounter_01`,
`bw_random_encounter_01`, `bw_random_encounter_02`, `bw_random_encounter_03`,
`bw_random_encounter_bristlewood_01`, `bw_random_encounter_thistlerow_01`,
`gw_wandering_encounter_01`, `prototype_encounter_01`, `prototype_encounter_02`,
`pv_wandering_encounter_01`, `sg_chef_encounter`, `sg_demo_derby_encounter`,
`sg_miner_encounter`, `sg_pet_trainer_encounter`, `sg_pet_trainer_encounter_02`,
`sg_random_encounter_02`, `sg_random_encounter_clearing`,
`sg_random_encounter_skullcamp`, `sg_random_encounter_treefort`,
`sg_wizard_encounter`, `sh_random_encounter_01`,
`sh_winter_holiday_encounter_01`, `ss_random_encounter_01`.

**Interior / shop (9).** `bw_briarheart_castle_interior`, `sg_ayani_interior`,
`sg_dressing_room`, `sg_shop_combatshop_01`, `sg_shop_fashionshop_01`,
`sg_shop_interior_01`, `sg_shop_salon_01`, `sg_shop_shoeshop_01`, `sg_tavern_cellar`.

**Test / starting / prototype (12).** `CombatTest`, `fast_load_zone`, `hsg_test`,
`hsg_test_interior`, `sg_housing_test02`, `sh_housing_test`, `starting_area`,
`starting_area_dell`, `starting_dell`, `test_fishing_bbe`, `test_pirates`,
`obstical_course_test_01`.

**Adventure / dungeon / fishing / maze / cave (64).** `bs_bone_bog_cemetery`,
`bw_briarheart_caverns`, `bw_corrupted_vale`, `bw_fishing_stream`, `bw_hedgemaze`,
`bw_hedgemaze_return`, `bw_hidden_valley`, `bw_mo_mushrooms`, `bw_mushroom_cave`,
`bw_spider_lair`, `bw_tanglewood_fort`, `bw_trail_of_betrayal`, `bw_treeleaf_retreat`,
`bw_vale_of_thorns`, `bw_vine_grotto`, `gl_misty_mountain`, `gw_buried_temple`,
`gw_rocky_ridge_road`, `mv_den_of_secrets`, `mv_necronomicus`, `pv_palescale_pit`,
`sea_side_test_area`, `sea_side_test_area_02`, `seaside_platform_jumping`,
`sg_bandit_hideout`, `sg_bear_cave_combat`, `sg_bixie_hive`, `sg_changeling_caverns`,
`sg_changeling_caverns_return`, `sg_cliffs_of_insanity`, `sg_coin_cloud`,
`sg_coin_cloud_climb`, `sg_crossroads_pet_tutorial`, `sg_fishing_stream`,
`sg_floren_forest`, `sg_haunted_mines`, `sg_haunted_mines_150`, `sg_highroad_hijinx`,
`sg_highroad_junction_pet_tutorial`, `sg_mugworts_hollow`, `sg_npx_cavern_01`,
`sg_robgoblin_trove`, `sg_sheep_watch`, `sg_stillwater_crossing_combat`,
`sg_stillwater_crossing_pet_tutorial`, `sg_treasure_vault_free`,
`sg_treasure_vault_premium`, `sg_treeble_dell`, `sh_cable_guy`, `sh_deep_mines`,
`sh_fishing_medpond`, `sh_frostfang_cavern`, `sh_platfrom_holiday_instance`,
`sh_yeti_cave`, `shrouded_glade_gloam`, `ss_cray_caves`, `ss_nekark_cave`,
`ssv_dwarf_dam_01`, `wc_invasion`, `wc_party_crashers`, `wc_sweetwater_climb`,
`winter_holliday_snowhill_2`.

The dungeon subset with a recovered zone definition is the piece_059 table
(`notes/Research/Pieces/059 Dungeon definitions.md`, 14 real `.gzne` entries plus the
84-byte `mv_den_of_secrets` stub). 29 of the 43 wiki dungeons have no `.gzne`; they cannot
be a scene transition yet.

### 1.2 Numeric ids that do exist

`PacketClientBeginZoning` reader `FUN_008e4ae0` (pseudocode line 1029104; opcode case
`0x1f` at line 1108664) reads, in order: `u16 opcode`, scene name, `i32 type`, four `f32`
position, four `f32` rotation, sky name, `u8 tutorial`, `u8 unknown`, `i32 id`, `f32
update radius`, `i32 geometry id`, `u8 override update radius`, `u8 wait for zone ready`.
The client's own log line in the handler is `"Loading zone %s type %d ..."` in
`FUN_009386f0`; it passes the scene name and type to `FUN_00790520`, while
`FUN_008a7840` maps type `0..3` to the config keys `TileStatic`, `TileSeamless`,
`RuntimeSeamless`, `Mesh`.

- **Scene name** is the only scene key. There is no client table `id -> scene`.
- **`type`** is the zone type, not a scene id; the captured `sg_kt_track` uses 2
  (`RuntimeSeamless`).
- **`id`** is an instance/zone-transition id, not a type and not unique: the captured
  `FabledRealms` and `sg_random_encounter_clearing` both use 0, the housing lot uses 1
  (piece_078). Do not use it as a zone type.
- **`geometry id`** is not a scene id. `FUN_008a78d0(client, geometry_id)` stores it at
  `client+0x194` and calls `FUN_00a44b70`, which walks the journal region list and selects
  the row whose field `+0x14` equals the geometry id; if none matches it selects region 1.
  The client constructor also loads a default `GeometryId` from its config
  (`FUN_00910880(&DAT_0180589c, "GeometryId", ...)` at pseudocode line 1091144). The
  geometry id is therefore a **journal region selection key / server geometry value**,
  not a scene index. The captured values are 2 (world), 96 (clearing), 101 (kart), 214
  (housing). The current content gives both the world and `sg_npx_cavern_01` 214; the
  captured world value is 2, so the content is suspect and must be checked with the client,
  not copied.
- **`ObjectTerrainData.xml`** (client `Resources/ObjectTerrainData.xml`, 34 rows) is the
  client's terrain/sky/minimap table: `Id` 1 Snowhill, 2 Briarwood, 3 Wilds, 4 JewelCave,
  6 Seaside, 7 MemorialCaverns, ..., 71 Wugachug, 75 KartCaverns, 83 SunstoneValley,
  84 ShroudedGloamLot, 86 NPXCavern, 87 DenOfSecrets. These are **terrain data ids**, not
  zone or scene ids; they are useful for terrain/sky/minimap and should not be loaded as
  `Zone.id`.

### 1.3 The 17-row `zones.json` is a place/region table, not a zone table

`data/content/zones.json` rows 1..17 are named after the regional NPC guide titles:
Blackspore Swamp, Briarwood, Cobblestone Village, Durango Zoo, Farnum Farmstead,
Greenwood Forest, Highroad Junction, Lakeshore, Merry Vale, Sanctuary, Seaside, Shrouded
Glade, Snowhill, Stillwater Crossing, Sunstone Valley, Wilds, Wugachug. Every row says
`(R) zone package id ... no native numeric zone id or geometry mapping`. The client
`FabledRealmsAreas.xml` uses those names as **areas/places**, not `.gzne` scenes; only
`FabledRealms` (world) and the instance scenes are `.gzne` names. Live
`NpcVisibility::load` (`world.rs`) reads `zones.json` only to build its nearest-anchor
list; it inserts **every drawn regional spawn into `OPEN_WORLD_ZONE` (zone 0)** and every
drawn extra spawn likewise, and the computed package/region id is discarded for extras
(`let _ = zone_id`). `World::runtime` passes an empty `ContentDb`, and `World::new`'s
`npcs_by_zone` index is used only by tests, so the package ids never select a live NPC
zone and there is no live package-id/zone mismatch. The 17 ids are still not client region
ids. The crosswalk (`(R)` by name/position) is:

| package id | `zones.json` name | client region id / place |
| ---: | --- | --- |
| 1 | Blackspore Swamp | region 2, place group `blackspore` |
| 2 | Briarwood | region 3, place group `briarwood` |
| 3 | Cobblestone Village | region 1 (inside `Newbiezone`), place id 18142 |
| 4 | Durango Zoo | **unresolved**; no client area/bounds |
| 5 | Farnum Farmstead | region 1/default (Wilds), place id 18735 |
| 6 | Greenwood Forest | region 1/default (Wilds), place id 428995; bounds reconstructed from NPC cluster |
| 7 | Highroad Junction | region 1/default (Wilds), place id 103715 |
| 8 | Lakeshore | region 1/default (Wilds), place id 18899 |
| 9 | Merry Vale | region 4, place group `merryvale` |
| 10 | Sanctuary | region 5, place group `sanctuary` |
| 11 | Seaside | region 6, place group `seaside` |
| 12 | Shrouded Glade | region 7, place group `shroudedglade` |
| 13 | Snowhill | region 8, place group `snowhill` |
| 14 | Stillwater Crossing | region 1/default (Wilds), place id 103701 |
| 15 | Sunstone Valley | region 10, place group `Sunstone Valley` (desert day/night beds) |
| 16 | Wilds | region 1/default; no banner |
| 17 | Wugachug | region 9, place group `wugachug` |

Client `Resources/StartingZoneLoad.txt` is a UI asset-load order (`UiAssets.txt`,
`HighRoadAssets.txt`, `MinigameAssets.txt`, `FilteredCommonAssets.txt`), not a scene
table; the `ClientStartingZoneLoader` class loads that file list. `ObjectTerrainData.xml`
is the terrain/sky/minimap table. We found no client `Zones.xml` or numeric scene-id
table in `Resources`; the pack `.gzne` catalogue is the scene table. (Scanned
`Resources`, the extracted named asset tree, the catalogue and the native strings.)

## 2. The client's region ids

The client has a first-party region table in two places:

1. `FabledRealmsAreas.xml`: one `AdvJournal` property with `RegionOverride="1"` on the
   `Newbiezone` area (`id 6093`, sphere `(-2057, 430) r415`). `ClientAreaAdvJournalProperty`
   reads `RegionOverride` (`0x00ceb810`), applies it through `FUN_00a44b10`, and removes it
   through `FUN_00a44b30`; `FUN_00a44b10` sets the region id and reference count. This is
   the only region override in the world area table; every other position falls back to
   the selected/default region.
2. `ScriptsBase.lua` `NotificationHandler:LoadingZone(zoneId)`
   (`lab/decompiled/client-ui/ScriptsBase.lua:11642-11664`) maps zone ids to banner movies:

| zone id | client name | client name numeric id (`ucdt`) | banner movie |
| --- | --- | --- | --- |
| 1 | default / `Newbiezone` | (no banner) | none |
| 2 | Blackspore | 3500 | `blackspore_notification.swf` |
| 3 | Briarwood | 18140 | `briarwood_notification.swf` |
| 4 | Merry Vale | 4428 | `merryvale_notification.swf` |
| 5 | Sanctuary | 3499 | `sanctuary_notification.swf` |
| 6 | Seaside | 3501 | `seaside_notification.swf` |
| 7 | Shrouded Glade | 3502 | `shroudedglade_notification.swf` |
| 8 | Snowhill | 3961 | `snowhill_notification.swf` |
| 9 | Wugachug | 3505 | `wugachug_notification.swf` |
| 10 | Sunstone Valley | 3793 | `sunstone_notification.swf` |

The numeric ids come from `lab/reports/locale-id-map.json` / `locale-id-to-text.json`
(hash -> `Global.Text.<id>` -> text). The `ucdt` records are the UI/client data strings;
the `ugdt` records with the same text are not addressable by a numeric id in the recovered
map and are not used here. `Now entering [*area*].` is numeric id **3604**; the banner title
`Now Entering` / `note_nowEntering` is numeric id **406408**.

Client place names and numeric ids recovered the same way (useful for 15/2 and data
passes): Lakeshore 18899, Cobblestone Village 18142, Greenwood Forest 428995, Highroad
Junction 103715, Stillwater Crossing 103701, Farnum Farmstead 18735, Queensfields 7262,
Briarheart 4029, The Wilds 20776, Snowhill Hot Springs 4807, Hot Springs Haven 30403 and
3269, Ponyvale 1646, Mystical Barrier 8802, Post Office 74174, Town Hall 74172, Hospital
74185, Adoption Center 435875, Tide Pools 93721, Seaspray Inn 93477, Boardwalk 116988,
Sasparilla Brewery 2463, Carnival 116994, Party Tent 116995, Chugaroot Farm 382811,
Ever Rainbow 116999, The Midway 94136, Pet Park 116996, Druid Academy 93702,
Gardener's Hutch 93701, The Courtyard 116998, Mire's Edge 117002, Sinking Graveyard
117003, Quagmire Quarry 74875. These place ids are `(R)` exact-text matches, not a
client crosswalk table; the region ids above are stronger.

## 3. Region bounds

### 3.1 Recovered nine-region table (community, but the cleanest banner bounds)

`src/restoration/piece_076.rs` `REGION_AREAS` is the recovered `AreaDefinitions` table of
world zone 1 from `lab/references/Sanctuary/src/Resources/Zones/FabledRealms.json` (four
rectangles, five circles):

| id | region | recovered shape |
| --- | --- | --- |
| 2 | Blackspore | rectangle `x -2820..-1680`, `z 820..1700` |
| 3 | Briarwood | circle `(-540, 2240) r700` |
| 4 | Merryvale | circle `(-1040, -404) r275` |
| 5 | Sanctuary | rectangle `x -1540..-860`, `z 130..490` |
| 6 | Seaside | rectangle `x -1162..37`, `z -1343..-993` |
| 7 | Shrouded Glades | circle `(96, 1190) r250` |
| 8 | Snowhill | circle `(380, 400) r350` |
| 9 | Wugachug | circle `(66, -396) r165` |
| 10 | Sunstone Valley | circle `(711, 2143) r765` |

This table is **community-recovered** (the zone `FabledRealms.json`), not first-party; it
is numeric reference data, not copied code. It is still the closest thing to an original
region table: the ids 2..10 and the names match the first-party Lua banner table exactly,
and its Blackspore rectangle covers the western quarry that the client `blackspore` prefix
group misses. Its `Merryvale`, `Shrouded Glades`, `Wugachug` and `Sunstone Valley` circles
match the client named beds almost exactly. The first-party client XML remains the
authority; use the recovered table for the **banner region** and the client groups for
places.

### 3.2 First-party client area groups (places and refinements)

`FabledRealmsAreas.xml` has 1,279 `AreaDefinition` rows (1,239 spheres, 40 boxes; 138
`AreaEnabled="0"`; 473 rows carry a `parentAreaId`). The exact filter used here is
**top-level** (no `parentAreaId`), **always-on** (`AreaEnabled != "0"`), **sphere**,
**radius > 0**: **697** rows. It is not "with Property children": only 692 of the 697 have
a `Property` child; the five without are `RaceTrack`, `Snowhill_YetiCaves_Exterior`,
`TheWild_smallvillage_1`, `Area937` and `Wugachug_Stage`. The lead's **707** is the same
filter **including the 10 top-level always-on box rows** (806 top-level shapes = 778
spheres + 28 boxes; 99 disabled leaves 707 = 697 spheres + 10 boxes). Pin **697 spheres /
707 shapes** in the loader test. The group counts below are the shipped T753
`regions.json` grouping: 19 place groups holding **590** of the 697 spheres; the other 107
top-level spheres have unassigned prefixes such as `mine`, `hotspring`, `butterflies` and
`pixienursery`. Group aliases (`regions.json` evidence): `thewilds*`/`thewild*`/`wilds*`
-> Wilds; `merryvale*`/`meryvale*`; `briarwood*`/`brairwood*`;
`shroudedglade*`/`shrouded*`; `stillwatercrossing*`/`stillwater*`;
`farnumfarmstead*`/`farnum*`; `queensfields*`; `memorialcaves*`/`memorialcaverns*`/
`memorial*`; `briarheart*`; `amb*`/`sunstone*`; plus the named beds `Newbiezone`,
`MerryVale`, `Shrouded_Glade` and `TheWild_WaguChugBrewery`. The earlier claim that the
per-prefix counts matched the lead's table except Seaside was false; the values below are
the measured grouping and the test must pin these, not raw prefix counts.

| group | parent banner id | shapes | AABB `(min x, min z)..(max x, max z)` | approx km² | largest shape |
| --- | --- | ---: | --- | ---: | --- |
| Blackspore | 2 | 49 | `(-2622, 814)..(-1592, 1386)` | 0.32 | `BlackSpore_Swamp_Bed_1` `(-1832, 1054) r240` |
| Briarwood | 3 | 54 | `(-1235, 1388)..(60, 2806)` | 0.88 | `Brairwood_Maze_1` `(-772, 1666) r251` |
| Briarheart | 3 | 2 top-level | `(-247, 2539)..(3, 2834)` | 0.01 | `BriarHeart_Water` `(-22, 2809) r25` |
| Merryvale | 4 | 57 | `(-1315, -761)..(-765, -129)` | 0.25 | `MerryVale` `(-1040, -404) r275` |
| Sanctuary | 5 | 36 | `(-1658, -231)..(-753, 787)` | 0.57 | `Sanctuary_waterfall_spray_01` `(-1239, 533) r250` |
| Queensfields | 5 | 4 | `(-1529, 227)..(-1363, 406)` | 0.01 | `Queensfields_butterflies_01` `(-1403, 267) r40` |
| Memorial Caves | 5 | 7 | `(-1051, 257)..(-834, 405)` | 0.02 | `MemorialCaverns_dragonflies_01` `(-906, 345) r60` |
| Seaside | 6 | 30 | `(-991, -1588)..(94, -814)` | 0.35 | `Seaside_waterfall_spray_01` `(-673, -1080) r250` |
| Shrouded Glade | 7 | 19 | `(-436, 761)..(442, 1570)` | 0.47 | `Shrouded_Glade` `(96, 1190) r250` |
| Snowhill | 8 | 27 | `(-65, 117)..(727, 644)` | 0.15 | `Snowhill_Bed_1` `(156, 390) r135` |
| Wugachug | 9 | 4 | `(-153, -773)..(263, -246)` | 0.17 | `Wugachug_waterfall_spray_01` `(47, -540) r200` |
| Sunstone Valley | 10 | 9 | `(-552, 815)..(1476, 2908)` | 3.35 | `AMB_BED_Desert_Night` `(448, 1815) r1000` |
| Wilds | 1 | 148 | `(-2249, -903)..(469, 1690)` | 1.31 | `TheWild_SkullVillage` `(-314, 1410) r280` |
| Lakeshore | 1 | 18 | `(-901, 459)..(-391, 936)` | 0.11 | `Lakeshore_MX` `(-751, 765) r150` |
| Crossroads | 1 | 17 | `(-1911, 295)..(-1740, 533)` | 0.02 | `Crossroads_butterflies_01` `(-1820, 433) r40` |
| Highroad Junction | 1 | 51 | `(-766, -380)..(-253, 704)` | 0.15 | `HighroadJunction_fireflies_10` `(-513, 227) r150` |
| Stillwater Crossing | 1 | 51 | `(-1593, 620)..(-710, 1406)` | 0.35 | `StillwaterCrossing_waterfall_spray_01` `(-994, 1156) r250` |
| Farnum Farmstead | 1 | 6 | `(-2267, 134)..(-2015, 386)` | 0.05 | `Farnum_Farmstead` `(-2141, 260) r126` |
| Newbiezone (region 1) | 1 | 1 | `(-2472, 15)..(-1642, 845)` | 0.54 | `Newbiezone` `(-2057, 430) r415`; `RegionOverride=1` |

The approximate area/coverage is a 20 m grid over the union of the group's spheres; the
AABB is the union extent, not the playable shape. The largest shape is often an ambient
sound/effect sphere (waterfall spray, fireflies), not the group's centre; use it as a
marker, not a precise centre. The Sunstone group now carries the two `AMB_BED_Desert_*`
beds plus seven `AMB_EMIT_*`/`AMB_Emit_*` ambient rows; for the banner region use the day
r765 circle `(711, 2143)` and treat the night bed and the ambient rows as
places/ambience, not the region bound.

`Greenwood Forest` and `Cobblestone Village` have **no** first-party `FabledRealmsAreas.xml`
prefix group. They exist in content (`npcs/greenwood_forest.json`,
`npcs/cobblestone_village.json`), in `zones.json`, and in the client locale; their place
bounds must be reconstructed from the NPC/POI positions (R) or left unresolved.
`Durango Zoo` has no client area and only unanchored/zero positions; it is unresolved.

### 3.3 Overlaps and gaps

Using a 20 m grid over the union AABB, the client group union covers 40.3% of the raw
AABB; 59.7% of the raw AABB is outside every group, but most of that is ocean/empty map
edge, not playable gap. The meaningful measure is row coverage:

- Client area groups alone: **981/1209 NPC rows** (81.1%) fall in a group; **228** are
  outside (18.9%).
- Recovered nine regions alone: **844/1209** (69.8%) fall in a region; **365** outside.
- Union of client groups and recovered nine regions: **1091/1209** (90.2%) covered;
  **118** remain outside both. 260 rows are in more than one client group; the priority
  rule below resolves them.

Top client-group overlaps (20 m grid, approximate): Shrouded Glade/Sunstone 0.36 km²,
Briarwood/Sunstone 0.30, Sunstone/Wilds 0.27, Shrouded Glade/Wilds 0.22,
Newbiezone/Wilds 0.18, Wilds/Wugachug 0.12, Stillwater/Wilds 0.05, Farnum/Newbiezone
0.05, Highroad/Wilds 0.05, Sanctuary/Stillwater 0.03, Sanctuary/Wilds 0.03.
The big Sunstone overlaps are the r1000 night bed; priority `Sunstone last` removes them.

The 118 remaining NPC rows by content zone: Greenwood Forest 40, Wilds 29, Lakeshore 15,
none/placeholder 12, Snowhill 7, Briarwood 5, Merry Vale 4, Seaside 2, Blackspore 1,
Durango Zoo 1, Highroad Junction 1, Sanctuary 1. Representative clusters (100 m cells):
`(-1600,-600)` 14 and `(-2000,-700)` 12 (Greenwood Forest camps),
`(-800,900)` 13 (Lakeshore/Stillwater edge), `(-300,-100)` 8 (Highroad/Wilds),
`(0,2800)` 4 (Briarheart north edge), `(-400,-900)` 4 (Wilds/Pixie Nursery).
The 183 `placement: unanchored` rows are reconstruction scaffolding, not drawn spawns:
`NpcVisibility::load` draws only `placement: legacy_data` rows. Their `placement_source`
distinguishes **171 `reconstructed_zone_center`** rows (placed at a zone anchor; the
group-centre cluster) from **12 `reconstructed_unanchored_default`** rows. Only four of the
twelve are near the world origin (ids `9500000257`, `9500000258`, `9500000259` and
`9500000265` in `npcs/durango_zoo.json`, within ~4 m); the other eight carry plausible
world coordinates (Cobblestone Village, Greenwood Forest, Merry Vale, Sanctuary, Seaside).
Use `placement_source`, not `placement`, and do not drop or zero the eight.

### 3.4 Position -> region rule

1. If the point is inside a **first-party place group**, the highest-priority group wins
   (priority in data: starting area 100, banner region groups 50, named places 30,
   Wilds 10, Sunstone 5; then smaller area, then stable id). `place_id` and
   `region_id` come from that row's `region_id`.
2. Else if inside a **recovered nine-region shape**, the region id 2..10 wins; `place_id`
   is null (or the region's default place).
3. Else if inside the client `Newbiezone` sphere, region 1 wins (no banner).
4. Else `None`; the caller must not invent a region.

The shipped T753 `RegionTable::region_at`/`place_at` scans at most 19 place rows and 10
region rows per query, with no allocation; at 20 Hz per moving player this is negligible
for a small server. Do not add the proposed 10 m precomputed grid: it is a second
representation of the region data with its own reload/atomicity rules, and no profile has
shown a cost. Revisit only with a measured hot path.

## 4. The "Now entering" banner and chat line

### 4.1 On-screen banner: server 47/9 -> `NotificationHandler:LoadingZone`

- Client `BaseUiPacketDispatch` `0x00a91bf0`; jump table `0x00a92480`; case **subtype 9**
  at `0x00a92121`. It calls `NotificationHandler:LoadingZone` at `0x00a921a9`.
- **IDA correction to `notes/Research/World Object Catalogue.md`:** `0x00a92121` is the
  case arm, not the payload reader. The reader is `sub_A8DA50` -> `sub_A8B360`
  (`0x00a8b360`), and the payload is **two length-prefixed strings**, not an untraced
  region id. IDA beats the older note.
- Payload reader `sub_A8DA50` -> `sub_A8B360` (0x00a8b360); its base reader
  `sub_A8B240` (`0x00a8b240`) consumes `u16 family`, `u8 subtype`, then the payload reads
  **two** length-prefixed byte strings. The dispatcher
  pushes both strings to Lua (`sub_70C580`, which constructs a Lua string at
  `0x0070bd80`); the first is `zoneId`, the second is unused by the current Lua function.
  **`(R)` both strings:** the reader is first-party, but we did not identify a preserved
  47/9 frame (the reviewed captures are login/zone/minigame frames; a raw byte scan is not
  proof because frames are encapsulated), so the meaning/encoding of the two strings is
  inferred from the Lua call. The first must be a decimal zone id for `tonumber` to work
  (`"8"` for Snowhill); the second is unused by `LoadingZone` and is likely the zone name.
  Send the zone's client name text (`"Snowhill"`) and confirm with the native check.
- Lua `NotificationHandler:LoadingZone(zoneId)` (`ScriptsBase.lua:11642-11664`):
  `movie:Invoke("playZoneAnimation", Ui.GetString("note_nowEntering"), t[tonumber(zoneId)] .. ".swf", "0xCCCCFF")`,
  then `Ui.PlaySoundFromDefinition(16125)`. `t[2..10]` are the nine `*_notification.swf`
  names in section 2. There is no client position check here; the packet selects the
  region. An id outside 2..10 makes `t[...]` nil and appends `.swf`, so the server must
  validate.
- Footage: `lab/video-archive/out/2sLufM-Du1I/frames/00222.jpg` shows the banner
  "Now Entering" + Snowhill emblem/name; `00224.jpg` shows the same banner a moment later.
  `lab/video-archive/out/kVJBEK5sVBQ/frames/00064.jpg` shows the chat line while in
  Snowhill. (No player names are transcribed here.)

**Alternate client trigger (do not enable accidentally):** `ScriptsBase.lua:5049-5053`
`BaseClient_CurrentPOI_OnDataUpdate` reads `CurrentPOI` column 1 (`POIId`; native
`CurrentPOIDataSource::ColumnName` `0x00c8e350` maps 0/1/2 to `POIName`, `POIId`,
`AreaId`) and calls `LoadingZone(poiId)` for `1 < poiId < 11`. The original POI ids 2..10
were therefore region hubs. The preserved community POI table reuses ids 3..10 for
Snowhill, Briarwood, Sanctuary, Greenwood Forest, Cobblestone Village, Shrouded Glade,
Lakeshore and Briarheart (NameIds 3961, 18140, 3499, 428995, 18142, 3502, 18899, 4029),
which do **not** match the Lua banner mapping; it is a community table, not original
region data. The shipped server content has no `pois.json` (only `tests/fixtures`), so the
server currently sends an empty startup POI table (message 9 / opcode 57) and the
`CurrentPOI` path cannot fire in production. Use 47/9; if a later task authors POIs 2..10
at region hubs, disable 47/9 to avoid double banners.

### 4.2 Chat line: server 15/2 `ChatPacketEnterArea`

- Client `BaseChatPacketReceiveDispatcher` `0x00b14350` first calls the shared base reader
  `0x00b0f550`, which consumes **two `u16` header words** (family, subtype); case 2 then
  calls `sub_B109B0` -> `sub_B10580`, which reads exactly one `u32` into packet field 3.
  The display path `0x00b10250` looks up the locale key `EnteringChatArea` (numeric id
  **3604**, text `Now entering [*area*].`), binds the packet value as the `area`
  parameter, and appends the formatted line to chat.
- Wire: `u16 15`, `u16 2`, `u32` area **string id** (8 bytes before the outer envelope;
  for Snowhill `0F 00 02 00 79 0F 00 00`, since 3961 = `0x0F79`). The header is **not**
  `u16 15, u8 2`: the base reader waits for a `u16` subtype, so a `u8` subtype would be
  read together with the first area byte as `0x7902`, case 2 would never run, and the u32
  would be read from the wrong offset. The existing 15/1 writer agrees (`social_chat.rs`
  emits `[15, 0, 1, 0]` for family 15/subtype 1). The packet is server->client; the
  client's own chat request path is 15/1 `PacketChat`, and the server's `social_chat`
  routes client 15/1. The outer gateway envelope is the existing tunneled application
  frame; use the chat module's writer.
- The area value is a **name string id**, not the 47/9 region id. For Snowhill send
  **3961**, not 8: the client prints `Now entering Snowhill.` For a sub-area use its place
  id (e.g. Lakeshore 18899, Mire's Edge 117002, Snowhill Hot Springs 4807). `(R)` the u32
  is a locale id: the reader proves the u32, the formatter and `[*area*]` template prove
  the parameter substitution, and the locale map supplies the numeric ids; we did not
  identify a preserved 15/2 packet frame in the reviewed captures (the frames are login/
  zone/minigame), so the native check is still required.
- The original sends this line on area/region entry, including instance scenes:
  `lab/video-archive/out/P2SusMJ2_tQ/frames/00010.jpg` shows the kart/garage scene in
  Briarwood with five `Now entering Briarwood.` lines in the area chat, and `00012.jpg`
  shows the same line later. The repeated lines are why the send rule below is `(R)`; the
  video proves the original sends the line, not the wire bytes. The server should send it
  when the **place/region name actually changes**, not for every position tick.

## 5. Map-label registration (affine fits)

`lab/references/quest-lines/wiring/map-labels.json` has 48 wiki maps and their labels;
some labels have a pixel centre from the OCR pass, some do not. We use the labels with a
centre as anchors when we can match them to an NPC with a `legacy_data`/`reference_data`
position from `data/content/npcs/*.json` (the `placement` and `placement_source` fields
are in each row). The fit solves `x = a*u + b*v + c`, `z = d*u + e*v + f` by least
squares; residuals are metres. A map with an anchor max residual over ~25 m is listed as
a bad fit and **not used** for world positions; a 3-anchor fit is exact by construction
and has no independent check, so its projections are candidates only.

| map | anchors (world source) | max residual | usable? |
| --- | --- | ---: | --- |
| `Map seaside.png` | Post Office Duane 15.3 m; Medical Doctor Callahan 9.1 m; Treasures Cafe Diggy 1.6 m; Seaspray Inn Seaspray 22.8 m | 22.8 m (RMS 14.5) | **yes**; place projections below |
| `Map wugachug.png` | Sasparilla Brewery Brockoom; Chugaroot Farm Farmer Murglot; Carnival/Party Tent Guard Thoomlug | 0.0 m (3 anchors) | exact, unvalidated; candidate only |
| `Map merryvale.png` | Merry Vale region centre (-1040,-404); Mainstage Tyra; Ever Rainbow Arianna | 0.0 m (3 anchors) | exact, unvalidated; candidate only |
| `Map shrouded.png` | Druid Academy Arestol; Pet Park Vidor Stormshroud; Wishing Well Emil Jadefish | 0.0 m (3 anchors) | exact, unvalidated; candidate only |
| `Map sanctuary.png` | Post Office Flitter 21.8 m; Town Hall Quills 18.7 m; Hospital Doctor Gordon 33.2 m; Royal Treasury Bezzle 92.5 m; Adoption Center Brent 96.6 m; Queensfields Sunflower 93.0 m; Pixiewood Hills Edwin 78.8 m | 96.6 m (RMS 70.4) | **no**; hand-drawn map |
| `Map snowhill.png` | Post Office Loryn 5.9 m; Town Hall Mayor Crystalline 24.9 m; Stage Flynn 49.8 m; Smithy Smitty 14.9 m; Penguin Institute Valinda 34.4 m; Hot Springs area centre 34.4 m | 49.8 m (RMS 30.9) | **no**; hand-drawn map |
| `BlacksporeSwamp.png` | Splortata 33.6 m; Chloe 29.7 m; Oswald 64.5 m; Oberyn Duskshine 69.0 m; Mire's Edge/Donovan 93.0 m | 93.0 m (RMS 62.6) | **no**; hand-drawn map |
| `Map world-0.png` | Wugachug 126.9 m; Shrouded Glade 251.7 m; Briarwood 276.8 m; The Sanctuary 488.8 m; Seaside 143.9 m; Merry Vale 204.4 m; Blackspore 102.7 m | 488.8 m (RMS 258.6) | **no**; old/atlas map |
| `Map blackspore.png`, `Map warpstones.png`, `Cragaramap.jpg` | insufficient independent anchors (1 centre or a hub map) | — | not registered |
| `Ponyvalemap.jpg` | `Mystical Barrier` [390,104]; `Pony Vale` no centre | — | not registered; see §6 |

The consistent conclusion is that the wiki region maps are **stylised illustrations, not
scale projections**. A single affine fit fails on Sanctuary, Snowhill, Blackspore and the
old atlas; a local/projective or piecewise fit would be needed before using them to place
NPCs. The lead's map-label idea is sound for names, not for coordinates. The one usable
map (Seaside) projects these places (candidate positions, source map + label, `(R)`):

| place | projected world `(x, z)` | client name id (if exact) |
| --- | --- | ---: |
| Tide Pools | `(-283.5, -1310.8)` | 93721 |
| Boardwalk | `(-504.1, -1279.9)` | 116988 |
| Center / Information Center | `(-564.8, -1187.9)` / `(-534.9, -1106.5)` | — |
| Luxurious Oceanfront Homes | `(-771.6, -1207.4)` / `(-790.6, -1204.3)` | — |
| Abandoned | `(-1133.9, -1133.9)` | — |

Candidate projections from the 3-anchor maps (not used without a native/eye check):
Wugachug title `(25.7, -438.3)`; Sasparilla/Chugaroot/Carnival anchors exact; Merry Vale
Sunflower Patch `(-1099.3, -854.0)`, Midway `(-1039.3, -494.6)`; Shrouded Glade Courtyard
`(-111.7, 955.4)`, title `(608.5, 1366.3)` (the title is not a spatial point; do not use).

Snowhill's no-centre labels were read off the 1568x882 image
`lab/references/freerealms-wiki/harvest/vision/13/13af842eac266cc2f6c2942359167abbae661fc7.png`:
TOWN HALL about `(815, 372)`, STAGE about `(975, 412)`, SMITHY about `(545, 320)`,
SINGING CRYSTAL MINES about `(632, 215)`, PENGUIN TRAINING INSTITUTE about `(1225, 238)`,
ADVENTURER'S LEAGUE about `(1015, 110)`, BIG J'S about `(810, 746)`. These were used only
to test the Snowhill fit; because that fit is bad they are not world positions. The
Sanctuary map's only no-centre place label is BARRACKS; its title "The Sanctuary" and
"BARRACKS" are stylised, and were not used.

Map label **names** are still the best name source for places: Sanctuary (Town Hall,
Royal Palace, Post Office, Hospital, Barracks, Pixiewood Hills, Royal Treasury, Adoption
Center, Queensfields, Memorial Cavern); Snowhill (Crystal Barrier, Adventurer's League,
Singing Crystal Mines, Penguin Training Institute, Smithy, Town Hall, Stage, Hot Springs,
Post Office, Big J's); Seaside (Tide Pools, Fisherman's Shanties, Ocean, Treasures Cafe,
Post Office, Seaspray Inn, Boardwalk, Medical Center, Information Center, Luxurious
Oceanfront Homes, Abandoned, Dojo); Wugachug (Sasparilla Brewery, Carnival, Party Tent,
Chugaroot Farm, Thoroughfare); Merry Vale (Ever Rainbow, Mainstage, Sunflower Patch,
Midway); Shrouded Glade (Gardener's Hutch, Wishing Well, Pet Park, Courtyard, Druid
Academy, Headmaster Merk's); Blackspore (Splorata's Camp, Ferry, The Guild, House, Isle of
Tears, Mire's Edge, Ancient's Wall, Sinking Graveyard, Quagmire Quarry, Shadow, Talon
Dojo); world atlas (Silver Hills, Stormfront Mountains, Black Forest, Lavender Coast —
2007-era names, not the 2014 region names; do not use).

## 6. Pony Vale

Pony Vale is **present in the client**, but not as a separate `.gzne` scene or a
`FabledRealmsAreas.xml` area:

- `Assets_manifest.txt` has `water_ponyvale_stream.adr/.cdt/.dma`, the `pv_*` model/effect
  assets, `pv_grass.eco`, and `pv_palescale_pit_*` map chunks; `catalog.tsv` has the
  `pv_palescale_pit.gzne` 84-byte stub and `pv_wandering_encounter_01.gzne`.
- Locale: `Ponyvale` numeric id **1646** (the `CodeStringMappings.txt` line
  `ssvMapPonyvale^1646^` is the Sunstone Valley map label); `Mystical Barrier` numeric id
  **8802**; many Pony Vale quest/dialog lines.
- The client calls it "the serene corner of Sunstone Valley known to some as Pony Vale"
  (locale text), and the `ssv` prefix on `ssvMapPonyvale` ties it to Sunstone Valley.
- The wiki map `Ponyvalemap.jpg` labels Pony Vale (no centre), Sunstone Valley, Sacred
  Grove and Mystical Barrier [390,104]; the blog (community, 2015 last-day) is the reason
  to look, but it adds no client id.
- There is **no** `Ponyvale`/`pv_` world area in `FabledRealmsAreas.xml` and no
  `ponyvale.gzne`; the `pv_palescale_pit` stub is a battle sub-scene. `(R)` the client's
  region id for the area is therefore **10 Sunstone Valley** (the containing region), and
  the place is Pony Vale (place id 1646 / name `Ponyvale`), with no numeric zone id of
  its own. Its world bounds are not recoverable from the client area table; derive them from
  the Sunstone terrain/POIs or leave it a named place with no polygon.

This is the last-day area, but this 2014 client already carries its locale/art; the
opening was a server/content switch, not a new client scene. Do not invent a Pony Vale
zone id.

## 7. Data pass: what the current tables say

The full data pass is build step (b); this is the measured starting point.

| table | rows | location/zone finding |
| --- | ---: | --- |
| `npcs/*.json` (17 files) | 1,209 | 1,105 carry `zone` + package `zone_id`, 104 neither; positions: 619 `legacy_data`, 298 `poi_anchor`, 109 `reference_data`, 183 `unanchored`; 981 in a client place group, 1,091 in place-or-recovered-region, 118 outside both |
| `npc_spawns_extra.json` | 2,767 | all have positions; 1,606 fall in the nine recovered regions, 1,161 outside; needs the same place pass |
| `npc_spawns_retired.json` | 632 | all have positions; 606 outside the nine regions; retired/dead rows, not drawn; leave unless reused |
| `field_enemies.json` | 49 enemies + 13 spawn groups | 45/49 in a client place group, 4 outside; spawn groups carry `zone_id`/`region` text |
| `quest_nodes.json` | 5 spawns + 1 pool | all 5 positions are in a ~35 m cluster around `(-245, -1166)` (Seaside/Wilds) but the pool's `zone_definition_id` is 1 (Briarwood package); a clear package-id/position mismatch |
| `harvest_nodes.json` | 76 spawns + 2 pools | 70/76 in a client place group, 6 outside; all 76 in the nine recovered regions |
| `mining_nodes.json` | 5 spawns + 1 pool | 0/5 in a client place group (the five Snowhill copper nodes cluster around `(399,351)`, just outside the town shapes), but they are in the recovered Snowhill region; assign region 8 |
| `fishing_spots.json` | 6 spots | 4/6 outside the client world groups; scenes are instances (`sg_fishing_medpond`, zone 563, etc.), so use the scene zone, not the world region |
| `zones.json` | 17 | all `geometry_id: null`; package/region labels, not `.gzne` scenes; the ids are not client region ids |
| `dungeons/*.json` | 75 | 74 `record.zone_id` null; only `opening_cave` has 411/`sg_npx_cavern_01`; `dungeon_data.zone.name` exists for all (e.g. Bixie Hive `Sanctuary (Queensfields)`), so the region crosswalk can be derived; 14 have a `.gzne` scene (piece_059), 29 do not, and `mv_den_of_secrets` is an 84-byte stub |
| `pois.json` | absent | no shipped POI table; the server's startup message 9 builds an empty POI list; only test fixtures |
| `encounters.json` | 1 dungeon + 7 world enemies + world | world `FabledRealms` says `geometry_id: 214`, but the capture says 2; `sg_npx_cavern_01` also 214; treat both as data defects pending a native check |

The 118 NPC rows outside both region sources are listed in §3.3 by content zone and
cluster. The 118 gaps break down as **Greenwood Forest 40**, **Wilds 29 + Lakeshore 15 (44
combined)**, **`reconstructed_unanchored_default` 12** (four near the origin, eight with
plausible coordinates; see §3.3), **Snowhill 7**, **Briarwood 5 (the Briarheart north
edge)**, **Merry Vale 4**, **Seaside 2**, **Blackspore 1**, **Durango Zoo 1**,
**Highroad Junction 1**, **Sanctuary 1**. Cobblestone Village is inside `Newbiezone`
(region 1) and is already covered; it is a place row, not a gap. The `zones.json` names can
supply place names, but not bounds for Greenwood Forest, Cobblestone Village or Durango
Zoo.

## 8. Build plan

Each step is small enough for one builder. The owning modules are named so the hub files
(`gameplay.rs`, `bin/osfr-gateway.rs`, `encounter.rs`, `gateway.rs`) get only a thin call.
No step changes session plumbing, saves ordering, economy, or launcher/login.

### (a) One shared zone/region lookup in data — shipped by T753

**Status:** T753 (commit 7d416a61) shipped this slice: `data/content/regions.json`,
`src/world_regions.rs` (`RegionTable`), loaded by the live owner `NpcVisibility::load`
(`src/world.rs`) through `Gameplay::load` (`src/gameplay.rs`), with
`Gameplay::region_table()` and `realm reload regions` staged on the `Gameplay` clone.
`ContentDb` was **not** extended (its NPC index is test-only; do not add a second table
there). The shipped schema drops the unrecovered BeginZoning `zone_type`/`tutorial`/`sky`
fields and has no `parent_place`; use the shipped row shape. `zones.json` stays for its
nearest-anchor use until step (b).

Shipped schema (row shape; the shipped table has 19 group rows, not one row per building):

```json
{
  "schema_version": 1,
  "client_name_ids": [3961],
  "zones": [
    {"id": 0, "scene": "FabledRealms", "kind": "world", "geometry_id": 2,
     "source": [{"kind": "client", "evidence": "..."}]}
  ],
  "regions": [
    {"id": 2, "name": "Blackspore", "name_string_id": 3500,
     "banner_swf": "blackspore_notification.swf", "parent_zone": 0,
     "shapes": [{"shape": "rect", "x1": -2820, "z1": 820, "x2": -1680, "z2": 1700}],
     "source": [{"kind": "reconstructed", "evidence": "..."}]}
  ],
  "places": [
    {"id": "snowhill", "name": "Snowhill", "name_string_id": 3961, "region_id": 8,
     "priority": 50,
     "shapes": [{"shape": "sphere", "x": 156, "z": 390, "r": 135}],
     "source": [{"kind": "client", "evidence": "..."}]}
  ]
}
```

Sanity checks at load (fail closed, `Result`, no panic; all implemented in T753): zone id
0 exists and is the only world zone; every scene name is a known `.gzne` stem or
explicitly `unresolved`; region ids are exactly 1..10, unique, with a name id and (for
2..10) a `*_notification.swf`; place ids/keys unique; every place `region_id` exists;
every shape has finite coordinates and positive radius or ordered box corners; every
place/region has a `source`; reject `geometry_id 214` for the world and any `geometry_id`
not in the captured/scene allowlist until checked; reject a region id used as a zone id
and vice versa; reject duplicate scene names; cap zones/places/shapes/name ids
(data-driven) to keep the lookup bounded.

`RegionTable::region_at(zone,x,z)` / `place_at(zone,x,z)` implement §3.4 and return
`Option`; the `Region`/`Place` fields each document their rule on the struct (checklist
18).

**Acceptance tests:** `tests/world_regions.rs` drives the live `Gameplay::load` (not
`ContentDb`), with synthetic table cases for duplicates, missing name id, invalid shape,
unknown place region and out-of-range ids; the reload transaction is covered by the inline
`content_reload` test and the existing `realm reload all` regression, not a second copy.
Pin the **shipped** data counts (**1** world zone, **10** regions, **19** place groups, **590** grouped
spheres, **9** recovered region shapes and the per-group list: **49** Blackspore, **54**
Briarwood, **2** Briarheart, **57** Merryvale, **36** Sanctuary, **4** Queensfields,
**7** Memorial Caves, **30** Seaside, **19** Shrouded Glade, **27** Snowhill, **4**
Wugachug, **9** Sunstone Valley, **148** Wilds, **18** Lakeshore, **17** Crossroads,
**51** Highroad Junction, **51** Stillwater Crossing, **6** Farnum and **1** Newbiezone).
The client XML measurement is recorded above, not pinned by the suite because the XML is
not in the repo: **697** top-level always-on spheres of **707** top-level shapes with the
filter above. Classify Sanctuary/Snowhill/Blackspore/Greenwood/Lakeshore/Newbiezone
points; refusal/unknown returns `None` and changes no state; **canary:** change one shape
bound and the pinned count/classification fails, then revert. Existing
`tests/content.rs`/`content_check.py --strict` stays green. A **gateway-level test is
still required for step (d)**: a synthetic `RegionTable`/`ContentDb` test alone does not
prove the running gateway loads the table or sends anything. **Native check:** a read-only
`realm region <x> <z>` debug line (or a `--check` mode) prints the table row; compare 10
points to the client area `ContainsPoint` (`ClientAreaSphere::ContainsPoint 0x00b0b0b0`,
box `0x00b0b1c0`) by hand.

**Perf:** one scan of at most 19 place rows + 10 region rows per lookup, no allocation; no
grid. Reload swaps the table atomically through the existing content reload path.

### (a2) NPC placement by role from registered map labels (after the region table)

This is the lead's role-based placement step, not a general "place every NPC". Only maps
with an accepted affine fit (currently **Seaside**, max residual 22.8 m) may place a row;
the 3-anchor maps (Wugachug, Merry Vale, Shrouded Glade) are candidates only until an
independent anchor validates them. For a row whose `placement` is `poi_anchor` or
`unanchored` and whose role names a labelled building, the builder may use the label's
projected world position **in front of** the building (standoff), with a row source of
the form `"(R) inferred from <map image> <label> + role <role>; fit residual <n> m"`.
Rows on a bad-fit map remain unanchored; do not use the Sanctuary, Snowhill or Blackspore
maps for placement.

**Acceptance tests:** every inferred row cites a map, image path, label and role, and the
fit's residual; no two drawn rows share the same position (rounded to 0.1 m) after the
pass; a row without a usable map stays `unanchored` with a reason; the region/place test
from (a) still classifies the new position; canary: remove the citation or duplicate a
position and the placement test fails; revert. **Native check:** stand at each inferred
position and confirm the labelled building is in front and the NPC is not inside geometry,
then interact with the NPC; this is an eye check, not a new packet.

### (b) Data pass: every location row gets its real zone and region

**Deliverable:** update every content table with (i) `zone_id` = 0 for world rows or the
instance zone id for instance rows, (ii) `region_id` 1..10, (iii) `place_id` where the
place is known. Add `zone_kind`/`unresolved_reason` for rows that cannot be placed; never
invent a position. Convert `zones.json` package ids to place keys/region ids; do not load
them as `Zone`.

- NPC rows: world NPCs -> zone 0; `region_id` from the recovered nine shapes; `place_id`
  from the client groups; `zone` text kept as a display/region page. There is **no live
  "no zone id for npc" refusal to fix**: `NpcVisibility::load` already inserts every drawn
  regional and extra spawn into `OPEN_WORLD_ZONE` and discards the computed package/region
  id. The data pass's value is the region/place classification for display, not the NPC
  zone target. Do not change the test-only `World::new`/`ContentDb` index and call it a
  fix; the gateway route uses `NpcVisibility`.
- `npc_spawns_extra`: same pass; leave retired/unanchored rows explicit; do not reuse a
  retired row as a live spawn.
- `quest_nodes`: the 5 Briarwood package rows at `(-245,-1166)` are a data error; either
  move the rows to the correct pool/zone or mark unresolved. `mining_nodes`/`harvest_nodes`
  keep their pool zone but add region/place.
- `field_enemies`: `zone_id 0` + region/place; spawn groups' `region` text becomes a
  crosswalk, not an id.
- `fishing_spots`: instance scenes keep their scene zone; no world region.
- `dungeons/*.json`: `record.zone_id` is an **instance zone id**, not a region id; add
  `scene` (`.gzne` stem), `region_id` (from the entrance place), `geometry_id`, and leave
  the 29 no-scene records `unresolved`. `opening_cave` 411/`sg_npx_cavern_01` is the one
  existing example; the 14 piece_059 scenes (e.g. `sg_bixie_hive`, `bw_spider_lair`,
  `sh_frostfang_cavern`, `ss_cray_caves`) get the next ids.
- `encounters.json`: change world `geometry_id` to 2; keep `sg_npx_cavern_01` 214 only if
  the client accepts it (native check); no 214 placeholder.
- `zones.json`: either delete it or convert its 17 rows to named places (with
  `unresolved` for Durango Zoo) and remove it from the test-only `ContentDb.zones` as well;
  the `.gzne` stems are the only scenes.

**Acceptance tests:** `content_check.py --strict` with a new rule: every world location
row has zone 0; every instance row's zone exists; every `region_id` is 1..10; every drawn
legacy/reference row has a place or region; every dungeon with a `.gzne` has a unique zone
id and scene, and no `DungeonHasNoZone` for it; every unresolved row has a reason. Pin
counts: 75 dungeons (74 null -> 14 resolved, 29 no-scene, rest explicit), 1,209 NPCs
(1,091 covered, 118 unresolved reason), 118 gap rows listed by cluster. Canary: change one
row's `region_id`/`zone_id` to an unknown value and the content load fails; revert.
**Native check:** start the gateway, `realm goto list` shows the drawn NPC count; spawn in
each of the nine regions and in Greenwood/Lakeshore; `realm goto` an NPC in each region;
no "no zone" refusal; a dungeon with a `.gzne` enters and returns.

### (c) Teleport and `realm goto` use the lookup

**Deliverable:** keep the existing `NpcVisibility::load` rule: every drawn world NPC is
already inserted into `OPEN_WORLD_ZONE` (zone 0) and the package/region id it computes is
diagnostic only. Do **not** change the test-only `World::new`/`ContentDb` index and call it
a fix; no live "no zone id for npc" refusal exists. `dev_teleport` targets include every
zone-0 drawn NPC; `admin_move`/warpstone/teleport destinations use the same `RegionTable`
for the arrival region/place. No new teleport path: reuse `dev_teleport::handle` and the
existing movement token/expected-discontinuity path.

**Acceptance tests:** `tests/dev_teleport.rs` — list is non-empty for zone 0; `realm goto
<npc>` works for a legacy NPC in each region; unknown/ambiguous/refused targets change
nothing (snapshot position, inventory, quest state; assert equal); teleport into a
non-world instance still requires a target in that zone; no new `WORLD_ZONE` constant
copy (grep: `dev_teleport::WORLD_ZONE` is the one). `tests/world.rs` — every drawn NPC
target is in zone 0 or a registered instance, never a region-only id (see
`zone_less_legacy_regional_npcs_spawn_in_the_open_world_and_teleport_index`). A
gateway-level test (real session/bot) `realm goto`s a legacy NPC in each region; do not
assert this through `ContentDb`. `cargo test --test spine_rules`. Canary: put a drawn NPC
in a region-only zone and `realm goto list`/the zone test fails; revert. **Native check:**
the same `realm goto` in each region, plus a warpstone teleport; no `no zone id for npc`.

### (d) Banner and chat line on region/place change

**Deliverable:** a focused `src/region_banner.rs` (or extend `client_areas.rs`, the owner
of server client-area/UI packets) that builds:

- 47/9 inner packet: `u16 47, u8 9, u32 len + decimal zone id bytes, u32 len + name
  bytes`; only for region ids 2..10 with a known SWF; use the gateway's existing
  application/tunnel envelope.
- 15/2 inner packet: `u16 15, u16 2, u32 name_string_id` from the region/place table; use
  the existing chat writer/reliable path; no client text/format strings. The shared base
  reader `0x00b0f550` reads the second header word as a `u16`; a `u8` subtype is wrong.
- Session state: `last_region_id`, `last_place_id`, `last_banner_ms` (not persisted); on a
  region change send 47/9 once; on a place/region change send 15/2 once; on world entry or
  reconnect recompute from the saved position and send the current banner/chat once;
  on teleport/zone entry run the same function after the position is committed. Data
  cooldown/minimum interval and enabled flags; no packet while unchanged; no per-tick
  allocation or clone.
- 47/9 is server->client only; do not route a client 47/9 to this code (gateway family 47
  currently routes only 12/13 to quest_engine; unknown subtypes are refused/logged). 15/2
  is server->client only; client chat remains 15/1 in `social_chat`.

**Acceptance tests:** `tests/region_banner.rs` — byte-level builders for 47/9 `"8"` +
`"Snowhill"` and 15/2 `3961`; the 15/2 expected bytes are `0F 00 02 00 79 0F 00 00`, and
the canary is the old wrong layout `0F 00 02 79 0F 00 00` (a `u8` subtype), which must fail
because the reader consumes subtype `0x7902` and never reaches case 2. Also canary a wrong
47/9 byte length. Drive a **real gateway session** (or the existing bot helper), not a fake
session or a copied builder: a player that moves across the Snowhill boundary gets exactly
one 47/9 and one 15/2, staying still sends none, reconnect sends one, a refused teleport
changes no state; a client 47/9/15/2 is refused and changes nothing. `cargo test --test
app_fuzz` only if an inbound selector is added (none planned); `tests/spine_rules`.
**Native check:** walk from Sanctuary into Snowhill; the client shows the Snowhill banner
and `Now entering Snowhill.`; recross with no duplicate; `realm goto` a Snowhill NPC; a
kart instance with a known region gets the chat line (and banner if the region is known);
reconnect in Snowhill restores the banner/chat once.

### (e) Dungeon zone ids

**Deliverable:** for each of the 14 dungeons with a real `.gzne` (piece_059), assign a
unique instance `zone_id`, `scene` stem, `region_id` from the entrance (`dungeon_data.zone`
text), and `geometry_id` from the captured/scene data; the 29 without a `.gzne` stay
`unresolved` with a reason; `mv_den_of_secrets` remains a stub. Route the dungeon entry
through the same zone/region function; no `DungeonHasNoZone` for a resolved dungeon.

**Acceptance tests:** every dungeon record's `zone_id` is unique and not 0/1..10; every
resolved scene is a `.gzne` stem in the catalogue; a resolved dungeon enters and returns
(win/cancel/disconnect/timeout) and the player is free, in zone 0, with the pre-entry
position; unresolved dungeons refuse before they consume the entry/party state and change
nothing; canary: duplicate a zone id/scene and the dungeon test fails; revert. **Native
check:** enter each resolved dungeon, observe BeginZoning + return; no stuck actor; the
region/chat on entry uses the dungeon's place name id, not the region id.

## 9. Open gaps and (R) rules

- **No identified 47/9 or 15/2 packet frame** in the reviewed captures (login/zone/
  minigame frames; a raw byte scan is not proof because frames are encapsulated). The
  15/2 display is visible in `P2SusMJ2_tQ/frames/00010.jpg` (chat line), but not its wire
  bytes. The readers are first-party; the two-string 47/9 payload (first = decimal zone
  id, second = unused/name) and the 15/2 u32-as-string-id are `(R)` until the native
  check. The 15/2 header width is **not** `(R)`: `0x00b0f550` reads two `u16` words.
- **Geometry id semantics** are only partly recovered: the client uses it to select the
  journal region (`FUN_00a44b70`), but the content's world `214` contradicts the captured
  world `2`; the `id` field is an instance id, not a type. The note recommends captured/
  scene-specific geometry ids and a native check; do not fill `214` everywhere.
- **The 17-row `zones.json`** is a reconstructed package/region-name table, not a client
  zone table; its ids are not client region ids. Retiring it as `Zone` is a design change
  for the lead/build step (b); no code was changed here.
- **Map fits:** only Seaside passes an affine fit with an independent anchor (max 22.8 m);
  Wugachug/Merry Vale/Shrouded Glade are 3-anchor exact but unvalidated; Sanctuary,
  Snowhill, Blackspore and world-0 are bad fits (48-489 m) and are not used. The wiki
  maps are stylised, not scale maps; a local/projective fit is a separate task. The
  "read off image" Snowhill no-centre pixels are recorded but not used because the fit is
  bad.
- **Pony Vale** is a client-named, asset-backed **place in Sunstone Valley** (name id
  1646; Mystical Barrier 8802), not a `.gzne` scene or an area row; no bounds. The 2015
  last-day opening was server/content, not a new scene.
- **Greenwood Forest/Cobblestone Village/Durango Zoo** have no first-party area shapes;
  Greenwood/Cobblestone bounds must be reconstructed from NPC clusters (R) or left
  unresolved; Durango has only placeholder rows.
- **The 118 NPC gaps** are listed by content zone/cluster in §3.3; the full per-row report
  is the output of step (b), not this note. Do not force a region on a
  `reconstructed_zone_center` row, and use `placement_source` for the 12
  `reconstructed_unanchored_default` rows; only four of those are near the origin (see
  §3.3).
- **Region 1/no banner:** the only `RegionOverride` is `Newbiezone` id 1; the Lua table
  has no SWF for 1. Never send 47/9 with 1 (no SWF); the chat line can still use a place
  name id.
- **The client `CurrentPOI` alternative** can trigger the banner from a POI id 2..10; the
  preserved community POI table is not original region data and the shipped server has no
  `pois.json`. Do not enable both paths.
- **No economy/save impact:** the banner/chat carries no rewards, items, coins or
  persistence; region is derived from the saved position. This is deliberate.

## 10. Structure

- **Graph-first queries run** on codebase-memory project
  `C-Users-<user>-Documents-Projects-CartoonMMO-autopilot-int`:
  `search_graph` for "zone id scene name region area teleport" returned 917 related
  symbols, including `piece_076.REGION_AREAS`, `piece_077.WarpRegistry.zone_teleport`,
  `dev_teleport`, `world`, `client_areas`, `zone_transition`, `EncounterEngine.field_teleport_targets`,
  `terrain.terrain_loads_zone_files_by_id`, and `gameplay.housing.scene_zone`. The graph
  server was available; no grep-only fallback was needed for the owner search. The
  research then read the owning modules and the hub call sites, not the whole hubs.
- **Modules touched:** none by the T749 research commit; F752 corrected this note after
  review. The shipped owners are `src/world_regions.rs` (lookup) and `src/world.rs`
  (`NpcVisibility::load`), reached through `Gameplay::load`; step (b) uses the content
  tables, step (d) `client_areas.rs`/`social_chat.rs`, step (e) dungeon content. Hub files
  (`gameplay.rs`, `bin/osfr-gateway.rs`, `encounter.rs`, `gateway.rs`) get no new logic
  beyond a single call; net lines added per hub file: **0**.
- **Deleted:** none. **Net hub lines:** 0. **Tests:** none changed; the plan's tests are
  for the follow-up builders. F752's corrected counts/byte vectors are measured in
  `notes/Autopilot/F752.md`.
- **Scratch:** all analysis scripts/outputs are in `%TEMP%\t749` (outside the workspace);
  F752's verification scripts are in `%TEMP%\f752`; no scratch files were added to the
  repository.
