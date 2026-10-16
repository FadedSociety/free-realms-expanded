# Harvesting Independent Review

Independent review of MMO piece 002 (Harvesting, TYPE 8) against the shared TYPE 8 engine owned by `mining_restore`, followed by the authorized repair of the two reproduced execution blockers and the mining-hardcoded launch configuration. Reviewed revisions (SHA-256 prefix, 2026-10-09T06:05Z): `src/harvesting.rs` `4e5196ba35ab99c6` (was `cdb451ae…`), `tests/harvesting.rs` `8cd000ed0e6695c9`, `src/mining.rs` `028b839f5dc6e01b` (was `d9871f90…`), `tests/mining.rs` `55d003906b9a80d6`, new `tests/harvesting_review.rs` `a45cc3691f9513df`. Evidence bundle: `../lab/reports/harvesting-review-20261009T054339Z/`.

Verdict: every supported harvesting spec now loads, launches and plays as real code; gap profiles refuse launch explicitly; launch assets are selectable per content and default to the recovered mining names. The emitted playfield is a labelled reconstruction (fidelity gap), and native acceptance still has not happened.

## Repair pass

RED first (`repair-red-harvesting_review.log`, 4 failed): the real engine rejected harvesting 111 with `mining playable area is not rectangular`, every gap profile still proposed a board, the emitted shape had no dropzone row, and the persisted-completion reward path could not even load.

1. **Execution geometry (blocker 1, fixed).** `engine_rectangle_shape(10, 9)` emits a full playable rectangle plus one row of original `aMyGrid` dropzone codes (`4`) inside the 10x10 grid, the smallest shape the shared engine accepts. `diamond_shape` keeps the screenshot-observed staggered board as a recorded artifact only and no longer feeds `board_spec`; every regional `rule_notice` states the fidelity gap explicitly, so the staggered claim is not silently replaced.
2. **Gap profiles (blocker 2, fixed).** 58, 110, 112, 114, 116, 118, 120 and 217 have no recovered produce set, so `board_spec`, `content`, `content_json` and `write_content` now fail with `harvesting game N … produce set is unrecovered, so launching it is unsupported`, and `validate_content` refuses the same ids instead of tolerating empty `selectable`/`collectable`. No `harvesting.json` is written for them.
3. **Launch assets (integration gap, fixed).** The content wrapper carries `preload_file` and `launch_movie`; the engine advertises the content's preload list in `39/16` and requests the content's movie through `26/12`, validating both as plain asset names. Mining keeps its recovered defaults (`MiningAssets.txt`, `game_mining.gfx`) even for older `mining.json` files without the fields; harvesting uses `FarmingAssets.txt` (`Assets_000.pack` entry 50, 97 preload names ending in `game_farming.lst`) and `game_farming.gfx` (`AssetsW_009.pack` entry 1448; `.lst` srcfilename `.../Minigames/MatchThree/Farming/game_farming.swf`).
4. **Rewards (medium finding, fixed).** `validate_spec` refuses any output whose tile value has no matching goal, and `reward_transaction` returns `io::Result<Option<…>>`: an orphaned output is an explicit error instead of silently dropping every grant, and a zero-bundle goal no longer cancels the other goals' grants. Winning still requires the server-side `LevelComplete` → `OnNextLevelMsg` path, with replay monotonic.

## Resolved findings from the first pass

1. Engine load rejection of every regional spec — resolved; all six supported ids load and reach Playing.
2. Gap profiles emitting engine-refused config — resolved by explicit refusal.
3. `MiningAssets.txt` / `game_mining.gfx` leaking into harvesting launches — resolved by selectable launch configuration; mining regressions pin the defaults.
4. Unmatched-output grant loss — resolved by load-time proposal validation plus a fail-closed runtime error.

## Remaining findings and gaps

1. **Low — engine scoring fidelity.** `collect_fall` still scores a flat 500 per collected tile; the original `GameLevel.HandleCollectableTile` doubles it once the goal's quota is reached, and neither module applies the `Progression.PROPERTY_TIMERCHAIN * 0.1` timer term or `LimitToCollectLimit`. Score-only, no reward impact; outside this repair's scope.
2. **Low — schema/grid oracle strictness.** Exact-count caret parsing rejects an extra trailing `^` column the original AS2 would ignore, and `parse_grid_templates` consumes leading-`-` rows and header-only templates where `GameServer.OnGridData` would stop. Deliberate guard strictness; no reward impact.
3. **Evidence boundary.** The farming preload/movie names are recovered from the pinned packs and manifests, but their packet-level use mirrors the mining recovery: no harvesting capture exists yet, so the `39/16`/`26/12` content binding stays reconstruction-by-analogy.
4. **Native gate open.** No native client run was attempted; the shared window stays lead-controlled. Until a native harvesting start is captured, no playable claim is valid.

