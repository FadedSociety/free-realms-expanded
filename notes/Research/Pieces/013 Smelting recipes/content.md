# 013 Smelting recipes/content

Status: furnace definitions, materials and movie payload surface recovered and
verified 2026-10-09 03:00 UTC from the preserved lab corpus. One new
conformance validator, one extractor/verifier and one private evidence bundle.
No native client run was made by this worker; nothing here is a playability
claim.

## What this piece delivers

| Path | What |
| --- | --- |
| `scripts/pieces/piece_013.py` | Reads the private AS2/client inputs read-only, re-derives the recovered movie facts, verifies source digests, the 278-entry preload manifest, the prepared recipe/materials and the once-only shape, then writes a report |
| `tests/pieces/test_piece_013.py` | 16 stdlib tests: AS2 table/parser fixtures, payload-surface extraction, per-cell classification, preload CRC checks, once-only rules, real-corpus re-run |
| `prototype/osfr-tools/src/restoration/piece_013.rs` | Recovered constants (5 ores, furnace, heat furnace, payload surface, GameWon literal) plus `classify_phases`/`check_content`/`check_payload_surface`; not yet registered in `lib.rs` |
| `prototype/osfr-tools/tests/piece_013.rs` | 6 owned tests (5 ordinary + 1 opt-in private content check) |
| `../lab/reports/piece-013-20261009T030014Z/` | Evidence bundle: `report.json` (24,231 B) and `summary.md` (1,921 B) |

## Recovered original facts

`SmeltingGameServer.OnLevelData` reads 19 caret columns and names them
`LD_ID`, `LD_GAME`, `LD_SEQUENCE`, `LD_DURATION`, `LD_OBJECTIVE_ID`,
`LD_STARTID`, `LD_ENDID`, `LD_STAGE_TYPE`, `LD_DATA1`..`LD_DATA11`. The
per-stage meaning of the data columns is recovered from
`OnLevelFinishLevelData` and the round constructors:

| Stage | data1 | data2 | data3 | data4 | data5 | data6 | data7/8 | data9/10/11 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 101 breakDownOre | ore id 1-5 | performance = min(100, v*20) | A clicks | B clicks | hit radius increment (0-300) | strength-meter speed (\|v\| 10-100) | hits per power smash (clip 1-5) | score per power smash (floor 50 / 2x chain / cap 10000) |
| 102 fillFurnace | measure target percent (1-100) | measure precision | perfect precision | good score (clip 100-50000) | perfect score (>= good, cap 50000) | - | - | - |
| 103 heatFurnace | heat goal | heat-loss multiplier (* 0.05) | - | - | - | - | - | - |
| 104 moldMetal | spill (integer row) | score per action (clip 500-50000) | - | - | - | - | - | - |

The movie's own tables in `Smelting/SmeltingData.as` are original data:

| Ore | id | A/B/C clicks | measure | ml/ore | powder range | liquid thickness/range |
| --- | --- | --- | --- | --- | --- | --- |
| Copper | 1 | 1/1/1 | 0.65 | 50 | 5 | 0.3 / 4 |
| Tin | 2 | 1/1/1 | 0.70 | 50 | 5 | 0.25 / 4 |
| Iron | 3 | 2/1/1 | 0.75 | 50 | 4 | 0.2 / 3 |
| Silver | 4 | 2/2/1 | 0.80 | 50 | 3 | 0.16 / 2 |
| Gold | 5 | 3/2/1 | 0.85 | 50 | 3 | 0.13 / 2 |

Furnace scalars: door close 0.3, required metal 0.7, mix 150, hammer sizes
100/130, measure precision 8, perfect precision 3, spill 2.5, maximum five
ores, stage ids 1-4. `SpecialRounds/HeatFurnace/Data.as` adds heat gain max 30,
loss rate 0.05 and the blower angles 65/90/115.

Payload surface (recovered message names only, no code): the movie sends
`FRServer_Connect`, `FRServer_ReadyForGameData`, `FRServer_StatUpdate`,
`FRServer_Pause`, `FRServer_RoundTransitionPause`, `OnStartGameMsg`,
`FRServer_CompleteLevel`, `FRServer_StopLevel`, `FRServer_GameWon`,
`FRServer_ScoreSet`, `FRServer_ScoreIncrement`, `FRServer_GameLost`,
`FRServer_GameQuit`, `FRServer_GameOver`, `FRServer_GameClose`; it registers
the reply handlers `OnGameDataMsg`, `OnPhaseDataMsg`, `OnCustomDataMsg`,
`OnDataComplete`, `OnGameWonMsg`, `OnGameLostMsg`, `OnServerAddScoreMsg`,
`OnSkipStageOrLevelMsg`.

