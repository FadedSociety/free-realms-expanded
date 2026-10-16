# Native Mining Dead Board (run 125556) - read-only diagnosis

Scope: reading only. No source, UI, save, process, or content edits. Investigation owned by
this task: this note plus `lab/reports/native-mining-dead-board-20261009T131834Z/`.

Inputs
- Run: `lab/runs/client-baseline-20261009T125556Z` (Miner14 native, release gateway sha256 `7d5be63fd3cb1814e15211c07423ddf472d63f0f236410d7790c6b5b5d475f39`; root states this
  build carries the frozen old gameplay/lib `077`/`376`).
- Source read-only: `project/prototype/osfr-tools/src/mining.rs` sha256 `e6e421d8e7cd0e923b9f033638f24fb04d648613e9b6719e78a28c3fc45e090e` (the e6e4 revision).
- Original scripts: `lab/decompiled/minigames/mining/game-scripts/__Packages` (anchors inline below).

## 1. Observed native state (decisive)

`character-after-batch1-settled.json` sha256 `cd5aee73837d80e5b620ccef76635d0290d5c7347c793bdadddf79c64879ba1d` and
`character-after-batch1-stop.json` sha256 `7dbe00e231abe55bc68e30433b0a719886149c6330cf9c45575be55161d0870a` (identical board):

- `phase=Playing paused=True chains=6 score=3000 collected=[0,0] reshuffles=0 remaining_ms=172329 rng=3309872643 end_sent=False`
- `idle` true 196/196 - no tile is mid-accept/mid-fall; the client had no animation in flight (root
  already verified a 3 s unpause changes nothing, so this is not pause/animation lag).
- row 10 (bottom playable row): `3:403 4:402 5:405 6:404 7:405 8:602 9:405 10:601`
- row 11: `100` (TILE_DROPZONE) at columns 3..10 - both bottom ores sit directly above dropzone cells.
- 11 live ores remain; root's dry run of the exact board under `has_move` (mining.rs 1394-1441) reports
  no legal chain. `reshuffles=0`: the post-chain reshuffle never fired.
- Full extract: `retained-state-probe.txt` (this bundle).

## 2. Original fall wiring - why a static bottom ore never sends CLIENT_FALL

Chain settle on the client: `core/Grid.as` 1054-1061 `clearChain; UpdateIdle; SendChain;
updateCurrentGridArray; checkFalls; fallTiles; updateCurrentGridArray; VerifyGridWithServer`.

- `core/Grid.checkFalls` dropzone sweep (`Grid.as` 1119-1146; server mirror `core/ServerGrid.as`
  567-591): walking up from each `TILE_DROPZONE` tile, the first non-empty cell is tested; a
  fall-type value (`GameData.FALLTYPE_START` 600 .. `FALLTYPE_END` 949, `GameData.as` 64/74, so 601/602
  qualify) gets `addFallOffBottom()` and the walk stops. This sets a flag only.
- `core/Tile.addFallOffBottom` (`Tile.as` 207-210) sets `bHasToFallOffBottom = true`. It does not add
  fall rows and does not move the tile.
- The flag is consumed in exactly two places:
  - `core/Tile.fall` (`Tile.as` 303-351): inside a running fall animation, once
    `nTime >= GameData.nTIME_TO_GET_OFF_GRID` (7), line 339-344 clears the flag, calls
    `onFallFinished(false)` and broadcasts `fallOffBottom`.
  - `core/Tile.appear` (`Tile.as` 288-292): when a spawn (APPEAR) finishes with the flag set.
- `core/Grid.fallOffBottom` (`Grid.as` 492-508) handles that broadcast: sound, state COLLECTED, then
  `this.mNetwork.SendFall(x,y)` (line 502) -> `GameClientNetwork.SendFall` (`GameClientNetwork.as`
  30-33) -> `OnLevelEventMsg 50` -> `GameLevel.OnLevelEventMsg` (`GameLevel.as` 463-469) ->
  `ServerGrid.FallOffBottom` (`ServerGrid.as` 1047-1060), which collects only when the server tile's
  `bFallOffBottom` is set (set by the same sweep, 567-591, or by `onFallFinished` 142-155).
- Movement gate: `core/Tile.doFall` (`Tile.as` 219-237) animates only when `nTilesToFall > 0`.
  `nTilesToFall` is raised only by `Tile.addFall` (`Tile.as` 193-202), called from
  `Grid.addColumnFall` (`Grid.as` 1231-1245) for tiles above a real hole; `addColumnFall` skips
  `TILE_EMPTY` and `TILE_DROPZONE` cells (line 1237), so the dropzone row never creates fall rows.

