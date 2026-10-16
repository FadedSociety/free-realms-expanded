# NPC Placement

Updated 2026-10-13 for W56.

## Result

Regional NPC rows now use a unique, catalog-backed Legacy spawn where the locale name and spawn identity join unambiguously. Legacy positions are source data, not coordinates proved by the native capture.

| Regional placements | Before W56 | After W56 |
|---|---:|---:|
| Legacy-data position and heading | 0 | 578 |
| Reconstructed placement | 1,199 | 621 |
| Total regional rows | 1,199 | 1,199 |

The 621 retained reconstructed rows consist of 544 with no Legacy name candidate, 74 ambiguous rows, and 3 unique identity candidates whose model files are absent from the client catalog.

## Recovered sources and confidence

- Native AddNpc reader is PlayerUpdatePacketAddNpc_ReadBody at 00921a10, confirmed; see Research/Startup World Content.md, ?NPC spawns, 35/2?. The typed parser crosswalk is prototype/osfr-tools/src/startup_world.rs:599-610; parse_add_npc reads actor ID, name ID, model ID, position and rotation at lines 868-881. Native position and rotation are float4 values.
- The capture input is lab/content/startup-snapshot/startup.json, world.npc_spawns (23 rows). Each capture-confirmed extra row cites its actor_id.
- Legacy built data is FreeRealms-Legacy/src/Scripts/Zone/FabledRealms.lua (4,253 spawn calls) plus src/Resources/Npcs.json (4,253 resource rows). Every regional Legacy placement cites the exact Lua line and Npcs.json Id in its source and field_sources. No Legacy source code was copied; only built-content data was read, with attribution.
- NameId/Name agreement against lab/reports/locale-id-to-text.json: all 1,659 named spawn rows have a resolving locale ID; 1,655/1,659 match the text exactly (99.76%); all 1,659 match after Unicode normalization and case folding. There are 1,046 distinct named NameIds.
- ModelFileName agreement against lab/extracted/client-assets/catalog.tsv: 3,956/4,253 spawn rows (92.9%) have a matching catalog filename. The other 297 rows use 132 absent filenames.
- No supplied Legacy file says its spawn coordinates came from this client capture. Do not describe the Legacy positions as capture-derived.

## Join rules (R)

The original 1,199 regional records had no model_id. The join uses regional display name -> normalized locale text -> Legacy NameId and Name, then normalized name as fallback. Normalization uses Unicode NFKD, case folding, and removal of non-alphanumeric characters. A Legacy row is assigned only when one spawn line maps to one regional record and its model file exists in catalog.tsv. The matched row receives position, raw heading, model_id, model_file_name and NameId from Legacy, plus an explicit legacy-data source tag. Its rotation is the prior Y-axis yaw-quaternion conversion from raw heading; that representation is marked (R), and the exact heading remains available.

No model-plus-region fallback was applied: no regional input had a model_id, and no verified region geometry exists for classifying the Legacy coordinates. 632 regional rows had one candidate spawn line before cross-row collision checks. Fifty-one rows share one candidate line with another regional row (25 shared lines), so those rows stay reconstructed. Another 23 rows have multiple candidate spawn lines. Three otherwise unique rows stay reconstructed because their model filenames are absent from the catalog.

### Ambiguous regional joins left unchanged

Multi-spawn candidates (region; FabledRealms.lua line numbers):

- Grave Elemental (Blackspore Swamp): 365, 367, 368, 3466, 3527, 3528
- Mucklewhump (Blackspore Swamp): 244, 3523
- Petra (Blackspore Swamp): 232, 3479, 3518
- Tormented Spirit (Blackspore Swamp): 357, 358, 359, 366, 371
- Troll bone shaman (Blackspore Swamp): 1892, 1893
- Clack (Briarwood): 738, 1426, 1751, 1767, 1783
- Darkthorne (Briarwood): 4216, 3507
- Wompugg (Farnum Farmstead): 392, 1981
- Archer on Patrol (Greenwood Forest): 2200, 2227
- Bixie Guardian (Highroad Junction): 1657, 1658, 1663, 1664, 1666, 1670-1676
- Bixie Magus (Highroad Junction): 1659-1662, 1665, 1667-1669, 1677-1679
- Sammy (Merry Vale): 3517, 3530
- Queen Valerian (Sanctuary and Shrouded Glade): 3500, 4184
- Burglz (Seaside): 638, 1863
- Headmaster Merk (Shrouded Glade): 2508, 3509
- Cronyn (Snowhill): 1062, 2359
- Sorin (Snowhill): 7, 1060, 1061, 2349
- Therin (Snowhill): 5, 6, 463, 464, 2344
- Starlight (zone unknown): 2685, 2686
- Ninjawug Archer (Wugachug): 2063, 2073, 2912, 4157, 4162, 4164
- Ninjawug Bixie-rider (Wugachug): 2068, 2071, 2911, 4163, 4165, 4166
- Ninjawug Blademaster (Wugachug): 2061, 2062, 2066, 2069, 2070, 2072, 2910, 4158-4161

