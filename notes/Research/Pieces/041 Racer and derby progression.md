# 041 Racer and derby progression

Queue item 041. Acceptance: recover job/rank/vehicle unlock definitions; link verified game
outcomes to progression.

Status 2026-10-09 05:31 UTC: the two driving jobs' identity, the rank model, the rank-gated gear
and kart-part definitions, the 70-row vehicle catalogue and the six vehicles the supplied capture
proves the character owned are recovered. A bounded progression contract credits one validated
race/derby outcome exactly once and reports the unlock rows a rank opens. The star/XP threshold
table and any payout numbers stay explicit gaps. **No native or gameplay acceptance was
attempted.**

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_041.rs` | Recovered job/rank/unlock constants, `DriverProgression` credit contract |
| `prototype/osfr-tools/tests/piece_041.rs` | 11 tests; includes the module by `#[path]` until the lead registers it |
| `scripts/pieces/piece_041.py` | Read-only checker over the capture, item catalogue, item classes, locale, packet enum and client Lua |
| `tests/pieces/test_piece_041.py` | 13 stdlib tests: hermetic tamper cases, parsers and real-corpus pins |
| `../lab/reports/piece-041-20261009T0402Z/` | `racer-derby-progression.json`, `provenance.json`, RED/GREEN logs, checker output |

## Recovered original facts

1. **Jobs.** The captured self packet carries the `ClientPcProfiles` rows "Start of Kart Driver"
   (`JobGUID` 48, `JobName` 2917, `JobDescription` 2918, `JobLevel` 20, badge 7494) and "Start of
   Demo Derby Driver" (49, 2919, 2920, `JobLevel` 20, badge 7494). Locale ids 2917/2919 resolve to
   "Kart Driver" / "Demo Derby Driver"; the community `Profiles.json` agrees on id, name id and
   description id. Cross-link: [[031 Job catalogue]] owns the generic job table; this note only
   carries the definitions the two driving jobs need.
2. **Activity binding.** Lobby game type 1 = Kart Race, 2 = Demo Derby. The racing capture rides
   tunnel inner opcode 33 (`BaseVehicleRacePacket`) and the derby capture 34
   (`BaseCehicleDemolitionDerbyPacket`); the loadout and part families are 137
   (`BaseVehicleLoadoutPacket`) and 139 (`BaseVehiclePartPacket`).
3. **Rank model.** The client Lua caps the displayed job rank at 20 (`if tonumber(rank) == 20 then
   exp = 100 end`), the captured driving profiles sit at `JobLevel` 20, and both gear catalogues
   gate items across ranks 1..20. The wiki (revision 43870) documents 20 as the maximum and 4 for
   non-members; only the 20 is reproduced by the captured client, so the 4 stays community policy.
4. **Gear unlock definitions.** `ItemClasses.txt` row 24 is "Kart Driver Gear" (profile name id
   2917) and row 22 "Demolition Derby Driver Gear" (profile name id 17401). The catalogue holds
   1,842 Kart Driver gear rows over ranks 1..20 and 1,647 derby gear rows over ranks 1..8 and
   10..20: **no rank 9 derby gear row exists**. Representative rows (lowest item id per rank, with
   the name id and the full row count) are carried in the module and re-derived by the checker.
