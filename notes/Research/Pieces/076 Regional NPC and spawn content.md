# 076 Regional NPC and spawn content

Status: recovered the regional actor placement catalogue for world zone 1 and its npc/model
identifiers from two pinned community revisions of the same project, plus a checker that validates
placements, zone membership and asset resolution. Implemented in Rust (module + acceptance test) and
Python (extractor/checker + unittest suite) with RED-first evidence. No native client run and no
native acceptance is claimed.

Queue item: `notes/WorkItems/076.md` (acceptance: source-backed actor placements/zone membership;
cross-zone identifier and asset validation). Worker: crack_builder, DeepSeek V4.1 Flash,
2026-10-09 UTC.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_076.rs` | Recovered zone record, nine area definitions, guid classification, asset binding, placement audit and cross-revision model check |
| `prototype/osfr-tools/tests/piece_076.rs` | 9 focused tests; `#[path]`-includes the module until the lead registers it in `src/restoration/` |
| `scripts/pieces/piece_076.py` | Read-only checker over the pinned sources; emits `placement-catalogue.json` + `validation-report.json` |
| `tests/pieces/test_piece_076.py` | 16 stdlib unittest cases (real-source counts + rejection paths) |
| `../lab/reports/piece-076-20261009T043748Z/` | Private evidence: closed-form measurements, the checker run, RED/GREEN cargo logs and the full-suite run |

## Pinned sources

| Source | Revision / hash |
| --- | --- |
| `lab/references/Sanctuary/src/Scripts/Zone/FabledRealms.lua` | Sanctuary `4f028dcc274f628b65ef665fd59b184f5f3d3e3e` (AGPL-3.0), 305075 bytes, sha256 `1fecb07813fb0770b7c44a401fadd2be87e202614a775bb11ed04ff970d73b18` |
| `lab/references/Sanctuary/src/Resources/Npcs.json` | same revision, 436324 bytes, sha256 `b51d2d92da1bded648f347337e1beebf399fcd409eb77d1165e33ceb19c0b38f` |
| `lab/references/Sanctuary/src/Resources/Zones/FabledRealms.json` | same revision, 1785 bytes, sha256 `a049fcfe89b40c331e9c604d613fe8a017c3a6cfe9fad1679b286b6c1ed473d1` |
| `lab/references/FreeRealms-Legacy/src/Scripts/Zone/FabledRealms.lua` | `raisingkaines/FreeRealms-Legacy` `9e082025391177d5a33ee18c714ddd06b457a695` (AGPL-3.0), 380317 bytes, sha256 `bc0673f8d1fc7ed4e20ae2d501d2197e9bcc8133a20edff962f60fe36b106e74` |
| `lab/references/FreeRealms-Legacy/src/Resources/Npcs.json` | same revision, 519102 bytes, sha256 `f8f224482a022fbb6c7d96280bf977ed0a648135ba965b51fb854bb75d69cff2` |
| `lab/extracted/client-assets/catalog.tsv` | extracted client catalogue, 11218192 bytes, sha256 `12bfd0f81236ffc02e6a9a661e1ffb14890a2bd4ac2306c14b20e9cbe445b1c9` |
| `lab/references/freerealms-wiki/wiki.sqlite` | read-only wiki corpus, 16171008 bytes, sha256 `b9d7b9f1adabb42db54c3253bf43fddc249d6d70e3f2075b134a9441e2fcd911` |

All sources are read-only, outside Git, and hashed by every checker run.

## Recovered facts

- **Placements.** The Sanctuary revision contains exactly 3426
  `zone:spawnNpcWithGuid(npcId, guid, x, y, z, heading)` calls; the checker parses all 3426 and
  refuses a script where any call does not match the recovered shape. All 3426 npc ids are distinct.
- **Zone record.** `FabledRealms.json` is a `World` zone: id 1, name `FabledRealms`, tile size 64,
  latitude -48..80, longitude -32..96, spawn position `[-1904.883, -39.7098, 412.6024]`.
- **Named runtime areas.** Nine `AreaDefinitions`: Blackspore, Briarwood, Merryvale, Sanctuary,
  Seaside, Shrouded Glades, Snowhill, Wugachug, Sunstone Valley (four rectangles, five circles).
