---
updated: 2026-10-26
task: T748
status: research and design; no source or data changes
---

# Dungeon Instances

Research brief, 2026-10-26: how the client enters and runs quests inside the dungeons. This note
answers how the 2014 client enters, runs and leaves a dungeon, what the server must send, and the
smallest ordered
set of build slices that ends with quests inside a dungeon completing. It changes no source, content
or data. Everything marked **(R)** is a reconstruction or a local design choice, not a recovered
server rule.

## Short answer

A dungeon is a **combat minigame** (client type 4) carried by two existing packet families:

- family **41 (encounter)**: 114 details/start screen, 107 zone ready, 108 Go, zone change on 31,
  106 state, 2 player enter, 133 fighting, 5 objective state, 109 exit;
- family **39 (minigame framework)**: 17 start, 18 game over, 47 score rows, 19 remove, 45 wheel
  landing, 46 wheel stop report, 62 boss rows, 68 reward update.

The client contains no dungeon instance manager, no dungeon zone table and no prize-wheel table. The
server owns the run, the scene name and zone id, the enemy list, the goals, the rewards and the
choice of wheel slice. Client data supplies the **atlas markers** (opcode 57), the locale text, the
models, the item name ids and the UI behaviour.

The current server already runs the whole generic combat-minigame route for eleven field battles,
five of them dungeon-named (Bixie Hive, Stone Heart's Hideaway, Sheep Watch, Bandit Hideout, King of
the Ring). They all use the shared arena scene `sg_random_encounter_clearing`, zone 9600 (R), not
their own dungeon scene. Only the opening cave enters its own dungeon scene today
(`sg_npx_cavern_01`, local zone 411). The missing work is therefore **content and small
data-driven extension**, not a new dungeon subsystem.

The first playable dungeon should be **Misty Mountain**: its scene and all its chunks are in the
extracted client, and its captured atlas row gives zone **200312** and the exact gate position in
Shrouded Glade. The smallest end-to-end slice is one content row using the existing field-battle
route, with the row's zone changed to `gl_misty_mountain` and an invisible marker on the existing
gate. Vale of the Ancients is the same shape but its four terrain chunks are in the client manifest
and **not in the extracted catalogue**, so its scene must be extracted and delivered first.

## Corrections to the brief

| Brief statement | Verified state on the integration tip |
|---|---|
| 74 dungeons have an instance scene name | **25** files: 24 guide pages plus `opening_cave.json` (T015). The other 50 files have no scene field; Vale's candidate is recovered from the manifest below. |
| 74 dungeons have no zone id | The 75 JSON files still have only opening cave 411, but the **captured startup POI stream** carries the client's zone id for **41 dungeon atlas rows**, including Misty Mountain 200312 and Vale of the Ancients 2367. |
| Only `opening_cave.json` runs | Eleven field-battle rows run end to end, including Bixie Hive, Stone Heart's Hideaway, Sheep Watch, Bandit Hideout and King of the Ring. They run in the shared arena, not in their dungeon scenes. |
| The 2,008 unresolved references are client-data gaps | The 2,008 consist of 1,667 item, 239 NPC, 41 object, 52 zone and 9 quest references. For the 1,667 prize-wheel item references, the **names** are client locale text and the **item ids** are recoverable from the item definition name id: 1,628 resolve. The missing part is the **server-authored wheel composition**, which is not in client data at all. |
| The client sends a dungeon-entry request | No dungeon-specific request was found. Entry starts with an ordinary world interaction (26/8) on a marker actor; the actual start request is 41/108 Go after the 41/114 start screen. |

## Client evidence

All addresses below were read in the supplied 2014 `FreeRealms.exe` with `ida_query.py`. The
function names are the native map's names where they exist; a plain address means the function has
no accepted name.

### Atlas markers: opcode 57

The client's atlas destination table is rebuilt from the server's 57 reply:

- handler `ClientPointOfInterestManager_OnDefinitionReply` **`00b98b70`**; row stream
  `00b98990`; row reader **`00b98290`**;
- the row reader consumes **13 little-endian dwords** (three ids, three position floats, heading,
  and six further dwords), a signed-length byte string, and a trailing dword;
- the client data-source column table **`00b97c40`** names column 2 **`Zone ID`**, which is the
  third dword the row reader reads. The Lua `PointOfInterestData_DS` confirms the same order
  (`ScriptsBase.lua` line 2966: Id, Name, ZoneID, X, y, z, Heading, IconId, NotificationType,
  SubName, BreadcrumbQuestId, AtlasName).

The preserved startup export `lab/reports/worldcontent-20261013T203151Z/world-content.json`
contains 149 captured POI rows. Dungeon rows have `notification_type` 3 and icon 11970 (the
crossed-swords combat marker). The captured Vale/Misty rows read:

| Field | Misty Mountain (POI 90) | Vale of the Ancients (POI 77) |
|---|---:|---:|
| `name_string_id` | 71773 | 18143 |
| `location_id` = client **Zone ID** | **200312** | **2367** |
| marker position | 158.58415, 44.722233, 1075.0023 | -850.7995, -61.67808, 2307.195 |
| heading | 2.405411 | -1.57396 |
| icon id | 11970 | 11970 |
| notification type | 3 | 3 |
| teleport location id | 210104 | 210091 |
| atlas | sacredgrove | sacredgrove |

The values are from the private startup capture, not an original-server zone dump. The
interpretation of the third dword as the zone id is client-reader evidence; the value's provenance
is the capture. Treat the zone ids as **R until one dungeon scene transition is observed natively**.

The captured POI stream gives zone ids and entrance positions for 41 dungeon rows. The 25 recovered
scene names map as follows (the rest of the 41 rows have no recovered scene stem yet):