Single-line collisions also remain ambiguous. Each line below is the sole candidate for every named regional row on that line:

| Lua line | Regional rows sharing the candidate |
|---:|---|
| 203 | Ceard Bauchlog (unknown zone; 9500000012), Ceard Bauchlog (Shrouded Glade; 9500000658) |
| 213 | Sera / Bella (unknown zone; 9500000207), Sera Bella (Shrouded Glade; 9500000712) |
| 1083 | Doctor Gordon (Sanctuary; 9000000073), Doctor Gordon (unknown zone; 9500000574) |
| 1121 | Captain Sunspark (Seaside; 9500000565), Captain Sunspark (Shrouded Glade; 9500000657) |
| 1422 | Kade Carmichael (Briarwood; 9500000153), Kade Carmichael (Wilds; 9500000912) |
| 1595 | Admiral Fishfinder (Briarwood; 9500000083), Admiral Fishfinder (unknown zone; 9500000547) |
| 1729 | Mustang Alli (Snowhill; 9500000759), Mustang Alli (Wilds; 9500000923) |
| 1731 | Burt 'Wrecking' Crewe (Snowhill; 9500000731), Burt 'Wrecking' Crewe (Wilds; 9500000884) |
| 1732 | Chad Fastflutter (Snowhill; 9500000735), Chad Fastflutter (Wilds; 9500000886) |
| 1733 | Petal Oceanmist (unknown zone; 9500000613), Petal Oceanmist (Wilds; 9500000930) |
| 1758 | Linx Redline (Briarwood; 9500000164), Linx Redline (Wilds; 9500000916) |
| 1824 | Ty (Lakeshore; 9000000223), Ty (Seaside; 9500000641) |
| 1959 | Aunt Daisy (Cobblestone Village; 9500000244), Aunt Daisy (Farnum Farmstead; 9500000267) |
| 2026 | Necronomicus (Merry Vale; 9500000413), Necronomicus (Sanctuary; 9500000508) |
| 2029 | Derp Axemaker (Snowhill; 9000000069), Derp Axemaker (Wilds; 9500000892) |
| 2032 | Shimmer Shinyaxe (Snowhill; 9000000201), Shimmer Shinyaxe (Wilds; 9500000942) |
| 2128 | Sakka (Snowhill; 9500000766), Sakka (Wilds; 9500000937) |
| 2292 | Bruce (Sanctuary; 9500000455), Bruce (Snowhill; 9500000729) |
| 2322 | Baron von Darkcheat (Seaside; 9500000556), Baron von Darkcheat (Snowhill; 9000000020) |
| 2378 | Jeni Shortfuse (Sanctuary; 9500000498), Jeni Shortfuse (Snowhill; 9000000115) |
| 2454 | Vicki (Briarwood; 9500000229), Vicki (Farnum Farmstead; 9000000230) |
| 2509 | Golmie Greasebeard (Shrouded Glade; 9500000686), Golmie Greasebeard (Sunstone Valley; 9500000821) |
| 2833 | One of Three (Cobblestone Village; 9500000253), One of Three (Greenwood Forest; 9500000317), One of Three (Sanctuary; 9500000510) |
| 2834 | Two of Three (Highroad Junction; 9500000338), Two of Three (Stillwater Crossing; 9500000795) |
| 3264 | Chatty (Greenwood Forest; 9500000294), Chatty (unknown zone; 9500000330) |

The three unique but uncatalogued model candidates left reconstructed are Robbie (line 3257, model 1027, robgoblin_m_bobgoblin.adr), Rumble King Don (line 3365, model 4219, dwarf_m_rumblekingdon.adr), and General Wugashima (line 2909, model 4468, chugawug_m_ninjawug_bixie_rider_general_01.adr).

## Capture comparison, all 23 rows

