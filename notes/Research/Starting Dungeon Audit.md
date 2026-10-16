# Starting dungeon audit

The supplied opening story points to **Briarwood Caverns / Darkthorne**, followed by Farnum Farmstead and Cobblestone. The first implementation uses the supplied new-player cave map and a reconstructed defense encounter. Original dungeon scripting, spawn tables, rewards and combat balance have not been recovered. Native zoning and a complete native encounter remain acceptance gates.

## Evidence and selected route

- Pinned FreeRealms-Legacy `StartingZone.SendAdventurersJournalInfo` places Briarwood Caverns at hub 1, with quests 2513 and 2514 preceding Farnum and Cobblestone.
- Supplied locale 4419563/4419564 names *The Last Best Hope* and asks the player to defend Darkthorne in the Heartseed Chamber. Locale 4419610 is the defense objective. Locale 5100071 describes Darkthorne sealing the Gloam Overlord and sending the player to Farnum; the Overlord is not described as killed.
- Supplied locale 5100532 links “Encounter 411” to Darkthorne's teleport. This supports the progression event 411. **Zone ID 411 is a local assignment**, not a recovered zone definition.
- Supplied `Resources/ObjectTerrainData.xml:33` maps `NPXCavern` to `sg_npx_cavern_01` and `sky_blackspore24.xml`. The extracted `sg_npx_cavern_01Areas.xml` contains first, second and final tutorial room bounds. These corroborate the map's tutorial purpose. They are ambient sound/effect areas, not NPC spawn records.
- The initial candidate spawn `[287, 28, 286]` lies inside the first room's area bounds `[261,12,268]` to `[316,42,304]`. This is **not a recovered spawn or floor proof**. The existing `fr-adr-toolkit` reader rejects the room meshes' 44-byte vertex stride; its supported 36/52-byte layouts were not silently extended.
- `ObjectTerrainData.xml:32` separately maps `NewbieZoneShowdown` to `sg_newbiezone_showdown`. That map and the Mac story belong to the final starting-area showdown. Its standalone native zoning/spawn is not yet verified; the initial implementation places the Mac fight at an explicitly local outdoor location instead.

Do not substitute `bw_briarheart_caverns` or unrelated Cracked Claw Caverns merely because their names contain “caverns.” Also do not use supplied `MiniGameData.txt` row 411 as the cave definition: it is type 16 with locale 13340, **Blackheart's First Round**, a TCG activity. The story's encounter reference and this table do not establish the same namespace/version. The launch packet therefore carries inline encounter details.

## Community protocol research