Consequence: a collectable that is already idle on the bottom playable row above the dropzone has
`nTilesToFall == 0` and never re-enters APPEAR. `doFall` does nothing, `fall()` never runs, the
`fallOffBottom` broadcast never fires, and no `CLIENT_FALL` is sent - even though both the client and
the server sweep have flagged it. The sweep alone is inert without a running fall or spawn.

## 3. Rust counterpart (mining.rs `e6e421d8e7cd`)

- `gravity` (1198-1247) moves tiles only into holes inside the playable bounds and refills from the
  tile queue; it has no path that moves a tile into the dropzone row and emits nothing per moved tile.
- The only collect path is inbound `OnLevelEventMsg 50` (658-671) -> `collect_fall` (1249-1311),
  which refuses anything not on the bottom playable row (1259) or not a goal collectable
  (1263-1267). The native run never failed the session, so no accepted client 50 exists for these
  two tiles; `collected` stayed `[0,0]`.
- Reshuffle gates carry a suppressor the original does not have:
  616-617 (post-chain) and 1307 (post-collect) are
  `!has_move(...) && !has_bottom_collectable(...)`.
  `has_bottom_collectable` (1443-1449) is true whenever any goal collectable sits anywhere in the
  bottom playable row - exactly `(10,8)=602`, `(10,10)=601` in the retained state.
- Original: `ServerGrid.updateCurrentGridArray` (`ServerGrid.as` 725-758) is
  `if(!HasMove()) LevelReshuffle()` with no collectable suppressor; `HasMove` (189-220) is the
  ordinary match check (`GetCategory(401..405) == TILE_SELECTABLE 400`, matching-neighbour count > 1,
  `GetCategory` 391-394); `LevelReshuffle` (321-335) destroys all tiles and rebuilds the board.

Precise cause: after the 6th chain the board is genuinely chainless, so the original would
`LevelReshuffle`. Rust instead suppresses that reshuffle because a collectable is parked in the
bottom row, and its only drain for such a tile (client `OnLevelEventMsg 50`) requires a client fall
animation that the client never produces for a static, idle bottom tile. The two facts combine into a
permanent deadlock: `reshuffles=0`, `collected=[0,0]`, all tiles idle.

## 4. Frame capture check (bounded)

`manifest.json` lists 13857 relay frames (6303 client-to-server, 7554 server-to-client). ASCII scan:
`OnLevelEventMsg` appears only in 13 server-to-client frames (plaintext protocol text). The
client-to-server game frames (opcode 9) are ciphertext in the capture (sample head
`0009 0000 aabbb843 f12cca67 ...`), so the literal absence of a client `50` cannot be proven from
frames; the settled gateway state above proves no accepted collect happened. Gateways never logged a
client message failure, and `collect_fall` fails closed on a mismatched event, so an accepted-but-
ignored event is not possible.

## 5. Bounded test-first repair proposal (not applied - no edits in this task)

1. RED test first, in the existing mining test target: build from the exact retained board
   (14x14; playable 3..10 x 3..10; dropzone 100 at row 11 cols 3..10; values from
   `retained-state-probe.txt`; `chains=6 score=3000 collected=[0,0]`), feed one accepted ordinary
   chain that leaves `has_move == false` with both bottom ores present, and assert the reshuffle path
   runs (`run.reshuffles` increments and the client-visible rebuild messages are emitted). On the
   current source this is RED: `reshuffles` stays 0.
2. Minimal fix: delete the `&& !has_bottom_collectable(...)` clauses at 616-617 and 1307 (restore the
   original `if(!HasMove()) LevelReshuffle()` parity) and remove the now-unused helper 1443-1449.
   Green: same board reshuffles; the stuck ores are replaced by the original rebuild semantics.
3. Regressions: reshuffle still only when `has_move` is false; a board with a legal move and a bottom
   ore still does not reshuffle; any existing test that asserted the bottom-collectable suppressor
   becomes an original-parity guard.
4. Rejected alternatives: server-side auto-collect of bottom ores (invents economy/animation timing
   the original keeps client-authoritative); lowering quotas or forcing a fresh board (masks the
   deadlock; the board is a legitimate chainless board under the original rules).

Open (not needed for the fix): whether an original *falling* ore could overshoot onto the dropzone in
a column whose bottom cell was a hole at sweep time; the sweep has no undefined-cell guard, which is
worth a separate bounded read if the lead wants the full history.

## Evidence limits

- Client-to-server opcode-9 payloads are ciphertext in the relay capture (above), so the exact
  per-message trace of the six chains is unavailable from frames.
- AS2 `trace(...)` output is not in the capture; anchors above are from the decompiled scripts only.


## 6. Repair receipt - reshuffle parity (applied mining-dead-board-repair-20261009T133240Z)

Authorized bounded test-first repair. Owned paths only: `src/mining.rs`,
`tests/mining_dead_board.rs`, this note, private report bundle.