5. **Kart parts.** `ItemClasses.txt` rows 75 ("Racing Kart", part item `Type` 9) and 76 ("Demo
   Derby Kart", `Type` 10) hold 131 part rows each: categories Engine (129), Turbo (130), Steering
   (131), Traction (132), Mod A (133), Mod B (134), Decals (136) and Chassis (139). Their rank
   gates are only 1 (128 rows) and 5 (three members-only decals each).
6. **Vehicles.** Item `Type` 11 with `CategoryId` 127 is the 70-row vehicle catalogue
   (Racer/Speedster/Wrecker/Crusher/Zipper/Runner families, `PowerRating` 1-5, `Cost` 540 on the
   priced rows). The captured `VehicleStruct` carries six loadouts; every loadout's item id, name
   id, description id and model index (`Param1`) matches a catalogue row exactly - Roadster Racer
   48316/6, Ripper Wrecker 48317/9, Splash Zipper 61006/51, Arrow Runner 61000/45, Barbed Crusher
   56995/40, Cardie Racer 48318/7. The loadouts also carry the equipped part-slot guid list, which
   is recovered data but not yet mapped to part definitions.
7. **Outcome link.** `MiniGameStarsEarnedPacket` is inner opcode 21 and its payload is a single
   byte; the client's minigame datasource exposes `MiniGame_DS.EarnedStars` and the star counter
   increments from it. The job rank itself is server-driven: the client dispatches a `JobLevelUp`
   call handler and reads `Profile_DS` columns Rank (4), StarsAvailable (5) and RankPercent (12).

## Reconstructed policy

`DriverProgression` (local, not historical) is a per-job ledger:

* `credit(outcome, profile_rank)` credits one validated outcome at most once, keyed by the outcome
  id from the owning session contract (`kart_racing`/`derby` granted reward path). Replays return
  `Duplicate` and move nothing.
* A derby outcome credited against the Kart Driver ledger (or the reverse) returns `WrongJob` and
  changes nothing; the activity-to-job binding is the recovered game type, not a caller claim.
* An unpriced finish (`stars_earned` 0, the racing payout case) is recorded and adds zero; the
  module never invents an award.
* At profile rank 20 the outcome is recorded once and returns `AtMaxRank` without advancing.
* `unlocked(rank)` / `unlocks_at(rank)` report the recovered gear rows; the rank itself is supplied
  by the caller because the threshold table was never recovered.
* The ledger moves no coins, items or vehicle ownership. The lead's economy layer decides grants.

## Not recovered (explicit gaps)

* The star/XP threshold table that turns an award total into the next job rank. It was server-side;
  no captured payload, client table or community source in the preserved corpus contains it.
* Race and derby payout amounts, and therefore the reward value of a win.
* Whether a vehicle row is a store purchase, a quest reward or a rank grant: `Cost` 540 is present
  on the priced rows and `MinProfileRank` is 1 for all 70, so the catalogue alone does not prove
  purchase semantics. The part-slot guid lists in the loadouts are not yet joined to part rows.
* Native behavior for anything above: entering a race/derby, the rank-up call handler, and the
  vehicle picker still need the original client against the lead's gateway.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test piece_041` with a skeleton module | RED: 2 compile errors, 26 unresolved contract items named |
| `scripts/cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test piece_041` | GREEN: 11 passed, 0 failed, no warnings |
| `scripts/cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --lib --test piece_041` | GREEN: lib suite plus 11 target tests |
| `rustfmt --edition 2024 --check src/restoration/piece_041.rs tests/piece_041.rs` | exit 0, empty output (my two files; the crate-wide `cargo fmt --check` still reports other workers' in-flight files) |
| `python scripts/pieces/piece_041.py --out <lab report>\checker` | 40 checks pass, 0 problems |
| `python -m unittest discover -s tests/pieces -p "test_piece_041.py" -v` | GREEN: 13 passed |
| whole-workspace `cargo test` | not run; other workers hold in-flight targets, and the piece target plus the lib suite cover this change |

## Skills applied

| Skill | Concrete benefit here |
| --- | --- |
| RTK | Commands run normally; only the decisive test and checker lines were read. |
| Ponytail | Constants plus one ledger struct; no traits, no curve table, no new dependency; a single `ponytail:` ceiling comment marks the deliberate stop. |
| Caveman | Terse evidence-first reporting; ids, ranks and hashes quoted exactly. |
| game-recon | Identified the data sources (captured self packet, item catalogue, item classes) instead of guessing at vehicle progression. |
| game-research-websearch | Used the pinned wiki corpus revisions and the community packet enum/locale exports with provenance rather than live scraping. |
| reverse-engineering | Read the Ghidra client export for the star packet, `JobLevelUp` and the profile datasource columns. |
| asset-pipeline | Not applicable: this slice consumes definitions, it does not convert sprites or models. |

## Reproduce

```powershell
$lab = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab'
$report = "$lab\reports\piece-041-20261009T0402Z"
& "$lab\tools\wiki-env\Scripts\python.exe" "$report\analyze-piece-041.py"
cd '%USERPROFILE%\Documents\Projects\CartoonMMO\project'
& "$lab\tools\wiki-env\Scripts\python.exe" scripts\pieces\piece_041.py --out "$report\checker"
& "$lab\tools\wiki-env\Scripts\python.exe" -m unittest discover -s tests\pieces -p "test_piece_041.py" -v
.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test piece_041
```

Cross-links: [[007 Demolition derby]], [[Kart Racing Restoration]], [[015 Minigame launch catalogue]], [[Gameplay Coverage]], [[../Design/Rust Revival Roadmap]].