| Dungeon | `instance_scene_name` | Zone id | Source |
|---|---|---:|---|
| Arachnia's Lair | `bw_spider_lair` | 925 | POI 57 |
| Bandit Hideout | `sg_bandit_hideout` | 1130 | POI 57 |
| Bixie Hive | `sg_bixie_hive` | 2039 | POI 57 |
| Bone Bog Cemetery | `bs_bone_bog_cemetery` | 210900 | POI 57 |
| Briarheart Caverns | `bw_briarheart_caverns` | 2044 | POI 57 |
| Cray Caves | `ss_cray_caves` | 14055 | POI 57 |
| Deep Mines | `sh_deep_mines` | 2026 | POI 57 |
| Den of Secrets | `mv_den_of_secrets` | 14053 | POI 57 |
| Floren Forest | `sg_floren_forest` | 2047 | POI 57 |
| Frostfang Caverns | `sh_frostfang_cavern` | 1631 | POI 57 |
| Haunted Mines | `sg_haunted_mines` | 18310 | POI 57 |
| Highroad Hijinx | `sg_highroad_hijinx` | 2046 | POI 57 |
| Misty Mountain | `gl_misty_mountain` | 200312 | POI 57 |
| Mugwort's Hollow | `sg_mugworts_hollow` | 2048 | POI 57 |
| Mushroom Cavern | `bw_mushroom_cave` | 2365 | POI 57 |
| Necronomicus Unleashed | `mv_necronomicus` | — | no POI row |
| Opening cave | `sg_npx_cavern_01` | 411 | local assignment (R) |
| Robgoblin Treasure Trove | `sg_robgoblin_trove` | 1632 | POI 57 |
| Sheep Watch | `sg_sheep_watch` | 14062 | POI 57 |
| Sweetwater Climb | `wc_sweetwater_climb` | 14052 | POI 57 |
| Tanglewood Fort | `bw_tanglewood_fort` | 14057 | POI 57 |
| Tavern Cellar | `sg_tavern_cellar` | 1133 | POI 57 |
| The Rumbledome | `gw_rumbledome` | 256670 | POI 57 |
| Trail of Betrayal | `bw_trail_of_betrayal` | 2045 | POI 57 |
| Treeleaf's Retreat | `bw_treeleaf_retreat` | 2451 | POI 57 |

Vale of the Ancients has no recovered `instance_scene_name` in its JSON, but the client manifest
contains `bw_corrupted_vale.gzne`, `bw_corrupted_valeAreas.xml` and four terrain chunks; its POI row
gives zone **2367**. The association `Vale of the Ancients` -> `bw_corrupted_vale` is (R) until
the scene is loaded natively. Its manifest rows are:

```
bw_corrupted_vale.gzne.z                 2032140275  1068
bw_corrupted_vale_0_0.gck2               2271686789 356750
bw_corrupted_vale_0_0.gcnk               4109593679 659964
bw_corrupted_vale_0_4.gck2               2048457023 344200
bw_corrupted_vale_0_4.gcnk               1706734140 662922
bw_corrupted_vale_4_0.gck2               2901276269 358953
bw_corrupted_vale_4_0.gcnk               2404089959 757809
bw_corrupted_vale_4_4.gck2               2503078253 288457
bw_corrupted_vale_4_4.gcnk               2354910026 603330
```

`lab/extracted/client-assets/catalog.tsv` currently holds only the GZNE, Areas.xml and TileInfo for
Vale; the four GCNK/GCK2 pairs and the textures are absent. Misty's 16 scene chunks are present.

The complete dungeon POI table (zone id and entrance position for every captured dungeon marker):

| Dungeon | POI id | Zone id (`LocationId`) | Entrance position (x,y,z) | Teleport id |
|---|---:|---:|---|---:|
| Arachnia's Lair | 52 | 925 | -790.889,-71.6888,2153.09 | 210066 |
| Bandit Hideout | 54 | 1130 | -1702.97,-6.099,-574.389 | 210068 |
| Bixie Hive | 62 | 2039 | -1445.68,-38.3424,49.8923 | 210076 |
| Bone Bog Cemetery | 119 | 210900 | -2386.58,-23.7879,1236.64 | 210953 |
| Briar Patch | 71 | 10112 | -410.724,-54.931,2097.39 | 210085 |
| Briarheart Caverns | 66 | 2044 | -38.8019,-0.00425,2900.15 | 210080 |
| Bristlewood Glade | 76 | 2366 | -385.926,-68.0362,1871.81 | 210090 |
| Cracked Claw Caverns | 85 | 14056 | -2267.7,-29.346,896.016 | 210099 |
| Cray Caves | 84 | 14055 | -45.1757,-31.0491,-1172.11 | 210098 |
| Croaking Vale | 72 | 9193 | -319.82,-102.05,2593.83 | 210086 |
| Danger Peaks | 64 | 2041 | -1433.58,-13.73,-904.22 | 210078 |
| Dark Spore Depths | 74 | 2364 | -645.029,-50.7753,2507.95 | 210088 |
| Darvon's Descent | 80 | 5672 | -903.477,-47.8685,239.337 | 210094 |
| Deep Mines | 61 | 2026 | 486.989,47.2909,272.661 | 210075 |
| Den of Secrets | 83 | 14053 | -803.671,-51.344,-678.601 | 210097 |
| Floren Forest | 69 | 2047 | -223.126,3.54397,-800.55 | 210083 |
| Forest Troll Fort | 65 | 2043 | -433.54,-0.11074,1345.53 | 210079 |
| Forgotten Caves | 63 | 2040 | -912.187,-63.6597,399.106 | 210077 |
| Frostfang Caverns | 59 | 1631 | 91.5179,66.3374,552.865 | 210073 |
| Grexan's Camp | 79 | 3026 | -680.377,44.2332,1492.32 | 210093 |
| Haunted Mines | 88 | 18310 | -2587.95,-19.1101,1024.19 | 210102 |
| Hewey's Escape | 89 | 20103 | -447.579,-0.67554,326.01 | 210103 |
| Highroad Hijinx | 68 | 2046 | -2182.35,-36.5444,141.077 | 210082 |
| Hot Springs Haven | 53 | 934 | 264.565,50.7842,169.914 | 210067 |
| Howling Hills | 73 | 2357 | 385.373,117.924,-16.925 | 210087 |
| Misty Mountain | 90 | 200312 | 158.584,44.7222,1075 | 210104 |
| Mugwort's Hollow | 70 | 2048 | -115.596,20.9826,800.971 | 210084 |
| Mushroom Cavern | 75 | 2365 | -67.6682,-30.3359,2313.79 | 210089 |
| Precursor Ruins | 147 | 209974 | 419.868,8.10896,2222.97 | 256667 |
| Robgoblin Treasure Trove | 60 | 1632 | -537.532,26.0394,772.462 | 210074 |
| Sandscale Oasis | 148 | 256671 | 631.585,81.3666,1760.91 | 256669 |
| Sheep Watch | 87 | 14062 | -2114.05,-57.0443,279.464 | 210101 |
| Snowy Canyon | 57 | 1247 | 421.329,84.5465,643.128 | 210071 |
| Sweetwater Climb | 82 | 14052 | 145.904,37.9414,-469.974 | 210096 |
| Tanglewood Fort | 86 | 14057 | -360.277,-83.0824,2403.68 | 210100 |
| Tavern Cellar | 55 | 1133 | -419.089,18.8017,-246.099 | 210069 |
| The Rumbledome | 146 | 256670 | 428.472,71.5256,1840.53 | 256665 |
| The Snarling Hedges | 56 | 17018 | -901.375,-49.96,1835.92 | 210070 |
| Trail of Betrayal | 67 | 2045 | -546.115,-39.0505,2591.94 | 210081 |
| Treeleaf's Retreat | 78 | 2451 | -710.791,-49.829,1754.53 | 210092 |
| Vale of the Ancients | 77 | 2367 | -850.799,-61.6781,2307.2 | 210091 |

