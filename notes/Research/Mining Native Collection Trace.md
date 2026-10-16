# Mining Native Collection Trace (read-only, live run 151307)

Live run `lab/runs/client-baseline-20261009T151307Z` (root owns UI/save/release);
state branch `lab/state/mining-harvest-native-20261009T150229Z`; frozen engine
`da3b5b567bdcaf054d713297af9bae412a367453c8989dc157030f8946fe7a1d`. This note
and `lab\reports\mining-native-collection-trace-20261009T154431Z/` are the only writes; no source/content/driver/save/
UI/process edit. Trace captured at 2026-10-09T15:48:48Z;
the run is live, so later states will differ.

## Verdict (bounded)

The first ore credit is real. At **constant chain count** the engine credited
`+500` score and `collected 0→1` while the ore that had stood on the bottom row
`(col 8,row 10)=601` disappeared. That is the `OnLevelEventMsg 50 →
collect_fall` transaction; the chain ACK path never collects. Ore advance to the
bottom comes from gravity/refill plus the client's own fall animation.

Conservation is **narrowed**: gravity/refill moves and replenishes tiles, and
the observed first-credit transitions show no unexplained ore loss, but ore is
**not** only removed by `collect_fall`. `LevelReshuffle` deliberately destroys
every tile including collectables (original `core/ServerGrid.as` 321-335; our
`initialize_board(..., reshuffle=true)` at `src/mining.rs` 616-624, 1070-1081),
and cancelling/closing the run drops the active run entirely
(`Type8Engine::cancel` 404-405). Those destructive paths are outside the
conservation statement below.

Client authorship is a **source-path + accepted-state inference**, not a raw
frame observation: the relay capture stores client→server opcode-9 payloads as
ciphertext (`frames-scan.txt`), so the literal `50` message is not decoded
there. The decompiled movie authors the fall, and the engine only credits
through that path.

## 1. Exact transition (server save lines 146/148/150/152)

| line | chains | score | collected | wall_ms | board ores (col,row,val) |
| --- | --- | --- | --- | --- | --- |
| 146 | 9 | 6600 | [0,0] | 117041 | 10 ores, highest row 7 |
| 148 | 10 | 7000 | [0,0] | 118602 | ... 10 ores incl. **(8,10,601)** |
| 150 | 10 | 7500 | **[1,0]** | 119117 | 9 ores, (8,10) gone, all others identical |
| 152 | 10 | 7500 | [1,0] | 119893 | 9 ores, unchanged |

Chain count is identical across 148→150, so the +500/+1 is not chain score
(`CHAIN_SCORE`/`CHAIN_TILE_SCORE` are only added in the `OnChainMsg` arm).
`COLLECTABLE_SCORE = 500` and `collected+1` exist only in `collect_fall`
(`src/mining.rs` 1249-1311), reachable only from `OnLevelEventMsg 50`
(658-668). Full save table: `server-save-trace.txt`; transition detail:
`collection-transition.txt`.

## 2. Authoring and indexing (original movie vs route 39)

- Source path: the client authors every fall — `Tile.fall` broadcasts
  `fallOffBottom` once a running fall passes the get-off-grid frame with
  `bHasToFallOffBottom` (`Tile.as` 339-344); spawned tiles flag at appear
  (288-292); `Grid.fallOffBottom` calls `SendFall(x,y)` (`Grid.as` 502) with
  `getPosition()` = (`nX`, `nFutureY`) (Tile.as 138-143) →
  `OnLevelEventMsg 50` x y (`GameClientNetwork.as` 30-33) →
  `GameLevel.OnLevelEventMsg` 463-469 → `ServerGrid.FallOffBottom(x,y)`
  (1047-1060, requires the checkFalls dropzone flag from 567-591).
- Accepted-state inference: the only engine path that produces `+500` with
  `collected+1` at unchanged chains is `collect_fall`, so the client did send an
  accepted `50` for the ore that stood at (col 8,row 10).
- Route 39 matches that indexing: `["OnLevelEventMsg","50",x,y]` →
  `collect_fall(x,y)` requires `y == bottom playable row` and a goal collectable.
  The chain arm (574-626) sends chain score/time-bonus/gravity/rebuild only.
  Anchors: `route39-anchors.txt`.

## 3. Ore advance, gravity/refill and the destructive paths

- Save trace across chains 0→13+: board ores grow 0→1→2→5→7→8→10 while queue
  ores shrink 13→12→11→9→7→6 (`gravity`, `src/mining.rs` 1198-1247 pulls queue
  tiles in); the credited ore had advanced to row 10.
- `random_tile(..., true)` refills (1231-1243) can seed **new** collectables
  (queue ores 6→7 at chain 13), so totals are not a fixed global constant.
- Destructive paths NOT covered by conservation: `LevelReshuffle` erases all
  tiles including ore (original `ServerGrid.as` 321-335; `initialize_board`
  reshuffle 616-624 / 1070-1081), and run cancel/close drops the run
  (`cancel` 404-405; native close/cancel likewise).
- Correction note: an earlier revision of this note said ore was "removed only
  by `collect_fall`" / "no silent ore loss". That was over-broad and is corrected
  here; `conservation-reshuffle-correction.txt` records the anchors.

## Hypotheses

- H1 (confirmed, bounded to the captured transitions): collection requires a
  client `CLIENT_FALL`; the chain ACK alone never credits. Evidence: 148→150.
- H2 (confirmed, bounded): ore advance to the bottom comes from gravity/refill
  plus the client's fall animation; the gateway never authors falls.
- H3 (confirmed, bounded): refill seeding can add ores; totals are not fixed.
- H4 (corrected): `LevelReshuffle` (and run cancel/close) destroy ore outside
  the gravity/refill conservation window.
- Pending-fall race: **bounded** — `notes/Research/Mining Pending Fall
  Reshuffle.md` disproves it for the captured chain #19 of run 134930 and shows
  no pending ore in that trigger; a crafted spec-valid counterexample still
  shows a reshuffle can fire while a fall is animating in the original's
  ordering. This trace does not exclude pending-fall/reshuffle races in general.

## Limits

- Client→server opcode 9 payloads are ciphertext in the relay capture
  (`frames-scan.txt`), so the literal `50` message is not readable from frames;
  authorship is inferred from the decompiled source path plus the accepted
  state transition.
- The run is live; values after the captured wall_ms are not claimed.
- No universal claim about every ore path: gravity/refill, reshuffle and
  cancel/close are three distinct destruction/creation routes.
