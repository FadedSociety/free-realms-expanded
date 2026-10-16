# Cave Results Score Binding

Read-only diagnosis of the native cave win panel (run `client-baseline-20261009T114509Z`, `replay-win-score_small.png`: YOU WIN / Opponents Defeated 4 / Goals 1/1 5000 / **TOTAL SCORE 0**). No source, test, shared, state, UI, release or process file was changed; the active native run was only read.

## Verdict

The panel is bound correctly; one server field carries the wrong value. TOTAL SCORE is the **fifth i32 (Points) of the `39/47` row whose Order/Score-Type field is 4** (`scoreTotalScore`). `encounter.rs::end_scores` writes that field as a literal `0` for every run (`for value in [-1, score_type, count, -1, 0]`), so the native panel shows 0 even though the same run sends a 5000-point completed objective in `45/3`.

Native evidence from this run (read-only, decoded with `decode-cave-score-row.ps1`):

| Frame | Packet | Decoded |
| --- | --- | --- |
| `application-1369.bin` | `45/2` | objective `9100411` count `4` |
| `application-1370.bin` | `45/3` | objective `9100411`, extra `0`, value **5000** |
| `application-1374.bin` | `39/47 [411,-1,411]` | rows=2: `scoreEnemiesDefeated` `[-1,0,4,-1,0]`, `scoreTotalScore` `[-1,4,-1,-1,0]` |
| `application-1095.bin` | `39/47 [411,-1,411]` | same layout with Value=`3` (the loss) and total Points `0` |

So the win frame's total row is byte-identical to the compiled pinned writer with `Points = 0`, and the only field that changes for a non-zero total is that final i32.

## Independent oracle proof

1. **Original Lua binding.** `lab/decompiled/client-ui/ScriptsBase.lua:48393` calls `self.GetSwf():Invoke("setTotalScore", self:GetTotalScore())`, and `GetTotalScore` (`:48401-48409`) walks the `DS_MINI_GAME_SCORE` datasource returning `ds:GetData(i,5)` for the row whose `ds:GetData(i,2) == 4`. Column 5 is the row's Points field.
2. **Original score screen.** `lab/decompiled/client-ui/score-screen/scripts/__Packages/scorescreen/states/ScoreState.as::update()` maps a row with `Score Type == 4` to the `ScoreTotal` item and renders it from that item's `"Score Points"`; other types render as ordinary rows (type 3 = the goal separator) and are sorted by Score Type.
3. **Pinned community writer.** `lab/references/Sanctuary-combat-pr-research/pr-76/src/Sanctuary.Packet/BaseMiniGamePacket/MiniGameLootWheelPackets.cs`: `MiniGameScoreRow { Name, NameId, Order, Value, Max, Points }` and `MiniGameGameEndScorePacket` writes `[i32 count]` then per row `[i32 len][ascii name][i32 NameId][i32 Order][i32 Value][i32 Max][i32 Points]`; its comment notes the client appends its own `scoreObjectives`/`scoreBonusObjectives` rows from the MiniGameState.
4. **Compiled writer expectation (private probe).** `writer-probe/` builds the unmodified pinned writer with the private dotnet and serializes `Value = 4`, `Points = 0` and `Points = 5000`:
   - native `application-1374` inner == compiled writer `Points = 0` **byte for byte** (`wire-vs-writer.log`), so the layout and current value are confirmed;
   - compiled writer `Points = 5000` differs from the native frame **only in the final i32** (`88130000` = 5000 little-endian);
   - `writer-probe.log`: `total_points=0` sha256 `89e06c1417d49a493037c9a425207888ed40c522556be68bd1b0401caa0ea770`, `total_points=5000` sha256 `609920c009a82f4c4f8e0a8cdca129dc3bb1154211373c621d5bee2e9c2281e4`.
5. **Native handler shape.** The preserved disassembly of the `39/47` handler `009bb030` (quoted in `lab/reports/native-combat-results-trace.json` and `native-crafting-results-lifecycle-proof.json`) copies the server rows into the MiniGameState and then appends `scoreObjectives` and `scoreBonusObjectives` rows locally, which is why the client shows a `Goals 1/1` row that the server never sends.

## Original versus reconstructed score policy

- **Original.** The game server computed the run's score: the total is a server field (Points of the Order-4 row). The client never derives it. The original encounter-411 score table and points are **not recovered**; this repository's encounter combat rules, waves and rewards are documented reconstructions.
- **Current reconstruction.** `encounter.rs::end_scores` deliberately reports zero points ("the current combat rules award no score, so report real defeats without inventing points"), while `objective_complete` sends a local `5000` objective value. The result is the panel's internal contradiction: a 5000-point completed goal and TOTAL SCORE 0.
- **Where the 5000 appears.** The same run sends `45/3 [9100411, 0, 5000]` and the client's locally appended objective row renders `5000`; the value matches our packet one-to-one, and no 5000 constant exists in the score-screen AS2 or the Lua path. Inference limit: the linkage is value-identical and consistent with the native local-row handler, but no debugger trace proves the client copies that exact field.

