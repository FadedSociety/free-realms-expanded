# Mining Original Powerups

Read-only recovery of the original Mining (`game_mining.gfx`) powerup tiles, icons,
effects, spawn authoring and wire messages, compared against the current
`prototype/osfr-tools/src/mining.rs`. No source, content, UI, process or save file
was changed for this note.

## Verdict

The original mining movie ships **seven** powerup tiles — three "collected when the
chain removes them" powerups and four clickable ("PressMe") powerups. All seven are
recoverable end to end: value, icon frame label, server effect, client effect and
wire message.

The current engine implements **none** of the seven. It carries a `powerup_percent`
field that `mining_spec()` sets to `0`, never spawns a `900..=953` tile, never emits
`OnPowerupInitMsg`, never registers a powerup icon with `OnTileInfoMsg`, has no
`OnActionMsg` handler and has no `OnLevelEventMsg 40` handler. The `900..=953` arm in
`valid_value` only means such a value would load without complaint; nothing can
produce or act on one.

Two secondary findings matter for the paused native runs:

- The original `ServerGrid.HasMove()` treats *any* runtime powerup tile
  (`value >= 950`) as a legal move on its own. Once a clickable powerup exists on
  the board the server cannot declare "no moves" and cannot reshuffle. The moment
  the last `950..=953` leaves the board the normal reshuffle branch resumes.
- `900..=902` are **not** chain tiles. `ServerGrid.checkCurrentTile` reads the chain
  colour only from `TILE_SELECTABLE` tiles and admits only selectables and
  wildcard `200` (`core/ServerGrid.as` 440-479), so a chain that names a
  `900..=902` tile is refused. They are collected through the fall path instead —
  see "How `900..=902` are actually collected".

## Sources

Decompiled movie scripts (lab, outside Git):
`lab/decompiled/minigames/mining/game-scripts/__Packages/`. Per-file SHA-256 is in
`lab/reports/mining-powerups-20261009T1559Z/evidence-hashes.json`.

The movie itself was re-read with the pinned community reader (JPEXS Free Flash
Decompiler 26.3.0, `lab/tools/ffdec-26.3.0`) rather than trusting the earlier export;
`game_mining.gfx` SHA-256
`b3c07e23eb32d16faa83f2c93f2e7a615be050493590248be9d03243f01b0908`. Tag dump:
`lab/reports/mining-powerups-20261009T1559Z/dumpSWF-PowerUp-region.txt`.