### Entrance: what the client sends

No dungeon-specific entrance opcode was recovered. The route found in the current field-battle
implementation and in the native runs is:

1. The client selects the marker actor with **26/19** and interacts with **26/8**. 26/8 constructor
   `008b6bc0` stores the target (two dwords at +16/+20) and a byte at +24. It is the same packet
   an NPC or object interaction uses; `26/10` is the dialogue/choice packet, not an entrance
   request.
2. The server validates the marker, range, busy state, combat job and level, then sends
   **41/114** with `launch = false` plus **41/107**. `009bdd70` is the 41/114 details reader. The
   start screen shows title, icon, description, difficulty, level requirement and the preview
   prize rows. 41/107 calls `MiniGame:SetMiniGameReady` (`battle_screen` helper `009b0cc0`),
   which turns "Loading..." into **Go!**.
3. Go sends **41/108**. Constructor `008b6e70` is `sub_8a2fd0(108)` plus the two open-screen
   header words; sender `00938f00` fills them from the client's open screen. The server uses its
   own pending row, not the body.
4. The server starts the run and sends **31** begin-zoning. Reader `008e4ae0` reads the scene
   string, type, position float4, rotation float4, sky string, id, update radius, geometry and
   two booleans. This is the same codec the opening cave and kart already use.
5. The client loads the scene and sends **13/10** (finished loading / ready). The server then sends
   the entry burst (`EncounterEngine::on_zone_ready`): **41/114 launch=true**, **41/106** state
   4 and 5, **41/107**, **41/2** player enter, **39/17** game started, **62** combat rules,
   **41/133** fighting, **38/1** health, **35/2** actors, **35/5** health, **32/9** boss display,
   **35/12** relevance, **45/1–2** goal counts and **41/106** state 6.
6. The client fights with **36/10** ability requests; the server answers **32/7** results,
   **35/5** health, **125** movement and **35/8** animations. 41/2 reader `00a9ccc0` is the
   player-guid + trailing tutorial byte; the trailing byte is why the current server sends
   `u64 1 + 0`.
7. Goals update with **45/1–4** (and 5 unhide): `DispatchObjectivePacket` **`009be130`** dispatches
   objective-family subcodes 1–6 and 8–12. The `BaseClient.MiniGameGoals` table is backed by
   `MiniGameGoalDataSource` (`00c472d0`), whose rows come from the active minigame's goal list.
   The 41/114 goal list itself is read by `009bc380`: `i32 count`, then `0xe8`-byte records; each
   record's parser `008fd770` reads three dwords, a bonus byte, a reward bundle, four count/status
   dwords, a members-only byte and one trailing dword. 41/5 `EncounterPacketObjectiveState`
   (`00a9c300`, body reader `00aa31a0`, handler `009bd8f0`) is a second path that synchronizes
   encounter goal records into the same goal state.
8. The boss bar is either `BaseClient.Bosses` (32/9 registers the GUID; 35/5 keeps current/max HP;
   reader `00a2a8a0`, handler `00a2b910`) or `MiniGame.Bosses` (39/62; handler `00c48c90`, body
   reader `00c48620`; type 1 = key/name/health update, type 2 = delete). The minigame framework
   names `MiniGame.Bosses` for minigame bosses. The current combat path uses 32/9 and was accepted
   natively; whether the original dungeon's top-centre bar is the minigame table is unproven.
9. Knockouts/lives: 39/23 carries current and maximum knockouts to `GroupHandler:SetKO`
   (`009b87a0`); 35/18 carries the remaining lives. 41/125 opens the respawn window
   (`008a33b0`, reader `008d6730`) and 41/122 participant resume (`008a3250`) is the client's
   revive request. The current engine ends a run on the first knockout; the original score screen's
   "Knock outs Remaining N/5" means this is not yet faithful.

### Completion, score screen and wheel

- **39/18 GameOver** (`009ad480`): `u8 won` and three i32 (stars, elapsed seconds, total score).
- **39/47 GameEndScore** (`009bb030`): `i32 row count`, then per row a length-prefixed name and five
  i32 (icon, score type, count, max, points). The total row is type 4.
- **39/19 StateRemove** (`009bf190`): header only; it calls the Lua score screen. The client
  automatically answers 39/7; that is an acknowledgement, not the player's Close.
- The results screen is Lua `ScoreScreen` (`ScriptsBase.lua` 48300 onward). It reads
  `BaseClient.MiniGame`, `BaseClient.MiniGameScore`, `BaseClient.MiniGame.Reward` and
  `.Reward.Entries`; `ScoreScreen:PopulateLootWheel` reads
  `BaseClient.MiniGame.RewardPreview.Entries`.
- **39/45 MiniGameLootWheelSetItemToLandOn** (`009b9e20`) lands the wheel on the one reward entry
  the server chose. The client spins and sends **39/46 LootWheelRotationStopped**
  (`009bda80`); the server pays at most once per instance. **39/68 UpdateReward** (`009b9eb0`)
  carries reward bundles. The client does not decide the prize.
- Close is **26/42** (results Close constructor `008a2eb0`, sender `0092f300`) or **41/109**
  (encounter exit constructor `008a31a0`; the UI close chain is `008d2730` -> `009bd240`). The
  current engine also treats a bare 41/109 as exit.
- **41/3 EncounterComplete** (`00aa1d70`, reader `00aa02b0`) exists with a boolean and three
  reward reads. The current combat path does not send it; its role relative to 39/18 is not
  established by any capture. Do not add it speculatively.

### Prize item ids: where they live