Pinned Sanctuary and Legacy provide generic zoning and class/profile packets, but their combat-zone classes contain no original encounters. Closed, unmerged Sanctuary [PR 74](https://github.com/Open-Source-Free-Realms/Sanctuary/pull/74), [PR 76](https://github.com/Open-Source-Free-Realms/Sanctuary/pull/76), and [PR 77](https://github.com/Open-Source-Free-Realms/Sanctuary/pull/77) add useful packet layouts. Their source is retained with exact hashes and revisions in the private `lab/references/Sanctuary-combat-pr-research/SOURCE.json`; it was not copied into Rust. PR status and the absence of original encounter scripts remain material limits.

The original Rust encounter module serializes these documented families: 41/114 inline launch details; 41/106 state; 41/107 zone ready; 41/2 player enter; 41/132–133 combat gates; 39/17 start and 39/18 result; 39/19 state removal; 45/1–3 objectives; 62 combat rules and defaults; 35/2 NPCs and 35/5 health; **standalone opcode 125** movement; 32/7 attacks and 32/9 boss display. Player health is **38/1**, with current and maximum i32 fields. Encounter headers include two i32 identity fields after their u16 opcode/subopcode. MiniGame and objective subopcodes are one byte.

A private .NET oracle compiles those pinned serializers directly. The Rust private-content test matched the 98-byte begin-zoning, 36-byte movement, 50-byte attack feedback, 24-byte enemy health, 423-byte launch details, 45-byte rules, 12-byte player health, 15-byte start, and 11-byte goal activation. Six synthetic tests additionally cover loading gates, individual enemy persistence, once-only rewards, loss/retry, malformed requests, cooldown/range, and phase validation. This verifies serialization and local rules; it does not prove the native dungeon works.

## Local implementation scope

Private `lab/references/rust-opening/encounters.json` holds all supplied-derived data, the AddNpc template hash, and reconstruction provenance. `scripts/prepare-opening-encounters.py` prepares it after the opening progression content. It preserves the earlier `rust-gameplay` milestone.

The cave currently uses a compact reconstructed defense: three Gloamlings and an Overlord representation, a Darkthorne return interaction, locally chosen health/damage/timing, and a once-only 50-coin plus one Wheeler on the Cob reward. Reducing the Overlord representation to zero health signals Darkthorne's seal; this is not evidence of the original boss's death or original combat mechanics. The engine also owns five outdoor Hooligan Wolves, one Gloam Construct and Mac, gated by accepted opening quest stage. The five-wolf count follows the recovered wiki quest; their individual placements and spawn bindings are local choices.

The caller stages cloned encounter, inventory and progression services, persists them together, and then sends resulting packets. Entering and exiting emits a typed zone transition; exit restores the saved world coordinates and emits the completion event only after a win. The caller must suppress the earlier training combat and duplicate outdoor enemy spawns while this opening content is active.

## Research limits and next gates

The private audit records the exact search queries and failures. Searches included the XeNTaX/ZenHAX archives, GitHub, and GZNE/GCNK/QuickBMS tools. No applicable original encounter script or existing GZNE reader was located. The existing pack reader was already validated; no custom terrain parser was added for this work. Author-fork PR raw URLs returned 404; the preserved commit objects remained available through the upstream repository and were fetched there.

Original designer [Chris Molina's portfolio](https://auburn-sky.com/game-portfolio/Game%20Port%20HTML/FreeRealms.html) supports an important distinction: modular dungeon art and terrain do not supply the population, AI or event script. Those systems need separate evidence and native testing.

Required next checks are native cave loading and ground position, enemy visibility and attacks, retaliation/knockout/retry, native objective/result/return behavior, once-only inventory rewards, and restart restoration. The original tutorial's full room sequence, scripted barriers, chugawug rescue, potion/ultimate training, full Overlord mechanics and authentic reward tables remain unresolved.

## 2026-10-08 — Original room geometry and loading preparation

A second community reader resolves the earlier mesh-layout limitation. [EDITzDev/ForgeLightToolkit](https://github.com/EDITzDev/ForgeLightToolkit), pinned at `9c06714c5d79d9152c322814bc904abad944e06e` (MIT), successfully reads all 23 selected cave assets: 16 GCNK chunks and seven DME room/water meshes. Its original GCNK, DME and material-layout sources run unmodified in a private headless .NET helper with small Unity value/mesh shims. The supplied `materials_3.xml` provides the original vertex layouts. This is ForgeLight data imported by a Unity tool; it does not imply the game uses Unity.

The seven original room actors share position `[380,5,370]`, rotation `[-pi/2,0,0]` and scale1. The community importer's Euler mapping gives original game coordinates `X=380-localZ`, `Y=5+localY`, `Z=370+localX`, before its whole-world Unity Z flip. Vertical triangle queries now give the following **render-floor evidence**, superseding the bounds-only placement hypothesis above:

| Placement | X / configured Y / Z | Original room floor Y | Clearance above floor |
| --- | --- | --- | --- |
| Player spawn | 287 / 28 / 286 | 27.953926 | 29.748 |
| Guide | 284 / 28 / 284 | 27.953930 | 29.834 |
| Enemy1 | 292 / 28 / 289 | 27.953920 | 24.709 |
| Enemy2 | 295 / 28 / 292 | 27.953920 | 25.850 |
| Enemy3 | 298 / 28 / 295 | 27.962242 | 24.035 |
| Current boss | 304 / 28 / 299 | 27.480914 | 8.929 |
| Suggested inward boss | 299 / 28 / 297 | 27.991154 | 23.056 |

The original water plane is Y27.625334. All sampled spawn-to-guide/three-enemy routes have dry, nearly level render floors, using intervals of at most0.5units and four additional0.5unit offsets per sample. The inward boss route has170 samples, all dry, floor27.953920–28.005610 and minimum22.861units overhead. The current boss route has14 wet/shallow samples out of220 and intersects a nearby lower-room surface at23.434254. Prefer the inward position in a future content revision after the active run ends. **This investigation changes no live encounter data.**

Original static door01 is at `[314.99448,28.665304,288.69412]`; door02 is at `[360.06262,33.38316,345.52478]`. The compact encounter stays before door01 and does not depend on unknown original door-opening scripts. The original `STING_TUTORIAL` area near `[273.5,27,287.5]` also lies over a dry floor27.859455, corroborating the map transform; it is not a recovered spawn.

Collision is still a material limit. ForgeLightToolkit's `GcnkFile.LoadCollision` is explicitly TODO, and the separate original CDT begins `CDTA`, not a directly loadable Bullet serialization. Vertical mesh samples cannot prove horizontal wall clearance or native collision behavior. No original NPX server script or dynamic NPC spawn table was recovered. Native walking must verify the proposed reachable floor and actor placement.

The loading order is supported by pinned Sanctuary source: `Player.Teleport` sends begin-zoning31; client13 invokes initial-data sending, including14 and38/26; client10 marks the player visible and calls the finished-loading hook. The Rust gateway similarly resets ready/loaded after31, sends Ready templates on13 and initializes cave actors/encounter on10. Zone-details43 is sent during initial login; it is not automatically repeated after every31. This is source/serializer evidence, not proof of a successful native cave transition. Native acceptance must show31→13→ready-data→10, stable cave Y, approachable actors, encounter completion, world return and correct restart state.

Private reproducible evidence is `lab/reports/cave-acceptance-evidence.json`, `lab/reports/cave-geometry/floor-path-analysis.json` and `lab/tools/cave-community-reader/{Reader.csproj,inputs.json,analyze_floor.py}`. Source and original asset hashes are recorded there. No game was launched or driven for this analysis.

## 2026-10-08 — Native cave entry and exact boss asset recovery

Native run `20261008T073831Z` reached the cave's WorldReady state and supported combat, as reported by the live-test operator. It exposed a separate asset-delivery failure: the original boss model3956 requests `gloam_m_boss.adr`, which is absent from the supplied extracted catalog. Its mesh and material are present. The native log records the exact request `/127/gloam_m_boss.adr.z?3361679076` failing on the initial loopback cache, explaining an invisible actor without requiring a model-ID change.

The exact [public community CDN asset](https://osfr.editz.dev/assets/127/gloam_m_boss.adr.z?3361679076) was recovered through ordinary anonymous HTTP200. Its original envelope is3577 bytes, matching the supplied manifest; decoded raw data is12204 bytes with CRC32 **3361679076**, also matching that original manifest. Raw SHA256 is `ac842259cb0d62648e8c4083df6e1edf7d381d8d6681185ecd8a7ff830c477ba`. The verified raw file and delivery index are under `lab/references/asset-cache/http-raw`. The initial recovery's cache-index temporary suffix failed after validation; the writer was corrected and the same CDN file refetched and verified before publishing the index. The corrected report records this repair explicitly.

Unmodified [edenfps/fr-adr-toolkit](https://github.com/edenfps/fr-adr-toolkit), pinned at `5acfdf8d6006fa6f43d1d3605f076a6be7393e72`, resolves the exact ADR to `gloam_m_boss.dme`, `gloam_m_boss.dma`, `gloam_m_boss.dds` and `forestgiant.dsk`. Of38 explicit referenced assets including the ADR,36 are available and all36 match both the supplied pack catalog CRC and the original asset manifest CRC. Two animation clips remain absent: `gloamconstruct_boss_com_spawn.gr2` and `gloamconstruct_boss_com_despawn.gr2`. Idle, movement, attack and death clips are available. This proves the core original dependencies, not native rendering or complete animation acceptance; a retry must verify visibility.

The preserved model3972 illusion variant was also audited, but is unnecessary once the exact base actor is delivered. It uses the same mesh/skeleton and a distinct illusion material. No variant bytes were served under the original actor's filename or CRC, and no identity, content, encounter state or reward state was changed by the recovery.

Six first-room terrain textures remain missing: `sg_npx_cavern_01_Tile_000_000_2_{1,2,3}.dds` and `_3_{1,2,3}.dds`. Their exact native shard URLs returned404; none of these six textures or the base ADR appeared in the18 previously cached public pack indexes. The missing textures were not fabricated. Detailed CRC, hash, URL and dependency evidence is in `lab/reports/native-cave-asset-recovery.json` and `lab/reports/cave-boss-base-dependencies.json`. Private helpers are `lab/tools/recover-cave-assets.py` and `lab/tools/verify-cave-boss-variant.py --base`.

A physical exterior portal remains a separate research gap. Original Lua's `ActivityPortal` is the activities-hub window, not a recovered physical door binding. Preserved Legacy source places a Cobblestone warpstone near the current test spawn; its original Darkthorne positions are elsewhere. Neither ambient cave art nor the unrelated Briarheart/Memorial caves establish the original NPXCavern outside entrance. The current relocated guide and teleport remain explicitly reconstructed pending stronger original evidence.