| Claim | Source |
| --- | --- |
| `TILE_POWERUP = 900`, `_FREEZE = 900`, `_SCORE2X = 901`, `_SCORE5X = 902`, `_PRESSME = 950`, `_DYNAMITE = 950`, `_WILDTILE = 951`, `_DESTRUCT = 952`, `_SHUFFLE = 953` | `GameData.as` 70-79 |
| Icon frame labels `PowerUp_Time`, `PowerUp_Score2x`, `PowerUp_Score5x`, `PowerUp_dynamite`, `PowerUp_wildtile`, `PowerUp_destruct`, `PowerUp_shuffle` | `core/TileGenerator.as` 21-27 |
| `_nValue >= TILE_POWERUP_PRESSME` picks `SelectableTile`, otherwise `CollectableTile` | `core/TileGenerator.as` 165-179 |
| Icon atlas frame order and mapped character ids | `dumpSWF.txt` tags 27 (chid 13, frames 3/20/38/56/74/92/109/126) |
| Levels author powerups from spawn rows (`category 3`, type, spawn chance, duration, power, icon id) | `SpawnData.as` 21-25, 26-42; `GameServer.as` 41-54, 208-240, 140-169 |
| Per-level powerup spawn chance defaults to 10 and is set from the level row (`LD_POWERUP_SPAWN = 11`) | `GameLevel.as` 57, 344-350; `GameServer.as` 66, 271 |
| `SendPowerupInit(value, duration, power)` on level start | `GameLevel.as` 120-128; `GameServerNetwork.as` 60-63 |
| `OnPowerupInitMsg(type, duration, power)` stores `aPowerupPower[type] = power` | `GameClient.as` 158-165; `GameClientNetwork.as` 66-68 |
| Powerups are an `oPowerUpRandomizer` pool keyed by `GetValue()`; `SendTileInfo` is **not** called for powerups | `core/ServerTileGenerator.as` 74-85, 103, 239-250 |
| `HasMove()` returns true for any powerup `>= 950` | `core/ServerGrid.as` 189-221 |
| Collection dispatch reaches `HandlePowerupTile` only via `TILE_POWERUP` | `core/ServerGrid.as` 134-137; `GameLevel.as` 584-596 |
| Freeze / Score2x / Score5x effects | `GameLevel.as` 701-725; `GameServerNetwork.as` 44-50, 80-83 |
| Player press flow and client-side effect implementations | `core/Grid.as` 390-419, 1437-1479, 1523-1556, 1571-1612, 1620-1695 |
| `SendAction(index, list)` / `OnActionMsg(index, list)` | `core/Grid.as` 1473, 1554, 1607, 1690; `GameClientNetwork.as` 18-20; `GameLevel.as` 507-542 |
| Any powerup `>= 950` is spawned as a `SelectableTile` and raises a `mousePressed` broadcast | `core/TileGenerator.as` 170-177; `core/SelectableTile.as` 103-106 |
| Freeze pauses the countdown timer for N seconds (`mFrozenTime` accumulates on repeat) | `com/sonyonline/game/SoeTimer.as` 194-212, `library/Timer.as` |
| `isInFallType` defaults to `600 <= value < 949` when `setFallTypes()` was never called, so `900..=902` are fall types | `core/ServerGrid.as` 636-654; `GameData.as` 64, 74; `core/Grid.as` 1212-1224 |
| `setFallTypes()` / `setTransformTypes()` have no call site anywhere in the mining scripts | whole-tree scan of `__Packages` — definitions only (`core/Grid.as` 550, 559; `core/ServerGrid.as` 169, 179) |
| `CLIENT_FALL` runs `FallOffBottom` on the server; the engine already drives collection from that event | `GameLevel.as` 459-469; `core/ServerGrid.as` 1047-1060 |
| Chain membership is `TILE_SELECTABLE` colours plus wildcard `200` only | `core/ServerGrid.as` 440-479, 514-526 |
| `950..=953` are outside the fall default (`>= 949`) and outside every explosion list | `GameData.as` 74; `core/Grid.as` 1502-1521; `core/ServerGrid.as` 960-986 |
| Server `mDuration += seconds * 1000` for a boosted timer | `com/sonyonline/game/SoeLevel.as` 51-62 |
| Server inbound `OnLevelEventMsg` handles only `50, 51, 60, 61` | `GameLevel.as` 459-483; `GameNetworkEvent.as` 16-19 |
| `GameNetworkEvent.POWERUP_FREEZE = 40`, `LEVEL_RESHUFFLE = 11` | `GameNetworkEvent.as` 5, 15 |

## Recovered tiles

| Value | Kind | Icon frame (`sIdName`) | Collection trigger | Server effect |
| --- | --- | --- | --- | --- |
| `900` | collected | `PowerUp_Time` | falls off the bottom (`CLIENT_FALL`) | `ChangeTimerDuration(duration)`, `SendPowerupFreeze(duration)`, `+250` score |
| `901` | collected | `PowerUp_Score2x` | falls off the bottom | `updateMultiplier(2, duration)`, `+250` score |
| `902` | collected | `PowerUp_Score5x` | falls off the bottom | `updateMultiplier(5, duration)`, `+250` score |
| `950` | pressable | `PowerUp_dynamite` | click | `StartShatter`: clear a bounded flood of idle selectables/wildcards/blockers starting at the clicked tile |
| `951` | pressable | `PowerUp_wildtile` | click | `StartWildTiles(power)`: convert `power` idle selectables to `200` wildcards |
| `952` | pressable | `PowerUp_destruct` | click | `StartDestruction(power)`: remove `power` idle selectables (plus blockers) without the dynamite neighbourhood rule |
| `953` | pressable | `PowerUp_shuffle` | click | `StartShuffle(power)`: the pressed tile becomes a wildcard, then positions of up to `power-1` random idle visible tiles rotate |