- Pre-change source preserved: `e6e421d8e7cd0e923b9f033638f24fb04d648613e9b6719e78a28c3fc45e090e  bytes=58636  lab\reports\mining-dead-board-repair-20261009T133240Z\mining.rs.e6e4.bak`.
- Patch (3 hunks, 2 insertions / 12 deletions): deleted the two
  `&& !has_bottom_collectable(...)` clauses (chain path and collect path) and the
  now-unused helper. Diff: `3e8adc1482f4d2151d803df635193e6c45f9fff396a8b8ba34e3df2c7d0f4ab7  bytes=2119  lab\reports\mining-dead-board-repair-20261009T133240Z\mining-reshuffle.diff`.
- Post-change source: `ec611ec094fb86054651a14eafde15244fc3e19657dc7d0928514ee695b0adec  bytes=58183  project\prototype\osfr-tools\src\mining.rs`.
- New regression: `f2d71f4eb36d9acf568c62580882d629ccea9a087532783160cac3e1d8226bc3  bytes=10112  project\prototype\osfr-tools\tests\mining_dead_board.rs`.
- RED before the fix (`red-mining_dead_board.log`): 1 passed / 2 failed, both at
  `run.reshuffles == 1` (left 0) with `exit=101`; guard test green.
- GREEN after the fix (`green-mining_dead_board.log`): 3 passed / 0 failed, `exit=0`.
- Focused suites after the fix (`green-summary.txt`): mining 21/21, mining_review
  13/13, mining_tile_rendering 3/3, type8_gateway 10/10, harvesting 17/17, all
  `exit=0`.
- Unrelated concurrent failures, not reverted, evidence only:
  `type8_harvesting_gateway` 10/11 (`harvesting reward definition 2885 is not in
  the loaded inventory`, tests/type8_harvesting_gateway.rs:237 - Harvesting
  inventory/content binding, while `src/inventory.rs` and `src/gameplay.rs`
  changed at 06:36:02 during this run) and `harvesting_review`, whose test binary
  was blocked by an Application Control policy (`os error 4551`) on both attempts.
- Economy/quota/seed/packet effect: only the original rebuild (`OnLevelEventMsg
  11`, map/queue rebuild) now runs where the original `if(!HasMove())` would;
  no quota, seed, reward, spawn-percentage or slot changes.


## 7. Repair receipt - shape 1 regular pool (applied mining-shape-parity-repair-20261009T133907Z)

Second authorized patch, sequential after the reshuffle receipt above. Owned
paths: `src/mining.rs`, `tests/mining_shape_parity.rs`, this note, private
report bundle `lab\reports\mining-shape-parity-repair-20261009T133907Z`. Root's
`notes/Research/Native Mining Shape Parity.md` was read only and left untouched
(sha256 `22ddec0f5b3f53443c050a277e4f3db6e6ff984847974be4940ec4eab772a7f5`).

- Pre-change source preserved: `ec611ec094fb86054651a14eafde15244fc3e19657dc7d0928514ee695b0adec  bytes=58183  lab\reports\mining-shape-parity-repair-20261009T133907Z\mining.rs.post-reshuffle.bak`.
- Patch (1 arm + comment): shape `1` (`TILE_BUILDREGULAR`) now draws
  `random_tile(..., false)`; shape `2` and the refill queues keep `true`.
  Diff: `47138762e48e4727b3164bff5ddd783a9321c1492ca2bc2d513362fc738cbe99  bytes=1121  lab\reports\mining-shape-parity-repair-20261009T133907Z\mining-shape-parity.diff`. Anchors: ServerGrid.as 357-361,
  ServerTileGenerator.as 107-110, 126-137 (regular pool = selectable/hazard/
  wildcard, no collectable).
- Post-change source: `6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed  bytes=58345  project\prototype\osfr-tools\src\mining.rs`.
- New regression: `90c9b8eb85ab69695f11570fb9bb9f20968258533cb3941a31c9f1886d6110f5  bytes=10498  project\prototype\osfr-tools\tests\mining_shape_parity.rs` - forced 100% collectable chance on shape 1
  (RED before the arm change), shape 2 acceptance, short-queue gravity refill,
  and mixed seeds 18..21 at 50%.
- RED (`red-mining_shape_parity.log`): 2 passed / 2 failed, exit=101.
  GREEN (`green-mining_shape_parity.log`): 4 passed / 0 failed, exit=0.
- Focused regressions (`green-summary.txt`): mining 21, mining_review 13,
  mining_dead_board 3, type8_gateway 10, mining_tile_rendering 3, harvesting 17,
  all exit=0 (71 focused tests).
- Scope: no quota, seed, spawn-percentage, output, save or reward changes.