Item definitions carry their display name as a `NameId`; the client locale maps that id to the
text. The project's item catalogue (`lab/reports/item-catalogue-20261015/catalogue.json`) exposes
the definition bytes and the locale export (`lab/reports/locale-id-to-text.json`) resolves them.
Examples:

| Name | `NameId` | Item id |
|---|---:|---:|
| Battle Item Mystery Pack | 6666 | 10482 |
| Warrior's Battle Hammer of Whirlwind | 132014 | 75130 |
| Hydrostatic Medic Helmet | 133119 | 76111 |
| Warrior's Power Shard of Regeneration III | 131219 | 75475 |

Some locale rows wrap the display form as `{n}Singular{pl="Plural"}`; the plural value is the item
name. With that normalization plus removal of the guide's trailing wiki note, **1,628 of the
1,667** `dungeons_unresolved.json` item references resolve to at least one item id (724 unique
names). The catalogue is a reconstruction from Legacy rows plus client-if-present definitions, so
the locale text is first-party while the definition row is not independently proven for every id;
the resolved ids are strong content candidates, not a wholesale client-table dump. Vale resolves **25/25** unique wheel names;
Misty resolves **25/25** (plus one parser-mangled trailing token in the JSON). The item ids are
therefore client-data recoverable. The **assignment** of items to a dungeon and job is not: it is
sent by the server at run time in 39/45/68 reward entries. No static client table maps a dungeon to
its prize wheel. The 1,667 prize-item references inside the 2,008 gaps are therefore a
**content-authoring** gap, not a missing client table.

The wheel's coin slot is not an item: the Lua calls
`AddLootWheelItemData(22663, 0, coins, ...)` for the Coins icon and `9` for Stars. The coin amount
and the members-only bonus are server content.

## Current server: what is reused and what is missing

The current tip has a working **field-battle** route that is already a generic instance run:
`FieldBattle` rows are converted to `Dungeon` and appended to `content.dungeons`
(`src/encounter/field_content.rs`); `battle_screen` opens 41/114 + 41/107; `handle_battle_go`
accepts 41/108; `EncounterEngine::begin` returns the row's `zone` as a 31 transition;
`on_zone_ready` sends the entry burst and spawns the roster; `present_results` sends 39/18/47/19;
`exit` returns to the saved pose through 31. The route is gated by the shared busy table
(`Occupation::Encounter`), the combat job, the level floor, the item catalogue and the prize bag
preflight. Two-player isolation, reconnect and once-only payout are already tested.

| Capability | Existing owner on this tip | Reuse? | Missing for a real dungeon |
|---|---|---|---|
| Scene change and zone id | `Dungeon.zone`, `EncounterEngine::begin`, gateway `31`/`13` handling | **Yes** | Content row with the captured scene/zone, arrival, sky, geometry. No engine change for Misty. |
| Start screen / Go | `src/encounter/field.rs::battle_screen`, `details_packet`, `src/gameplay/field_battle.rs::handle_battle_go` | **Yes** | Real title/icon/description/objective/prizes; a correct 41/114 goal array. |
| Entry marker | `FieldBattle.spawner`, 35/2 AddNpc, marker wander, 26/8 | **Yes** | One marker at the captured gate. Suggested invisible `invisible_cube.adr` model 271 (R) so the existing static gate remains the visible prop. |
| Roster identity and spawn | `Dungeon.enemies`, `on_zone_ready` 35/2/35/5/35/12, `encounter` AI, 125 movement | **Yes** | Misty/Vale boss and elite model bindings, stats, positions; waves. |
| Goals | 41/114 goal list (`009bc380`), 45/1–4, 41/5, `BaseClient.MiniGameGoals` | **Partial** | One main + one bonus goal record with exact name/description/count, status text, bonus flag; phase order. The current 41/114 body was accepted for the start screen but its goal list has not been asserted against `009bc380`/`008fd770`. |
| Boss health | 32/9 + 35/5 (`BaseClient.Bosses`) | **Partial** | 39/62 `MiniGame.Bosses` only if the original top-centre bar is the minigame table; boss phases. |
| Results / quest credit | 39/18, 39/47, 39/19, `win_completed`, `CompleteDungeon`, `KillActorKind` | **Yes** | Real score rows, stars, KO count; 41/3 only if evidence appears. |
| Reward wheel | 39/45/46 + preview bundles, `prizes`, `battle_prizes_available` | **Partial** | Per-job wheel content, coin slot, member bonus, stars; one payout per instance. |
| Exit / death / reconnect | `return_location`, `exit`, `abandon_battle_for_reconnect`, saved `EncounterState` | **Yes** | Exit door/actor after the main goal; five-knockout/lives model; dungeon scene reconnect test. |
| Group run | 41/102/103 invites (`src/encounter_invite.rs`), World run instances, `bind_run_instance` | **Partial** | One shared enemy/objective simulation per invite group. Current T440 join gives each member their own copy of the enemies ("interim"). |
| Quests inside | `QuestEvent::CompleteDungeon`, `KillActorKind`, win event, 49/3/9/10 | **Yes** | Authored quest steps and dungeon enemy definitions with matching ids. Vale/Misty quest rows are currently empty shells or text-only mentions. |
| Private World dungeon | `World::enter_dungeon` exists but is **not called** by production code | **No** | Do not wire a second instance path. The run-instance binding already gives a private World instance for the solo run. |
| Vale scene assets | `lab/extracted/catalog.tsv` | **No** | Extract the 4 GCNK/GCK2 pairs and 64 DDS from the packs/manifest (or prove local packs suffice) before claiming Vale playable. |

## Misty Mountain: first playable slice

### Recovered facts