## Exact suspected fields (no source change made)

1. **`encounter.rs::end_scores`, `scoreTotalScore` row, 5th i32** — hardcoded `0`; this is what must carry the run total.
2. **`encounter.rs::objective_complete` 4th i32** — local `5000` objective value; either it should be summed into (1), or the scoreless policy must drive it to 0 so the panel is self-consistent.
3. **`scoreEnemiesDefeated` row, 5th i32** — also hardcoded `0`; if the accepted policy keeps no kill points, this is consistent and should stay labelled reconstructed.

Minimal options for the lead, both with no schema or protocol-layout change:

- **A (consistent non-zero total):** compute the run total from the score sources the run actually awarded (the completed objective's value, plus any accepted kill points) and write it into the Order-4 row's Points. The compiled writer needs no change; the wire differs only in that i32.
- **B (strict zero-score):** keep the total at 0 but stop sending the 5000 objective value so the client's goal row also reads 0. This keeps "no invented points" while removing the contradiction.

The compiled pinned writer expectation for either choice is already proven above; no layout investigation remains.

## Artifacts and hashes

Evidence bundle `../lab/reports/cave-score-binding-20261009T121500Z/`: `decode-cave-score-row.ps1`, `native-wire-decode.log` (`34e2fcfbe60be3f7…`), `writer-probe/` (unmodified pinned writer sources), `writer-probe.log` (`7ca18ed69dad1ff2…`), `wire-vs-writer.log` (`5198f3b34c3887f9…`).

Run frames: `analysis/application-1369.bin` `3067d598e94cb57d…`, `application-1370.bin` `f909a5faf887b744…`, `application-1374.bin` `569ae65e6267c170…`, `application-1095.bin` `8dc51340fc8da3b5…`.

Prior recorded native evidence: [[Native Combat Results v3]] already noted "the win movie also renders its local Goals 1/1 row with 5000; total remains 0"; this investigation pins the exact field and the compiled-writer expectation behind that display.

## Implemented repair (2026-10-09)

Lead policy applied: the existing local objective award is the won total, a loss stays zero, and the score policy remains explicitly reconstructed. No kill points, historical score, reward, save field or XP/coin path was added.

- `src/encounter.rs` `9c0123fe…`-era change (`68cfc114b65ae9f8…` after formatting): one constant `OBJECTIVE_AWARD_SCORE: i32 = 5000` now feeds both `objective_complete` and `end_scores`. In the Order/Score-Type 4 row only the fifth i32 (Points) changes: won → `OBJECTIVE_AWARD_SCORE`, loss → `0`. The `scoreEnemiesDefeated` row keeps Points `0` and its actual defeat count, and every other field in the packet is untouched.
- New regression file `prototype/osfr-tools/tests/encounter_score.rs` (`9c0123feebda5efb`): RED first — the win `39/47` ended `00000000` against the compiled-writer golden `88130000` (`lab/reports/cave-score-coherence-20261009T124500Z/red-encounter_score.log`, 2 passed / 1 failed) — then GREEN 3/3 (`green-encounter_score.log`). The win packet is now byte-identical to the independently compiled pinned writer with `Value = 4`, `Points = 5000`; the loss test keeps three real defeats and total 0; the third test proves results present once, the `39/7` ACK and a wrong-instance close do not exit, and the bound-instance close does.
- Versioned oracle bundle `lab/reports/native-results-vectors-v4/` (generated by `lab/reports/cave-score-binding-20261009T121500Z/writer-probe/` from the unmodified pinned Sanctuary PR76 writers): identical to `native-results-vectors-v3` except the won end-scores row, whose last i32 is now `88130000`; win sha256 `58675e1da6af145b…`, loss `8e69efeb6e47f842…`. The original zero-total receipt `native-results-vectors-v3` is **preserved unmodified** and re-verified against its own manifest (`vector-delta.log`).
- The existing ignored private oracle test passes with the versioned bundle: `OSFR_NATIVE_RESULTS_VECTORS=<v4> cargo test --test encounter -- --ignored native_results_close` → 1 passed (`green-encounter-oracle-v4.log`). `tests/encounter.rs` needed no edit because that test resolves vectors through the environment variable.
- `rustfmt --edition 2024 --check` on `src/encounter.rs` and `tests/encounter_score.rs` exits 0.

Re-run history: a mid-way re-run hit a transient compile break while the concurrent `gameplay.rs` mining integration was half-applied (`E0063 missing mining`, then `E0599/E0277 apply_mining`; `blocked-encounter-oracle-v3-cargo.log`). Once that edit landed the full set re-ran green on the shared tree: `encounter` 15 passed (+2 ignored), `encounter_score` 3 passed, `gateway` 7 passed (+1 ignored), `opening_cave_replay` 5 passed (`green-final-gates.log`), and the ignored private v4 oracle test 1 passed (`green-encounter-oracle-v4.log`). Private proof: `lab/reports/cave-score-coherence-20261009T124500Z/score-coherence-proof.json`.