- **Guid convention.** 3421 placements use `100000000000 + npc id`; four use the npc row's
  `CaptureGuid` (33167 `14293651161170`, 33558 `23983097380882`, 33602 `7215545057314`,
  36143 `13125420056642`); one (33081) uses `100000033190`, which is neither.
- **Npc rows.** The pinned `Npcs.json` carries 3426 rows covering every spawned npc id (0 missing).
  Exactly four rows are interaction actors bound to existing terrain objects (`ModelId` 0 plus a
  `TerrainObjectId`): 36143 Sunstone Valley Warpstone, 36307 Stillwater Crossing Warpstone,
  36308 Blackspore Warpstone, 36309 Rumble Ridge Warpstone.
- **Asset resolution.** The placements reference 793 distinct model files. 737 resolve in the
  extracted client catalogue; 56 do not (`dwarf_m_therin.adr`, `chugawug_m_nucklehead.adr`,
  `cat_toygerghost.adr`, ...). These are gaps in the preserved client set, not invented failures.
- **Second revision.** The `FreeRealms-Legacy` revision has 4253 placements and 4253 npc rows; 3419
  npc ids are shared with the Sanctuary revision and **no shared id has a different model**.

## Inferred and reconstructed

- **Inferred (region membership).** No per-placement region field is recovered, so membership is
  derived from the nine recovered shapes: Seaside 442, Briarwood 385, Blackspore 238,
  Sunstone Valley 235, Sanctuary 167, Merryvale 147, Wugachug 140, Shrouded Glades 114,
  Snowhill 103. 1455 placements lie inside the world zone but outside those nine areas; the
  checker reports that as a classification gap, not as a defect or as an invented tenth region.
- **Reconstructed (validation policy).** Contradictions break the catalogue: unknown npc row,
  duplicate guid, a row with neither a model nor a terrain binding, a guid that aliases a different
  recovered npc id, or a shared npc id whose model differs between the two pinned revisions.
  Findings are recovered deviations that are reported and reviewed without invalidating the
  placement: npc 33081's non-conventional guid aliases no other npc row.
- **Wiki cross-check.** All nine recovered area names resolve as exact wiki titles; two need a
  documented spelling alias (`Merryvale` -> `Merry Vale`, `Shrouded Glades` -> `Shrouded Glade`),
  so a new region name cannot silently pass.

## Commands and results

RED (before the implementation existed):

```text
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_076.py" -v
FileNotFoundError: piece 076 checker is missing: ...\scripts\pieces\piece_076.py
FAILED (errors=1)

scripts/cargo.ps1 test --test piece_076
error: couldn't find file `tests\..\src\restoration\piece_076.rs`
error: could not compile `osfr-tools` (test "piece_076") due to 1 previous error
```

GREEN:

```text
scripts/cargo.ps1 test --test piece_076
running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_076.py" -v
Ran 16 tests in 2.059s
OK

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts/pieces/piece_076.py --output ..\lab\reports\piece-076-20261009T043748Z\checker
{"status": "pass", "placements": 3426, "conventional_guids": 3421, "capture_guids": 4,
 "unexpected_guids": [33081], "unresolved_model_assets": 56,
 "cross_revision": {"legacy_placements": 4253, "shared_npc_ids": 3419, "model_conflicts": 0},
 "contradictions": [], "gaps": 99}
```

Shared-workspace context: `scripts/cargo.ps1 test` (the whole crate) currently stops on another
worker's in-flight `tests/piece_051.rs`, whose `src/restoration/piece_051.rs` does not exist yet.
Running every other test target explicitly (130 targets, logs in the report directory) gave
`1298 passed; 0 failed; 57 ignored`.

## Boundary and remaining gates

- Recovered original/community facts: placement rows, npc rows, area shapes, model file names,
  guid convention and the second-revision coverage. Community *recovered*, not first-party bytes.
- Reconstructed: the region membership inference and the contradiction/finding policy. Numeric
  positions and identifiers are copied, never invented.
- Not established: whether Sanctuary's placement recovery is historically complete for the world
  zone, which of the 56 unresolved model files exist in the public CDN set, and any native spawn,
  motion or interaction behaviour.
- Integration/native gates for the lead: register `pub mod piece_076;` and use the catalogue as the
  regional actor source for gateway spawns (items 051/052/053/091), then confirm actor presence,
  facing and interaction in the client. Nothing in this piece spawns an actor or grants a reward.