| Fact | Value | Evidence |
|---|---|---|
| Name / description / main goal | 71773 / 140757 / 140758 | client locale rows; exact strings |
| Bonus goal | 140755, "Search throughout the Misty Mountains to find the 4 pieces of the lost teachings of the Oracle! " (trailing space preserved) | client locale |
| Goal groups | 140748 primary, 140752 bonus | client locale |
| Difficulty / level gate | 3 / combat job level 5 | wiki + client requirement string 2145075890; level 5 text |
| Zone id | **200312** | captured POI 90, client `Zone ID` column |
| Scene | `gl_misty_mountain` | content JSON; GZNE + 16 chunks in extracted client |
| Gate prop | `bw_gate_01.adr` at 159.10722, 44.80690, 1074.39954; object id 1345842609 | `lab/reports/world-objects-20261015.json`, `pack_02/002508.gcnk` |
| Atlas marker | POI 90 at 158.58415, 44.72223, 1075.0023; icon 11970; notification 3 | captured POI stream |
| Combat bed | `gl_misty_mountainAreas.xml` `Bed` id 3455, sphere (273,40,408) r=300 | client Areas.xml |
| Start-panel picture | `icon_ui_combatstart_panelgraphic_snow.dds`, Images.txt id 314 (R candidate; no `mistymountain` panel art exists) | client Images.txt |
| Boss name | Snowface, locale 103637 | client locale; no model binding |
| Enemy name | Elemental Guardian, locale 103636 (alt 5102915) | client locale; no model binding |
| Friendly NPC | The Rose that Knows, locale 140646 | client locale / video |
| Prize names | 25 unique, all resolve to item ids (table below) | item catalogue `NameId` + locale |
| Coins / members / stars | 54 / 125 / 334 or less | wiki + guide; (R) amounts |
| Gear tier | III | wiki / guide |

Misty prizes (job: item ids), resolved from the guide names:

| Job | Item ids |
|---|---|
| Archer | 75010 (`Archer's Composite Bow of Stunning`), 75187 (`Archer's Necklace of Bark Skin III`), 75215 (`Archer's Power Shard of Regeneration III`), 10482 (`Battle Item Mystery Pack`), 75758 (`Cross Wind Archer Gloves`) |
| Brawler | 75040 (`Brawler's Anvil Hammer of Cheapshot`), 75252 (`Brawler's Necklace of Bark Skin III`), 75280 (`Brawler's Power Shard of Regeneration III`), 10482, 75879 (`Rubber Match Brawler Gloves`) |
| Medic | 75070 (`Medic's Bonesaw of Vitamins`), 75317 (`Medic's Necklace of Bark Skin III`), 75345 (`Medic's Power Shard of Regeneration III`), 10482, 76104 (`Trauma Care Medic Gloves`) |
| Ninja | 75100 (`Ninja's Scythe of Shadow Armies`), 75382 (`Ninja's Necklace of Bark Skin III`), 75410 (`Ninja's Power Shard of Regeneration III`), 10482, 76234 (`Shinobi Ninja Gloves`) |
| Warrior | 75130 (`Warrior's Battle Hammer of Whirlwind`), 75447 (`Warrior's Necklace of Bark Skin III`), 75475 (`Warrior's Power Shard of Regeneration III`), 10482, 76344 (`Charge Attack Warrior Gauntlets`) |
| Wizard | 75160 (`Wizard's Bone Wand of Vortex`), 75512 (`Wizard's Necklace of Bark Skin III`), 75540 (`Wizard's Power Shard of Regeneration III`), 10482, 76421 (`Practitioner Wizard Gloves`) |