Icons: the movie's `PowerUp_Combined` atlas (character 13) carries frame labels in
this order — `PowerUp_dynamite` (frame 3), `PowerUp_shuffle` (frame 20),
`PowerUp_Time` (frame 38), `PowerUp_wildtile` (frame 56), `PowerUp_destruct`
(frame 74), then `LootDrop_common/rare/special` (frames 92/109/126). Score2x and
Score5x reuse the `Time` clock frame (character 6) — the atlas has no distinct
multiplier art. `TileGenerator.aNames` still names them separately, so the sent
`sIdName` is what matters, not the art.

## How `900..=902` are actually collected

Recovered end to end, but not by chaining. The path is:

1. Startup treats `900..=948` as fall types. `ServerGrid.isInFallType` returns
   `_nCategory >= GameData.FALLTYPE_START (600)` and
   `_nCategory < GameData.FALLTYPE_END (949)` whenever `aFallTypes` is null
   (`core/ServerGrid.as` 636-654). A whole-tree scan of `__Packages` finds
   `setFallTypes()` / `setTransformTypes()` defined but **never called**
   (`core/Grid.as` 550, 559; `core/ServerGrid.as` 169, 179), so the null default
   is what runs. `900 <= 900..902 < 949`, so all three qualify.
2. `checkFalls` walks each column upward from the dropzone and marks the first
   fall-type tile it meets with `addFallOffBottom()` (`core/ServerGrid.as`
   564-591). The equivalent client code is `core/Grid.as` 1122-1145.
3. The client reports the fall as `OnLevelEventMsg 50 x y` (`CLIENT_FALL`,
   `GameClientNetwork.as` 30-33). This is the C2S message the engine already
   handles, and it is the event that drives collection.
4. `ServerGrid.FallOffBottom` requires the flag, calls `addCollect()` and reaches
   `onTileCollect` (`core/ServerGrid.as` 1047-1060).
5. `onTileCollect` dispatches `HandleCollected(category, value)`
   (`core/ServerGrid.as` 134-137); `HandleCollected` routes `TILE_POWERUP` to
   `HandlePowerupTile` (`GameLevel.as` 584-596).

Excluded collection paths, each verified:

- **Chain:** refused. `checkCurrentTile` reads the chain colour only from
  `TILE_SELECTABLE` and inserts only selectables plus wildcard `200`
  (`core/ServerGrid.as` 440-479); `doMatch3` then re-colours any wildcard to that
  value (514-526). A `900` can never enter `aChain`, so the collect-and-explode
  loop never touches it.
- **Dynamite (`950`) and destruct (`952`):** excluded.
  `ShatterAddTileToArray2` (`core/Grid.as` 1502-1521) and the explosion arrays
  admit only `TILE_SELECTABLE`, `TILE_WILDCARD` and `TILE_BLOCKER`.
- **Wildtile (`951`) / the shuffle wildcard (`953`):** they convert and swap
  selectables and never touch a powerup in place.

Two consequences worth recording:

- The three collected powerups are only obtainable by falling, which is the
  natural outcome of clearing the tiles beneath them.
- `isInFallType` here is a null-default heuristic, not a recovered level table.
  A shipping table that called `setFallTypes(...)`, or a transform rule naming
  them, could narrow it. No such call site exists in the shipped mining scripts,
  so the `600..948` default is the strongest available evidence — but it is an
  inference from absence, not an explicit registration, and is labelled as such.