## Recovered versus reconstructed (explicit)

Recovered and independently re-verified: `MatchThreeSpawnData.txt` 14 columns `SD_ID..SD_TIMER_ID`, `MatchThreeLevelData.txt` 18 columns `LD_ID..LD_TYPEDATA`, `MatchThreeProgression.txt` 6 columns `PD_*`, `MatchThreeTimerData.txt` 6 columns `TD_*` with `SetChain` dividing by ten (`GameServer.as:24-67,330-344`); chain score `400+200*(n-3)`, two-second-per-tile bonus floored at one, `500` collectable score doubled at quota, `CheckCollectableQuota(successOnly=true)` semantics (`GameLevel.as:553-583,615-640,727-749`); tile value `600 + spawn type` (`GameLevel.as:279-287`); region produce sets in locale order (`locale 94149..94154` literally name the four produce per region), generic name/description `31023/35603`, icon `908` for all fourteen rows, bundle locale `23285..23300` mapping to item definitions `2875..2890` (`Type: 1`, `MaxStackSize: -1`; reissues `30574`/`30575`); and the farming launch assets `FarmingAssets.txt`, `game_farming.gfx`, `game_farming.lst`.

Reconstructed, shipped behind `reconstructed_rules: true` + `rule_notice`: the emitted 10x9 playable rectangle plus dropzone row (the screenshot-observed staggered board is recorded by `diamond_shape` but not executable by the shared engine — an explicitly labelled fidelity gap), 180 s, quota 15 tiles, 5 tiles per bundle, spawn percentages 5/12/0/0, one bundle per completed goal (`cluster_quantity` 1), `seed = game_id`, profile id mirroring the game id, the `58_000_001` instance namespace, and the launch-asset packet binding. The original `MatchThree*Data.txt` server tables were not shipped and remain unrecovered; every number in this paragraph is a local choice.

## Checks run

- `cargo test --test harvesting` -> 17 passed, 0 failed (`repair-green-harvesting.log`).
- `cargo test --test harvesting_review` -> 4 passed, 0 failed (`repair-green-harvesting_review.log`): executes all six regional specs through load → begin → `26/12` → handshake → Playing, asserts the farming launch assets, gap refusal, rectangle+dropzone geometry, and a serialized completion state granting `(2885,3)(2887,3)(2890,3)(2877,3)` once.
- `cargo test --test mining` -> 21 passed, 0 failed (`repair-green-mining.log`): the 19 pre-existing regressions plus two new ones (mining launch defaults stay recovered and explicit names are selectable; path-like names and orphan outputs are refused at load).
- `cargo test --test mining_review` -> 13 passed, 0 failed (`repair-green-mining_review.log`), the other worker's target untouched.
- `cargo test --test piece_018` -> 11 passed, 2 ignored, 0 failed (`repair-green-piece_018.log`).
- Independent review harness (`vectors/`, 65 checks, `repair-edge-vectors.log`): the retained 44-vector set plus per-region execution probes. Native `OnChainMsg`/`OnIdleMsg`/`OnVerifyMsg` single-trailing-comma accept with double-comma/inner-empty/space variants rejected with state unchanged, stale instance and wrong game id ignored, client outcome claims inert, quota completion -> `OnStopLevel`, once-only win transaction, monotonic replay, `39/18+39/47+39/19` results once, server-authoritative `tick` loss, profile binding, schema/column round-trips, timer `/10`, `AddCollectableSpawnData` value mapping, 256-vector conservation (`evaluate().total_bundles` equals the engine grant formula and never exceeds the collected floor), deterministic typed config re-serialization, farming launch selection per content, and gap refusal.
- Pinned inputs re-hashed and unchanged: `ClientItemDefinitions.json` `40a3cc79…`, `locale-id-to-text.json` `f1f22a30…`, `minigame-content-inventory.json` `8535110a…` (all match `harvesting-restoration-20261009T020914Z.json`).

## Native acceptance

Not attempted and not inferable. The code path now loads every supported harvesting spec, but no native client run has confirmed a real harvesting launch, chain drag, bin display, results close or reward grant. The shared window stays lead-controlled; open native gates remain the type-8 `39/5` start, `39/14` payloads, `39/17`, `39/19`, restart/replay, and the actual bundle grant.

## Reproduce

```powershell
& scripts/cargo.ps1 test --test harvesting --test harvesting_review --test mining --test mining_review
cd ../lab/reports/harvesting-review-20261009T054339Z/vectors
& ../../../project/scripts/cargo.ps1 run --offline
```