The client has no model binding for Snowface/Elemental Guardian in the resolved content
(`enemies.json` has no row; T015 left Misty's Cray unresolved). The guide describes Snowface as a
frozen treant and the guardians as elemental constructs, but a name->model guess is not evidence.
For the first combat slice choose placeholder models from the shipped catalogue, record the choice
as (R), and verify them visually natively. Do not claim historical models.

### Minimum row for "enter and leave"

A content-only row using the existing field-battle route:

- encounter id `12500` (R; unique, not an opening id and not one of the existing 11 battle ids);
- title 71773, description 140757, objective 140758, difficulty 3, required combat level 5,
  icon 314 (R);
- zone: id **200312**, name `gl_misty_mountain`, sky `sky_shrouded_glade24.xml` (R; the GZNE stores
  materials, not sky), geometry 0 (R), position (273,40,408) (the `Bed` centre), rotation 0,
  tutorial false;
- spawner: unique GUID, invisible model 271 `invisible_cube.adr`, name 71773, nameplate icon
  11970, at the gate position;
- one dummy enemy at the bed centre, 1 HP, 0 damage (R; the loader requires a non-empty roster),
  or a real Elemental Guardian placeholder once its model is chosen;
- reward_coins 0, prizes empty for the entry slice, so the prize preflight cannot block it;
- source tags: `client` for the POI/zone/scene/name ids and the gate transform; `reconstructed`
  for the sky, geometry, arrival, model 271 and the dummy.

The scene load must be verified with the actual client. `lab/extracted` has all 16 Misty chunks;
`cargo test --no-run` and a content-check path are not enough.

### Acceptance test

Add `tests/misty_mountain_dungeon.rs` (one builder, reusing `tests/common/field_battle_fixture.rs`
and `tests/support/terrain.rs`; no helper copies):

1. Seed a combat job at level 5+, a save in zone 0, and a position at the captured gate.
2. Click the marker through the production `26/8`, assert 41/114 (`launch=false`) and 41/107, and
   assert the screen's title/description/difficulty/icon bytes.
3. Send 41/108; assert 31 carries zone **200312**, scene `gl_misty_mountain`, and the arrival
   position; assert the client-ready path then sends the 41/114 launched burst and the dummy 35/2.
4. Assert the dummy's 35/5 and the 35/12 relevance row.
5. Close with 26/42, assert a 31 return to zone 0 at the saved gate pose, no coin/item change, no
   active encounter, and the marker still there.
6. Refusal tests: forged 41/108 with no pending screen, Go from outside the marker radius,
   replayed 41/108, replayed 26/42, and a close while nothing is active. Snapshot the complete
   gateway state before and after each and assert equality (the refusal canary).

Canary: change the production row's zone name to `sg_random_encounter_clearing` (the old arena)
and the test must fail on the 31 scene assertion; restore. A second canary can change the arrival
to a point outside the `Bed` sphere and must fail the arrival/terrain assertion.

Native check: log in the seeded save, walk/teleport to the Shrouded Glade gate, click the marker,
watch Go, and record the scene loading screen and 5 s of stable ground at the arrival; close the
results and confirm the return to the gate. Capture the 31 scene name, the zone id, the player
position over time, and the absence of a fall or an `internal_error`. This is the first native gate
and should be done before any enemy content is added.

## Vale of the Ancients

### Recovered facts

| Fact | Value | Evidence |
|---|---|---|
| Name / description / main goal | 18143 / 77964 / 18908 | client locale rows |
| Bonus goal | 430443 "Destroy all of the Corrupted Artifacts!" (wiki count 9) | client locale + guide |
| Goal group | 140186 | client locale |
| Difficulty / level gate | 4 / combat job level 9 | wiki + client requirement string 1291047686 |
| Zone id | **2367** | captured POI 77, client `Zone ID` column |
| Candidate scene | `bw_corrupted_vale` (GZNE + Areas + four GCNK/GCK2 in the manifest) | client manifest; no scene name in the dungeon JSON |
| Gate prop | `sh_gates_01.adr` at -852.60608, -61.44305, 2307.46753; object id 1254797063 | `lab/reports/world-objects-20261015.json`, `pack_06/000935.gcnk` |
| Atlas marker | POI 77 at -850.7995, -61.67808, 2307.195; icon 11970; notification 3 | captured POI stream |
| Combat bed | `bw_corrupted_valeAreas.xml` `Bed` id 500, sphere (214,33,286) r=300 | client Areas.xml |
| Start-panel picture | `icon_ui_combatstart_panelgraphic_corruptedvale.dds`, Images.txt id 886 | client Images.txt |
| Boss name | Mysterious Stranger, locale 18891 | client locale; no model binding |
| Bonus target name | Corrupted Artifact, locale 419733 | client locale; no model binding |
| Exact enemies | Bristlewood Vulture `930055` (model 343, name 73654); Wolf `930071` (model 3284, name 374) | `enemies.json` + client locale/Models.txt |
| Unresolved enemies | Corrupted Hound / Greater Corrupted Hound / Gloam Hound / Corrupted Wolf / Mysterious Stranger / Corrupted Artifact | no exact Legacy NPC/model match |
| Prize names | 25 unique, all resolve | item catalogue `NameId` + locale |
| Coins / members / stars | 202 / 238 / 316 or less | wiki + guide; (R) amounts |
| Gear tier | IV | wiki / guide |

Vale prize item ids (25 unique names; the guide's six job rows share the Battle Item Mystery Pack
10482, and each job's shard/ring are per-job names):

| Job | Item ids |
|---|---|
| Archer | 75019, 75183, 75216, 10482, 75765 |
| Brawler | 75049, 75248, 75281, 10482, 75886 |
| Medic | 75079, 75313, 75346, 10482, 76111 |
| Ninja | 75109, 75378, 75411, 10482, 76241 |
| Warrior | 75139, 75443, 75476, 10482, 76351 |
| Wizard | 75169, 75508, 75541, 10482, 76430 |

Vale's asset risk is the first gap: the four GCNK/GCK2 pairs are in `Assets_manifest.txt` but not
in the extracted catalogue. Until they are extracted and delivered (or a local-pack load is
proven), entering `bw_corrupted_vale` will show missing terrain. This is a lab/asset task, not a
gameplay-module task.

## Build slices

Each slice is one builder, keeps new logic in the owning module, and leaves the four hubs at or
below their ceilings. Each slice has a focused test through the production gateway and a native
check. The slices are ordered by dependency, not by importance.

### 1. Enter and leave Misty Mountain

- **Change:** one `field_battles.json` row (id 12500) and one marker at the captured gate; zone
  `gl_misty_mountain` / 200312; arrival `Bed` centre; one dummy enemy; empty prizes.
- **Owner:** `src/encounter/field_content.rs` for the row type (only if a new field is needed),
  content JSON; otherwise no Rust change. Test in `tests/misty_mountain_dungeon.rs`.
- **Test:** the full 26/8 -> 41/114/107 -> 41/108 -> 31 -> 13/10 -> entry -> 26/42 -> 31 return
  path; snapshot equality for forced refusals; dummy AddNpc/HP asserted; no payout.
- **Native:** Misty scene loads, floor is stable, close returns to the gate.
- **Canary:** the old arena scene name or the wrong arrival makes the test fail.

### 2. Goals and start-screen data

- **Change:** data-driven main/bonus goal records for the row (Misty 140758 and 140755, bonus count
  4; Vale later 18908/430443, bonus count 9). Encode the 41/114 goal array exactly as `009bc380` /
  `008fd770` read it; send 45/1–3 for activation/progress/completion. Keep the existing 45/1–3
  shape if it already drives the visible counter; add only the missing goal rows/status text.
- **Owner:** `src/encounter.rs` / `src/encounter/field.rs` or `field_content.rs` (goal type next to
  `Dungeon`), not `gameplay.rs`.
- **Test:** exact 41/114 goal bytes derived from `009bc380`/`008fd770`; 45/1–3 byte equality; a
  two-goal run reaches `MiniGameGoals` row count 2. Native: "Goals (2)" panel with the right names
  and a 1/4 bonus counter.
- **Canary:** remove the bonus flag/count from production and the two-goal assertion fails.

### 3. Misty roster and boss

- **Change:** add Elemental Guardian (name 103636) and Snowface (name 103637) definitions with
  chosen models (R), health/damage, three guardians + one boss (R count) in the row; spawn at
  authored positions around the bed centre. Add 39/62 `MiniGame.Bosses` only if the native check shows the
  original top-centre bar needs it; otherwise keep the natively accepted 32/9.
- **Owner:** `data/content/enemies.json` for definitions; `field_battles.json` for positions;
  `src/encounter.rs` only for the 39/62 writer if required (small).
- **Test:** 35/2/35/5 for every roster actor, boss flag, one kill -> 39/18/47/19, 45 counts.
  Native: guardians and Snowface visible, boss bar, damage numbers.
- **Canary:** remove the boss flag or the 39/62 update and the boss assertion fails.

### 4. Phases, waves and the boss bar

- **Change:** an ordered wave list in the row (guards -> more guards -> Snowface), spawned only
  after the previous wave is down; phase index used only for server sequencing and goal text.
  No client phase packet exists; goals are the visible progress. Add 39/23/35/18 only when the
  five-knockout model is implemented.
- **Owner:** `src/encounter/field_content.rs` + `src/encounter.rs` (or a focused `encounter/`
  module). Reuse `GoalProgress`/`45` packets.
- **Test:** waves appear in order, a wipe stops the run, 39/18 win only after the last wave; no
  enemy from a later wave spawns early. Native: scripted sequence, boss bar hidden until the boss.
- **Canary:** reorder two waves in production and the order assertion fails.

### 5. Completion, wheel and exit

- **Change:** after the main goal, spawn the exit door/actor (the client shows an exit marker after
  the main goal), send the win 39/18 -> 39/47 -> 39/19 order, choose the wheel landing slice and
  send 39/45 before the client spins; accept 39/46 once and pay once. Add coins 54, the four/five
  job prizes for the active job, and optionally the members bonus/stars as data.
- **Owner:** existing `present_results`/`rewards`/`field_battle` paths; no new opcode.
- **Test:** one-win payout exactly once; replay 39/46/26/42/41/109 adds nothing; Close returns to
  the gate; inventory/coins/XP deltas match the row. Native: "YOU WIN!", wheel, landing,
  "Your Rewards", Close.
- **Canary:** pay on the second 39/46 and the replay assertion fails.

### 6. Vale scene assets and entry

- **Change:** extract the four GCNK/GCK2 pairs, the 64 DDS and the GZNE/Areas/TileInfo from the
  supplied packs or prove the client's local packs serve them; add the Vale row with zone 2367,
  scene `bw_corrupted_vale`, gate `sh_gates_01`, bed centre (214,33,286), one dummy.
- **Owner:** lab asset extraction + `field_battles.json`; no gameplay code beyond the row. This is
  the one slice that may need the lead because it touches asset delivery.
- **Test:** the same enter/leave test as slice 1 but with the Vale row; asset-delivery check
  resolves every manifest name. Native: Vale scene loads and the ground is intact.
- **Canary:** remove one GCNK row from the delivery index and the native check or the asset test
  fails.

### 7. Vale content

- **Change:** the same goal/roster/wheel slices for Vale: 18908/430443, Bristlewood Vulture 930055
  and Wolf 930071 (exact), placeholder Mysterious Stranger/Corrupted Artifact (R), 202 coins, the
  six job wheels, bonus count 9.
- **Owner:** content + `enemies.json`; no new engine.
- **Test:** the same focused suite parameterized by row; one win, wheel, return; boss/bonus counts.
- **Native:** full Vale run.

### 8. Group run

- **Change:** adapt the T440 invite path so an accepted invite joins the starter's **run**, not a
  second copy. The World instance and `bind_run_instance` already exist; the missing part is one
  encounter engine per run (shared enemy HP, goals, rewards, KOs) and fanout. Do not add a second
  handler; extend the existing `encounter_invite`/`World` flow.
- **Owner:** `src/encounter_invite.rs`, `src/bin/osfr_gateway/encounter_invites.rs`,
  `src/world.rs`; keep the hub changes at the existing dispatch lines.
- **Test:** two real sessions, one invite, one run; both see the same enemy HP and goals; one win
  grants each member exactly one parcel; a disconnect does not stop the other.
- **Native:** two clients, one invite, shared boss.

### 9. Quests inside the dungeon

- **Change:** author the empty Vale/Misty quest rows with real objectives (`complete_dungeon` with
  the new run id, and `defeat`/`KillActorKind` with the dungeon enemy ids), give them NPCs and
  turn-ins, and persist the completed run. Reuse `QuestEvent::CompleteDungeon` (already emitted
  from `win_completed`) and `KillActorKind` (already emitted per defeated enemy). Do not add a
  second quest credit path.
- **Owner:** `data/content/quests/*` and `src/quest_engine.rs` only if a missing objective kind
  appears; the win event is already in `Gameplay::apply_encounter`.
- **Test:** accept through the real NPC menu, enter, kill the goal enemies, see the counter rise,
  win, complete the `complete_dungeon` step, turn in through 49/13, reconnect; the quest counter
  and save survive. A loss or refusal does not advance.
- **Native:** one full quest inside Misty (and later Vale).

The slices intentionally stop at "one dungeon end to end" before adding all 75. Once the engine
fields (goals, waves, boss table, wheel) exist, the remaining dungeons are content rows and model
bindings, not new code.

## Threat note (secure-code-guardian)

A hostile client can send 26/8 with any actor GUID, 41/108 with any encounter/instance words,
36/10 with any ability/target, 41/109 or 26/42 at any time, 45/7 objective completion reports,
41/103 invite replies, 41/122 revive requests, and 39/46 wheel-stop reports. The future route must
refuse all of them without mutation:

- **26/8** resolves only a server-known marker, within its range, with a combat job, level and
  busy gate; an unknown GUID, a marker from another run, or a click from outside the radius starts
  nothing.
- **41/108** uses only the server-held pending screen for this session; the body's encounter,
  instance and GUID words are ignored. A replay after the screen was consumed, or from outside the
  marker radius, is inert.
- **41/109 / 26/42** close only the current run; a stale instance, a replay after close, or a close
  while nothing is active changes nothing. Rewards are applied once on the server's own HP/goal
  transition, never on Close.
- **36/10** resolves a live enemy in the current instance and server range; damage and rewards come
  from server HP, not from the client's claimed result. A forged target, a dead target, a
  cooldown/energy violation, or a duplicate hit grants nothing.
- **45/7** (client objective completion) must never grant a dungeon reward or advance a quest
  objective; only server events do. The client report is at most a UI acknowledgement.
- **41/103** must match a server-held invite row for this session, unexpired, unclaimed, with the
  inviter still in the run; the header words and GUID are untrusted. Accepting rechecks the busy,
  level, prize and bag gates and joins the server's run.
- **41/122** revives only this session's knocked-out state; the client sends no price or place.
  A paid revive charges the server price in the same commit; an insufficient wallet changes
  nothing.
- **39/46** pays only the current instance's already-chosen slice, once; a replay or a foreign
  instance pays nothing.
- **57/56** are server-to-client content; the client cannot ask for a zone or spawn. The server must
  not build a scene from a client-supplied scene name.
- **Persistence:** completed runs, unclaimed rewards, pre-activity pose and caps go through
  `character_store::replace_bytes` (atomic); no plain `fs::write`. A disconnect mid-run must save a
  resumable state or a clean return, never a half-paid transaction.

## Open gaps and contradictions

1. **No dungeon has loaded its own scene natively.** The closest accepted evidence is the opening
   cave (411) and the arena field battles. The 31 scene load for `gl_misty_mountain` /
   `bw_corrupted_vale` is the first native gate.
2. **Zone ids are capture-backed, not server-table-backed.** The client's POI data source column is
   literally `Zone ID` and the row order is fixed by `00b98290`, but no preserved dungeon 31 frame
   proves that `location_id` is the value the original server put in the zone packet. Treat 200312
   and 2367 as strong (R) until observed.
3. **Entrance actor identity is still unknown.** The captured POI position and the nearby static
   gate are recovered; the original interactive actor (model, nameplate, click radius, verb) is
   not. The invisible-model marker is a reconstruction; native test required.
4. **Misty/Vale boss and elite models are unresolved.** The names and locale ids are exact; no
   exact model binding exists in `enemies.json`/Legacy. Placeholder models must be labelled (R)
   and viewed natively. Snowface is described as a frozen treant; no model was proven to be it.
5. **The 41/114 goal array is not yet proven.** The reader expects a count plus 0xe8-byte records
   (`009bc380`/`008fd770`); the current body's goal-list block has not been matched field-for-field
   to that reader. The start screen and prizes work, but a two-goal tracker has not been asserted.
   Slice 2 must fix or prove this before phases.
6. **39/62 versus 32/9.** The client has two boss tables. The current path's 32/9 target bar is
   natively accepted; the original dungeon's top-centre `MiniGame.Bosses` bar is not proven. Do not
   send both; decide from a native frame.
7. **Five knockouts/lives are missing.** The score screen and 39/23/35/18 expose the concept; the
   engine ends on the first zero HP. A faithful dungeon needs lives, KO counting, and
   revive/respawn semantics; this is a later slice.
8. **Group runs are not shared.** T440 joins the World instance but each member currently fights
   their own copy of the roster. A true dungeon group needs one authoritative enemy/objective
   simulation.
9. **Vale assets are incomplete in the working extraction.** The manifest has the four chunks;
   `catalog.tsv` does not. Do not claim Vale playable until the scene loads and the assets are
   served.
10. **The prize wheel is content, not client data.** The names/ids are resolved, but the per-job
    assignment, coin amount, members bonus, star payout and wheel weights are server-authored and
    unrecovered. Use the wiki/guide values as (R) content and keep them in data.
11. **41/3 is unused.** Its role versus 39/18 is not established; do not add it without a capture
    or native observation.
12. **The current field-battle rows are not the dungeon scenes.** Bixie Hive, Stone Heart, Sheep
    Watch, Bandit Hideout and King of the Ring already prove the generic route but not the
    instance map. Keep them as regression gates.

## Sources and commands

Read for this task:

- `AGENTS.md`, `notes/Project Map.md`, `notes/Design/MMO Worker Contract.md`, the T440/T312/
  T373/T427 notes, `notes/Research/Pieces/059/060/061/063/065`, `notes/Research/Starting Dungeon
  Audit.md`, `notes/Research/Startup World Content.md`, `notes/Research/Content Batch Dungeons.md`,
  `notes/Research/World Enemies.md`, `notes/Autopilot/T015.md`, the Minigame Framework and Combat
  guides, `src/encounter.rs`, `src/encounter/field.rs`, `src/encounter/field_content.rs`,
  `src/gameplay/field_battle.rs`, `src/world.rs`, `src/gateway.rs` (zoning), and the current
  `tests/*dungeon*.rs`/`field_battle_roster.rs`/`two_player_dungeons.rs`.

IDA commands (read-only, `lab/autopilot/ida_query.py`):

```text
decompile 00b98290   # POI row reader: 13 dwords, zone id is the third
decompile 008b6bc0   # 26/8 interaction constructor
decompile 009bdd70   # 41/114 details reader
decompile 009bc380   # 41/114 goal list: count + 0xe8 records
decompile 008fd770   # goal record fields
decompile 009be130   # 45 objective dispatcher
decompile 009bd8f0   # 41/5 goal synchronization
decompile 008b6e70   # 41/108 Go constructor
decompile 00938f00   # 41/108 sender fills encounter/instance
decompile 009b0cc0   # 41/107 -> MiniGame:SetMiniGameReady
decompile 008a31a0   # 41/109 exit constructor
decompile 009b9e20   # 39/45 wheel landing constructor
decompile 009bda80   # 39/46 client stop report
decompile 009b9eb0   # 39/68 reward update constructor
decompile 00c48c90   # 39/62 boss message handler
decompile 00c48620   # 39/62 boss body reader
decompile 008e4ae0   # 31 zone transition reader
```

Supporting reads: `009beb70` (HandleMiniGameStart) and `009b0db0` (41/6 exit). The results/lifecycle
readers 39/17, 39/18, 39/47, 39/19, 26/42, 41/2, 41/106 and 41/133 are pinned by the existing
accepted notes and tests cited above; this run re-decompiled the entry, goal, wheel, boss and POI
readers. The graph server project `C-Users-<user>-Documents-Projects-CartoonMMO-autopilot-int` was queried
for dungeon instance/entry and field-battle symbols; `trace_path` rejected the method names it was
given, so the callers were taken from direct source reads and the existing test notes.

Data checks (read-only Python, scratch under `%TEMP%\T748`):

- mapped 41 captured dungeon POIs to zone ids/positions;
- resolved 1,628/1,667 `dungeons_unresolved.json` item names through item `NameId` + locale;
- verified the Misty/Vale gate props and scene assets in the world-object catalogue, `Models.txt`,
  `Assets_manifest.txt` and `catalog.tsv`.

No web fetch was needed; the pinned wiki/guide corpus and the supplied client were sufficient.
No source, content or data file was changed.

## Structure

- Graph queries: `search_graph` for "dungeon instance enter dungeon encounter launch" and the
  `.*[Dd]ungeon.*` name pattern; the results identified `World::enter_dungeon` (unwired),
  `EncounterEngine::dungeon`, `Gameplay::apply_encounter`, the field-battle tests and the dungeon
  content validators. `trace_path` returned "function not found" for the qualified method names, so
  the final caller checks used `grep`/source reads.
- Modules touched: none. The diff is this note and `notes/Autopilot/T748.md`.
- Net lines per hub: `src/gameplay.rs` 0, `src/bin/osfr-gateway.rs` 0, `src/encounter.rs` 0,
  `src/gateway.rs` 0.
- Deleted: none.

## Skills applied

- **ponytail (full)**: the design reuses the existing field-battle/encounter route and the World
  run instance instead of proposing a new dungeon manager; the smallest first slice is one content
  row, not a module.
- **ponytail-review**: reviewed the design for speculative layers and cut a separate dungeon
  service, a second entry handler and a new zone table; the three data fields (goals, waves, boss
  table) are added only in the slice that needs them.
- **secure-code-guardian**: threat note above; every future client-triggered path has an explicit
  server-owned gate and a refusal-changes-nothing rule.
- **reverse-engineering**: IDA decompilation of the packet readers/handlers and the POI row reader.
- **codebase-memory**: graph queries on the integration project for the owners and callers.
- **game-research-websearch**: not used; the pinned wiki/guides and existing client exports were
  the content source, as the task allows.