The table compares 3D XYZ distance. Nine rows have a same-model Legacy spawn within 0.6 m and are copied to npc_spawns_extra.json at the exact captured position/rotation/name ID. Two further warpstone candidates are close but above that cutoff. The other 12 have a nearest positional Legacy row 11.875-136.071 m away; some same-model rows are much farther. The evidence does not establish whether those actors moved, were phased, or differ by client/content version.

| Captured locale name and model | Nearest same-model Legacy row | Nearest Legacy row by position | Assessment |
|---|---|---|---|
| Ed Sullenfangs, 3311 | none | Judd, model 669, 22.054 m | model absent |
| Jake Blackforest, 3296 | none | Judd, model 669, 16.153 m | model absent |
| Default Housing NPC, 1062 | line 2402, bw_boombox_01.adr, 828.469 m | unnamed model 343, 12.064 m | same model far; nearby different model |
| unnamed, 4579 | none | unnamed model 343, 12.954 m | model absent |
| Halloween Harry, 1692 | line 4249, Halloween Harry, 1,133.682 m | unnamed model 343, 11.875 m | same named model far; nearby different model |
| unnamed, 1013 | none | unnamed model 343, 12.274 m | model absent |
| Briarheart Warpstone, 280 | line 1596, same name and NameId, 0.574 m | same row | capture copied |
| Bristlewood Warpstone, 280 | line 1392, same name and NameId, 0.753 m | same row | close, above 0.6 m cutoff |
| Briarwood Warpstone, 280 | line 1295, same name and NameId, 0.395 m | same row | capture copied |
| Greenwood Forest Warpstone, 280 | line 3245, Greenwood Warpstone, NameId differs, 0.262 m | same row | capture copied; client label retained |
| Merry Vale Warpstone, 280 | line 919, same name and NameId, 0.114 m | same row | capture copied |
| Lavender Coast Warpstone, 280 | line 2165, same name and NameId, 0.338 m | same row | capture copied |
| Cobblestone Village Warpstone, 280 | line 1001, same name and NameId, 0.144 m | same row | capture copied |
| Lakeshore Warpstone, 280 | nearest model 280 is Briarwood Warpstone at line 1295, 689.046 m | unnamed model 1931, 132.617 m | wrong-area same model |
| Shrouded Glade Warpstone, 280 | line 916, same text, different NameId, 0.282 m | same row | capture copied |
| Highroad Junction Warpstone, 280 | line 1629, same name and NameId, 0.232 m | same row | capture copied |
| Wugachug Warpstone, 280 | line 918, same name and NameId, 0.752 m | same row | close, above 0.6 m cutoff |
| Seaside Warpstone, 280 | line 917, same name and NameId, 0.142 m | same row | capture copied |
| Sanctuary Warpstone, 280 | nearest model 280 is Cobblestone Warpstone at line 1001, 497.983 m | unnamed model 800, 64.509 m | wrong-area same model |
| Snowhill Warpstone, 280 | nearest model 280 is Highroad Warpstone at line 1629, 528.306 m | unnamed model 17, 136.071 m | wrong-area same model |
| Kellin Warpwatcher, 468 | line 302, unnamed fairy_druid_m.adr, 991.863 m | unnamed model 800, 69.673 m | same model far; nearby different model |
| Vaelen Warpwatcher, 468 | line 133, Vidor Stormshroud, model 1356, 636.046 m | unnamed model 17, 130.427 m | same model far; nearby different model |
| Boom Box, 185 | line 664, unnamed sh_primitive_radio_01.adr, 1,655.140 m | unnamed model 17, 101.776 m | same model far; nearby different model |

The 9 client-copied extra transforms cite startup.json actor IDs individually. The captured Greenwood display string is ?Greenwood Forest Warpstone?; the Legacy name is ?Greenwood Warpstone?. The captured value takes precedence.

## Reference reconciliation

Eighteen previously unresolved NPC references now point through regional reference_ids to a unique regional row and one Legacy spawn. Their displayed name resolves through locale and does not collide with another regional row. Their source is marked (R) because some quest/job context zones differ from the regional row's zone or the row has no zone. Zone labels remain unchanged and were not used to invent coordinates. The NPC name/spawn association is not proof that a static actor is present in every context zone.