## Spawn authoring (where real percentages live)

Powerups are not hard-coded in the client movie. Each `SpawnData` row is read as
`id ^ category ^ type ^ spawnchance ^ createmax ^ collectmax ^ quota ^ rewardset ^ iconid ^ duration ^ power ^ requiredperreward ^ collectiontype ^ timerid`
(`GameServer.as` 41-54, 220-235). Category `3` is `CATEGORY_POWERUP`
(`SpawnData.as` 24) and routes to `GameLevel.AddPowerupSpawnData`
(`GameServer.as` 161-163).

`AddPowerupSpawnData` computes `GameData.TILE_POWERUP + data.GetType()` and rejects
any value outside `900,901,902,950,951,952,953` (`GameLevel.as` 429-447). The level
row supplies `LD_POWERUP_SPAWN` (`GameServer.as` 271) into
`nPowerupSpawnChance`, which defaults to `10` and is added to the progression
property `PROPERTY_POWERUP` (`GameLevel.as` 57, 344-350; `Progression.as` 18).

**Conclusion: the live/factory spawn percentages, durations and power values are
server level-table values that were never shipped inside the movie.** They are not
in the decompiled scripts and not in the current Rust content file. Any specific
`%`, duration or power is unrecovered; this note refuses to guess one.

## Wire messages

Server to client:

| Message | Payload | Notes |
| --- | --- | --- |
| `OnPowerupInitMsg` | `type, duration, power` | one per authored powerup, at level start; `type` doubles as the tile value (`900..953`) |
| `OnLevelEventMsg 40` | `duration` | freeze: the client pauses its countdown for `duration` seconds |
| `OnLevelEventMsg 22` | `multiplier, duration` | `Score2x`/`Score5x`; the current engine does not emit this for powerups |
| `OnLevelEventMsg 23` | — | multiplier expired; emitted by the original score timer |

`OnLevelEventMsg 40` is **server to client only**, and must not be accepted
inbound. `SendPowerupFreeze` builds it (`GameServerNetwork.as` 80-83) and the
client's `OnLevelEventMsg` case `POWERUP_FREEZE` calls `oTimer.Freeze(argA)`
(`GameClient.as` 356-358). On the other side the server's inbound
`OnLevelEventMsg` switch handles only `CLIENT_FALL` (50), `CLIENT_CONVERT` (51),
`CLIENT_TILEQUEUE_ACCEPT` (60) and `CLIENT_TILEQUEUE_REJECT` (61)
(`GameLevel.as` 459-483) — a client-sent `40` is not a recognised type and never
reaches a handler. A follow-up worker must emit `40` when a `900` is collected
and must not accept a client-supplied duration, which would let a forged message
extend the level clock.

Client to server:

| Message | Payload | Notes |
| --- | --- | --- |
| `OnActionMsg` | `index, coords...` | only for `950..=953`; `index` is the powerup value, `coords` a flat `x,y,...` list, trailing comma |
| `OnChainMsg` | `coords...` | collects `400..=499` and wildcard `200` only; never `900..=902` |
| `OnLevelEventMsg 50 x y` | coordinates | `CLIENT_FALL`; the client's report that a tile left the bottom, and the collection event for both `600..=699` and `900..=902` |

Fall replacement discipline, needed by the same handler: after a fall the server
draws replacement tiles from `aTileQueue`, which `GenerateTilesForQueue(..., -1, -1)`
fills through the **any** randomizer — so a replacement can itself be a powerup
(`core/ServerGrid.as` 815-843). A rejected queue draw is answered with
`SendTileQueue(true)` (`core/ServerGrid.as` 845-888) and the client replies `60`/`61`
(`GameClientNetwork.as` 38-44).

## Comparison with `src/mining.rs`

Current file SHA-256
`da3b5b567bdcaf054d713297af9bae412a367453c8989dc157030f8946fe7a1d`
(read-only this turn; the engine was not edited).