`SmeltingGameClient.as` `onWin` sends `SendGameWon(this.nTotalScore,10)`: the
reported quantity is a literal 10, not the produced amount. The produced
ingots come from the mold round, `nQuantity = int(performance / 20)`, and are
reported separately as `STAT_GAME_QUANTITY` (kind 8). A server must therefore
not derive the reward from the client quantity field.

The movie preload manifest `SmeltingAssets.txt` lists 278 unique `img*.dds`
names; all 278 resolve in the preserved pack catalogue and every CRC32 matches
the extracted bytes (278/278).

## Original vs reconstruction

The prepared `../lab/references/rust-opening/smelting.json` is classified cell
by cell in the report. Cells equal to recovered movie constants: ore id,
A/B clicks, hit radius increment, measure target (65), precision pair (8/3).
Reconstructed cells: durations, performance multiplier, strength-meter speed,
hits per power smash, scores, heat goal/loss and mold score/spill. The row
schema cannot express the movie's 2.5 spill default, so the prepared spill 2 is
an integer reconstruction. The recipe itself (Copper Ore 5 + Singing Coal 3 ->
Copper Bar 10) stays community/wiki-sourced (`Miner?oldid=46068`) plus the
movie's literal 10; no 2014 server recipe table was recovered.

Material definitions 2739 (Copper Ore), 879 (Singing Coal) and 21785 (Copper
Bar) exist in the live inventory. Finding: the stored 2739 record keeps an
earlier local serialization and differs from the pinned Legacy catalogue
record at two bytes (offsets 80 and 130); 879 and 21785 match byte for byte.
This is recorded, not silently corrected. Singing Coal has no movie-side
definition at all: the movie knows five metal ores only.

## Verification

RED was real: the Python test module first failed with
`FileNotFoundError: ... scripts\pieces\piece_013.py`, and the Rust test target
first failed with `error: couldn't find file tests\..\src\restoration\piece_013.rs`.

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_013.py" -v
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_013.py
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_013')
& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','piece_013','private_prepared_content_is_conformant','--','--ignored','--exact')
```

Observed: Python 16/16 OK; CLI `verdict: pass`, `verified_preload: 278`,
report written; Rust 5 passed / 0 failed / 1 ignored; the opt-in private check
1 passed. Scoped `rustfmt --edition 2024 --check` on the two owned Rust files
exits 0.

## Provenance

Every input is digest-pinned in `scripts/pieces/piece_013.py` and re-verified on
each run, including `SmeltingData.as`
`663bd85f9cbcdc15ccec1a253d8a77fdca98771b56ea516396d77f6b070a8b59`,
`SmeltingGameServer.as`
`88b40b430cfdc056ee3ed1ab14672453578da7d1dfc5f39b464bc8ca23b6d197`,
`SmeltingGameClient.as`
`49b759ae033eb6dcd000f786c7d60da1eaa6dd09841aac6e2005dead2c362ed5`,
`GameExternalNetwork.as`
`5f76bf92a1effede953568ebf47fdbb948f3bcc5f11f19179fe87a9b94b5cd88`,
`MiniGameData.txt`
`8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252`,
`pack_08/003825.txt`
`847fa8bcfcf32ca4bbc54365d667f1a7818a2f755bc54547180842304e7756e1` and the
pinned `ClientItemDefinitions.json`
`40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`.
Tool: JPEXS 26.3.0 AS2 export of the preserved `game_smelting.gfx`; the movie
and all raw/decompiled assets stay in the lab.

## Skills applied

`reverse-engineering` (reading the AS2 as the spec for the phase-row schema,
clamps and payload literals), `game-research-websearch` (community catalogue
and wiki provenance for the recipe, kept separate from recovered facts),
`ponytail` (one extractor, one validator, no new dependencies, reuse of the
existing prepared-content format), `caveman` (concise reporting). RTK: normal
commands were run; no blind prefixing.

## Remaining gates

- Lead integration: `piece_013.rs` is not registered in shared `lib.rs`; the
  lead decides whether `check_content` gates content loading.
- Native acceptance: furnace entry, the four mouse stages, the normal score
  screen, material consumption/output and quest turn-in are untouched by this
  piece; see [[Original Smelting]] and [[Native Smelting Controls]].
- Missing original evidence: the 2014 server phase table, authoritative
  durations/heat/score tuning and the original recipe table. Tin/Iron/Silver/
  Gold smelting recipes remain unmapped; only the movie-side metal definitions
  were recovered.