- 9000000012 Ari the Fish -> npc_id 9500000869, Lua line 2805
- 9000000017 and 9000000018 Aster -> npc_id 9000000243, line 2278
- 9000000049 Cleetus Farnum -> npc_id 9000000050, line 1975
- 9000000064 Crash Davis -> npc_id 9000000289, line 1786
- 9000000066 Dane Blueshift -> npc_id 9500000891, line 1782
- 9000000077 Eddie Eagle Eyes -> npc_id 9500000245, line 2992
- 9000000078 Emahl -> npc_id 9000000317, line 2146
- 9000000088 Fernando Flexsteel -> npc_id 9500000785, line 1693
- 9000000089 Fizzlesticks -> npc_id 9000000324, line 107
- 9000000099 Glerga -> npc_id 9000000332, line 2730
- 9000000105 Herme Fickleleaf -> npc_id 9000000347, line 2080
- 9000000116 Johnny Thunder -> npc_id 9500000911, line 1753
- 9000000135 Lugabow -> npc_id 9000000387, line 2210
- 9000000204 Simone -> npc_id 9000000467, line 825
- 9000000212 Talespinner Yvelle -> npc_id 9500000427, line 1037
- 9000000214 Terrence -> npc_id 9000000488, line 2076
- 9000000221 Tucker Thunderjolt -> npc_id 9000000495, line 1836

Only giver_npc_id values changed in quest files: 33 occurrences. Resolved reference IDs increased 277 -> 295; unresolved references decreased 188 -> 170 (118 NPC, 48 unknown, 3 ambiguous, 1 non-NPC). Quest-giver occurrences are now 436 resolved and 215 unresolved of 651. Trainer references are now 150 resolved and 22 unresolved of 172 unique IDs; jobs.json was not changed.

## Artifacts and validation

- Regional JSON: 578 records use Legacy placement/model/name data; all written model files are in catalog.tsv and all written NameIds resolve in locale.
- data/content/npc_spawns_extra.json: 3,265 unlisted Legacy spawns with catalog-backed model, name/NameId when present, position and heading. Nine rows carry client transforms and actor_id provenance. 286 further unlisted Legacy rows were omitted because their model filenames are absent from catalog.tsv.
- tests/content_npc_spawns.rs checks model catalog membership, locale IDs, finite positions/headings, source tags and captured XYZ/W/quaternion equality within 0.01.
- tests/content_npc_placement.rs checks regional placement classifications and capture-linked transforms.
- tests/content_npcs.rs and tests/content_jobs.rs verify source tags, reference aliases and quest-giver links.
- The strict v1 SourceTag at src/content.rs:21-34 has only client/video/wiki/reconstructed. The temporary content_npcs test adapter maps legacy-data to reconstructed only for its scratch ContentDb package. No production loader changed. Direct deserialization needs an explicit variant such as #[serde(rename = "legacy-data")] LegacyData(String) in SourceTag and that variant added to evidence(); lead integration must decide whether to add it.
- From prototype/osfr-tools, the scoped build command was:

        $env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W56'; & '..\..\scripts\cargo.ps1' test --test content_npc_spawns --test content_npc_placement --test content_npcs --test content_jobs

- Scoped formatting from prototype/osfr-tools used the private rustfmt.exe with --edition 2024 on tests/content_npc_spawns.rs, tests/content_npc_placement.rs and tests/content_npcs.rs. All four targets passed, one test each. Existing dead-code warnings in chess.rs and housing.rs remain unrelated.
- A count-output rerun with content_npcs '--' '--nocapture' printed 295 resolved / 170 unresolved references and 436 resolved / 215 unresolved quest-giver occurrences; the test passed.
- Data parsing and writing used an inline Python 3.12 standard-library script through PowerShell here-string piping (@' ... '@ | python -), from the project root. It asserted 4,253 resource rows, 4,253 parsed spawn calls and unique spawn GUIDs; no helper script was persisted. Inputs and output paths are listed above. A separate inline audit checked every written model file, locale ID, finite transform, and capture-linked XYZ/W/quaternion.

## Open limits

- The Legacy source is AGPL-3.0. This task reads built data only and copies no code; data provenance is attributed per row.
- Legacy positions are not proven to be client-capture-derived. Only nine of the 23 startup AddNpc rows closely match a same-model spawn; the other rows retain identity/version/phase uncertainty.
- 621 regional rows still use reconstructed placement. They include 544 with no name candidate, 74 ambiguous joins and 3 catalog-missing models.
- NPC placements and the extra file are data artifacts; the strict runtime loader does not yet consume them. Native gameplay acceptance was not run.