| Original capability | Current engine |
| --- | --- |
| Seven powerup definitions with duration/power | absent — `Type8BoardSpec` has only a scalar `powerup_percent` |
| `powerup_percent` spawn roll | parsed and validated, but `mining_spec()` ships `0` and `random_tile()` never consults it |
| `OnPowerupInitMsg` at level start | never emitted |
| Powerup icon registration | not needed by the original (icons come from the atlas frame the tile value selects); the engine also does not emit any powerup `OnTileInfoMsg` |
| `OnActionMsg` handler | absent — the match falls through to `unsupported or malformed mining message` |
| `OnLevelEventMsg 40` handler | absent |
| `900..=902` fall-through collection | absent — `collect_fall` requires the bottom-row value to match a goal, so a powerup fall would error rather than fire its effect |
| Chain collection of a powerup | impossible — `validate_chain` requires `400..=499` or `200` |
| `HasMove()` counts a pressable powerup | absent — `has_move` considers only `400..=499` and `200` |
| Dead-board reshuffle | present and independently working (`initialize_board(..., true)` + `OnLevelEventMsg 11`) |

## Minimal faithful capabilities and hard gaps

Minimal faithful set, in the order a follow-up worker should build it:

1. **Content schema.** Replace the single `powerup_percent` number with an authored
   list: `{ value, type, spawn_chance, duration, power }` for the seven legal
   values, plus a per-level `powerup_spawn_chance`. Keep `powerup_percent = 0` while
   the real numbers are unrecovered; the schema and the engine become capable
   without inventing balance. `validate_spec` must reject unknown powerup values,
   duplicate values and out-of-range chances.
2. **Spawn.** Add the powerup branch to `random_tile` and `category_tile(900)`.
   Order matters: the powerup roll must be tried before the wildcard/collectable
   branch, in the shape the current code already uses for wildcard/collectable.
3. **Level-start advertisement.** Emit `OnPowerupInitMsg` per authored powerup,
   so the client knows each pressable's `power`.
4. **Collected-powerup fall handler.** `collect_fall` currently errors when the
   bottom-row value is not a goal. Branch before that check: a `900..=902` fall
   resolves the powerup effect and emits its server message. `900` must **emit**
   `OnLevelEventMsg 40 duration` (bounded by `MAX_REMAINING_MS` on
   `run.remaining_ms`); the handler must **not** accept an inbound `40` from the
   client, because in the original that direction does not exist.
5. **`OnActionMsg index, list`.** Validate `tiles <= power`, coordinates in bounds,
   tiles idle, category rules per power, then apply the effect. The current
   `handle` already splits the payload on tabs into a `Vec`, so a variable-length
   field list parses without refactoring; the work is the tile-order state below.
6. **`has_move` fix.** Return `true` while any board cell holds `950..=953`
   (original `ServerGrid.HasMove()` 203-207). This must not be a permanent
   override: once no pressable remains, the existing no-move reshuffle branch
   must still run.
7. **Do not change `validate_chain` for powerups.** Chaining `900..=902` is not
   original behaviour; the existing `400..=499`/`200` rule is already correct.
   Their `+250` score and value-specific effect ride the fall path from step 4,
   which sits outside the goal/quota flow (`HandlePowerupTile` never calls
   `SendQuotaUpdate`).

Hard implementation gaps (not mechanical):

- **Random-tile order parity.** `StartWildTiles`, `StartDestruction` and
  `StartShuffle` validate the client's coordinate list against tiles drawn from the
  server's random-tile array. The engine must reproduce the original's ordering
  (`InitRandomTileArray` walks the grid, then `GetRandomNumberFromQueue`), or it
  cannot verify the very message it needs to accept.
- **Shatter neighbourhood rule.** `StartShatter` accepts only tiles reachable
  through `ShatterAddTileToArray2` (idle selectables/wildcards/blockers adjacent to
  already-shattered tiles) and caps the count at `power`. That is a different
  algorithm from `StartDestruction` and needs its own function.
- **Multiplier lifetime.** `Score2x`/`Score5x` and `OnLevelEventMsg 22/23` are
  client-timed in the original. If the engine applies a multiplier server-side it
  must also decide who owns the expiry, and the current spec has no per-multiplier
  state. Server-side ownership is the only version that can be tested headlessly.
- **Choice of engine vs. level table.** The real percentages, durations and powers
  are unrecovered. Shipping a non-zero `powerup_percent` without them would be
  invented balance, which this task forbids.

## Testable contract for the follow-up worker

Deterministic, headless, and independent of the paused native run:

1. Content: a spec with `powerup_percent = 0` and no powerup entries still loads and
   plays exactly as today (regression guard).
2. Content: a spec with a powerup entry whose `value` is not in
   `{900,901,902,950,951,952,953}`, or has a duplicate `value`, is refused by
   `validate_spec` before any writer runs.
3. Spawn: with `powerup_percent = 100` and one authored powerup, the first 70 queue
   tiles are all that powerup value; with `0`, none are.
4. Level start: the board build replies contain exactly one `OnPowerupInitMsg` per
   authored powerup, in authored order, with `value, duration, power`.
5. Freeze: a `900` falling off the bottom **emits** `OnLevelEventMsg 40 <d>` and
   increases `remaining_ms` by `d*1000` (clamped at `MAX_REMAINING_MS`). An
   inbound client `OnLevelEventMsg 40 <d>` is refused as unsupported and leaves
   `run` unchanged.
6. Move detection: a board with no colour match but one `950` tile reports a move
   (no reshuffle); the same board with no pressable falls through to the existing
   reshuffle path, proving the rule is not a permanent override.
7. `OnActionMsg` negative cases: unknown index, more coordinates than the authored
   `power`, out-of-bounds coordinates, non-idle tile, and a coordinate whose value
   does not match the server's expectation are each refused without mutating
   `run`.
8. `OnActionMsg 951` positive case: `n` idle selectables become `200` in place and
   the reply `OnActionMsg` echoes the pressed index and the same coordinate list.
9. `OnActionMsg 952` positive case: exactly the listed selectables are cleared and
   gravity runs; `953` rotates positions and leaves the tile count unchanged;
   `950` clears only the neighbourhood-reachable set.
10. Powerups are not chain tiles: a chain naming `900`, `901` or `902` is still
    refused, exactly as today. The `901` fall instead emits
    `OnLevelEventMsg 22 2 <duration>` and adds `250` score; the `900` fall emits
    `OnLevelEventMsg 40 <duration>`.

## Unresolved / not established

- Whether any reconstructable Mining18 board ever *needed* a powerup. The current
  engine already escapes dead boards with the reshuffle path, so this research does
  not claim the paused native run is blocked on powerups; that run's blocker was
  the reviewed window/process state outside this task.
- Whether the engine should apply multiplier expiry server-side or mirror the
  client's timer.
- Whether a shipping level table ever called `setFallTypes`/`setTransformTypes`
  with a narrower list. No call site exists anywhere in the recovered mining
  scripts, so the null default (`600..=948`) is used above; a level-table call
  could in principle exclude `900..=902`, and this note cannot rule that out from
  the movie alone.
- LootDrop tones (`1000..=1002`) share the atlas and the `StartLootDrop` hook but
  were out of scope for this note beyond naming the frames.

## Private evidence

`lab/reports/mining-powerups-20261009T1559Z/` — `dumpSWF.txt` (full tag dump,
SHA-256 `94d588fd330fb85d316e6a4bdb8921d2e8772f9628d33ad3cfc9aea27d467a18`),
`dumpSWF-PowerUp-region.txt` (character 13 region), `correction-proof.txt`
(line-excerpt proof for the fall-type collection path, the chain rejection and
the `40` direction), `evidence-hashes.json` (per-script and per-tool SHA-256),
`receipt.json`.
