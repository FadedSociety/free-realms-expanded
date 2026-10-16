# Mining Clickable Powerups Implementation

Implemented the four recovered clickable Mining powerups (`950..=953`) behind a
private, caller-authored content surface, with the collected `900..=902` timers
and score multipliers explicitly deferred. Test-first: the RED fixtures in
`prototype/osfr-tools/tests/mining_powerups.rs` were written against the recovered
behaviour before the engine gained the handlers, and each review correction below
kept a matched regression.

## Review corrections applied

1. **No server-to-client `OnActionMsg`.** `GameClientNetwork.SendAction` (18-20) is
   the only sender, `GameServerNetwork.OnActionMsg` (118-120) is a receiver stub and
   `GameLevel.OnActionMsg` (507-542) validates and mutates without ever replying;
   `GameClient` registers no `OnActionMsg` handler. The earlier echo was a phantom
   message and is gone; the server answers an action only through the tile queue
   and gravity events it already sends.
2. **Shatter neighbour order is down, up, right, left.** `ShatterAddNeighbors`
   inserts `_loc5_` (down), `_loc4_` (up), `_loc3_` (right), `_loc6_` (left) in
   both `ServerGrid` 956-966 and `Grid` 1491-1501, and the port matches.
3. **Shuffle never recycles the pressed tile.** `StartShuffle` runs
   `doWildTiles(PRESSED)`, shifts the pressed coordinate out of the client list and
   decrements the power, so the pressed cell keeps its `200` wildcard and the
   rotation covers only the remaining drawn tiles.
4. **Random queue: consume, then refill while short.** `GetRandomNumberFromQueue`
   pops first and `GenerateRandomNumbersForQueue` appends `nRANDOM_QUEUE_BATCHSIZE`
   per short-queue pass (795-808), so a restored short queue refills repeatedly
   and each batch is advertised in order.
5. **Implied counts come from the authored power or the pool cap**, never from the
   client list length: short forged lists are refused for all four powerups, while
   a genuinely short pool accepts the shorter implied count.

## What shipped

| Piece | Where |
| --- | --- |
| Recovered tile constants (`TILE_POWERUP_START`, `_PRESSME`, `_DYNAMITE`, `_WILDTILE`, `_DESTRUCT`, `_SHUFFLE`) | `src/mining.rs` |
| Private authoring row `Type8PowerupDefinition { value, spawn_chance, duration, power }` on `Content`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]` | `src/mining.rs` |
| `validate_powerup_definitions` — refuses the deferred `900..=949`, unknown values, duplicates, zero/oversized power, out-of-range tuning, and `powerup_percent > 0` with no weighted definition | `src/mining.rs` |
| Weighted spawn pick plus the powerup branch in `random_tile` | `src/mining.rs` |
| `OnPowerupInitMsg value, duration, power` per authored row at level start, ascending value | `initialize_board` |
| `ServerGrid.HasMove()` parity: a board holding `950..=953` always has a move | `has_move` |
| `OnActionMsg index, list` for `950..=953` with per-power bounds, in-board/idle/uniqueness checks and atomic errors | `apply_powerup_action` |
| Server-owned selection: action targets drawn from the recovered random-number queue and matched against the client list in order | `action_draw`, `init_random_tiles`, `get_random_tile` |
| All four effects — shatter (`950`), wildtile (`951`), destruct (`952`), shuffle (`953`) | `shatter`, `wild_tiles`, `destruction`, `shuffle_tiles` |
| Direct-port placement helper `shuffle_placements` (1228-1233 same-value walk) | `src/mining.rs` |
| 20 behavioral tests | `tests/mining_powerups.rs` |

Recovered references: `core/ServerGrid.as` `StartWildTiles` (988-1046),
`StartDestruction` (1102-1165), `StartShuffle` (1166-1249), `HasMove` (189-221);
`GameLevel.OnActionMsg` (507-542, receiver only); `GameClientNetwork.SendAction` (18-20, the only sender);
`core/TileGenerator.generate_powerup` (165-179).

## Default-path invariance

The authoring surface is additive and cannot change existing behaviour:

- A content file without `powerup_definitions` deserializes exactly as before, and
  `write_content` never emits the key, so the shipped `mining.json` bytes are
  unchanged.
- `powerup_percent == 0` takes the original `random_tile` branch and draws the RNG
  the same number of times in the same order, so board, tile queue, random queue,
  replies and saved `Type8State` are identical.
- `Type8BoardSpec`, `Type8Run` and `Type8State` are untouched, so the public board
  API and the persisted schema do not change.
- Serving, reward and quota paths are untouched: nothing in the action handler
  reads the client's score, quota or timer.

`default_powerup_percent_matches_the_old_engine_byte_for_byte` proves the first two
by running the same fixture with the key absent and with an empty list, and
comparing replies and the whole `Type8State`; `default_writer_bytes_carry_no_powerup_key`
proves the writer bytes.

## Faithful parts and deliberate simplifications

Faithful to the recovered originals:

- Spawn: a weighted pick over the authored rows, then the level's powerup roll,
  replacing the selectable branch of the regular roll.
- Advertise: one `OnPowerupInitMsg` per authored row.
- `HasMove`: any `950..=953` on the board is a legal move, so the reshuffle fires
  only when no pressable remains.
- Action envelope: `OnActionMsg index, list` where `index` is the powerup value and
  `list` is the recovered trailing-comma `x,y,...` form; the reply echoes
  `index` and the same coordinates.
- **Selection ownership: the server draws, the client list must match.** Every
  action pops its targets from the recovered random-number queue
  (`ServerGrid.GetRandomNumberFromQueue` 783-793, refilled by
  `GenerateRandomNumbersForQueue` 795-808) and requires the client's list to name
  exactly those tiles, in order. A mismatched list is refused atomically, so a
  forged or stale action changes nothing.
- Wildtile: one draw per named tile after the pressed tile; the pressed tile is
  popped out of the pool first (`StartWildTiles` 1013-1024); each confirmed tile
  becomes `200`.
- Destruct: the same draw-and-verify contract over idle selectables plus blockers
  (`StartDestruction` 1110-1155); each confirmed tile is cleared.
- Shuffle: the pressed tile stays the `200` at its own cell. The tile drawn at
  index `i` is parked for slot `(i + 1) % n` (`_loc13_[(i + 1) % n] = _loc4_`,
  1214), so the mover that lands on slot `i` is the tile drawn at
  `(i + n - 1) % n`. The destination walk (1222-1245) reads the *mutable*
  position list while reading values from the *static* grid, advances while the
  destination already carries the mover's value, folds the skipped destination
  back onto the current one (1240) and reverts when the walk runs past the
  rotated count (1234-1237). `[401, 402, 403] -> [403, 401, 402]`; with equal
  neighbours `[401, 401, 402] -> [402, 401, 401]`.
- Dynamite: the recovered two-level shatter BFS (`StartShatter` 889-955,
  `ShatterValidateNextTile` 950-955, `ShatterAddNeighbors` 956-987) — the pressed
  tile first, then one draw per named tile from the down/up/right/left neighbour
  pool, where every confirmed tile adds its four idle selectable/wildcard/blocker
  neighbours before the next draw.
- After an action the engine runs `gravity` and then `has_move`, exactly like the
  original `updateCurrentGridArray`.

Deliberate simplifications, with their ceilings:

- **The queue's rolls are the engine's own.** This engine seeds
  `run.random_queue` from its own `run.rng`, so the concrete picks differ from a
  historical server's roll, but the ownership, draw count and
  draw-and-verify contract match the recovered code. The queue refill is
  re-advertised with `OnRandomQueueMsg`, mirroring `SendRandomNumberQueue`.
- **No historical balance.** `spawn_chance`, `duration` and `power` are
  caller-authored tuning; the real values lived in the unrecovered server tables.
  `mining_spec()` still ships `powerup_percent = 0` and no definitions, so the
  default game is unchanged.
- The action list is bounded by the authored `power` (and 64 by validation), so no
  client list can drive an unbounded loop; a blow-up in shatter neighbours is
  bounded by the playable rectangle.

## Deferred, by design

- `900..=902` collected powerups: `validate_powerup_definitions` refuses any
  definition in that range with an explicit "not implemented" error, so no inert
  or partially-implemented spawn can ship. Timer extension (`40`) and score
  multipliers (`22`/`23`) stay unimplemented.
- A `900..=902` tile that still reached the bottom (a hand-edited save) is now
  accepted and removed instead of failing the fall handler; the effect is a no-op
  until the timer work lands.
- The action draw queue is refilled from `run.rng`, not from the historical
  `Math.random` seed, so picks are internally reproducible but not historically
  identical. That is the one remaining fidelity gap and it is labelled above.

## Validation

- `tests/mining_powerups.rs`: 20 passed, 0 failed.
- Mining family: `mining` 21, `mining_dead_board` 3, `mining_move_detection` 3
  (1 ignored private snapshot), `mining_review` 13, `mining_shape_parity` 4,
  `mining_tile_rendering` 3, `mining_powerups` 20 — all green.
- Gateway suites: `type8_gateway` 10, `type8_harvesting_gateway` 11 — green.
- `scripts/cargo.ps1 check --lib` clean for this crate's library and mining
  surface, with no warnings.
- **Earlier attempt on an older test revision, not a current blocker:** an
  `--all-targets` sweep run during this work stopped on the root-owned
  `tests/kart_progress_native.rs` (`client_row_finished` unresolved). That log
  predates root's later fixes; root has since verified by `rg` that the current
  file contains only `results_row_finished` / `results_screen_trigger`, with no
  `client_row_finished` anywhere. No `--all-targets` result is claimed for the
  pinned revision, and the frozen source and test file in this task are
  unaffected either way.

No release build, no native run, no content regeneration, no save or UI action was
performed. `src/gameplay.rs`, `src/harvesting.rs`, `src/lib.rs` and every other
file were left to their owners.

## Private evidence

`lab/reports/mining-powerups-impl-20261009T1700Z/` — `hashes.txt`,
`full-suite.log`; the frozen pre-edit source is
`lab/reports/mining-powerups-implementation-baseline-20261009T1607Z/mining-da3b.rs`
(SHA-256 `da3b5b567bdcaf054d713297af9bae412a367453c8989dc157030f8946fe7a1d`).

Final corrected pins and focused-run logs live in
`lab/reports/mining-powerups-impl-final-20261009T1830Z/` (`receipt.json`,
`hashes.txt`, `focused-summary.txt`, per-target logs).

### Shuffle placement repair (2026-10-09)

Root and an independent reviewer produced a runtime RED oracle
(`lab/reports/mining-clickable-review-final-20261009T170936Z/probe/tests/placement_queue.rs`,
reviewer-owned and not modified here) proving the previous shuffle produced
`[402, 401, 403]` where the recovered script yields `[403, 401, 402]`. The cause
and the direct-port fix are recorded in
`lab/reports/mining-clickable-repair-20261009T1900Z/repair-receipt.json`; the
pre-repair source and test copies are preserved there as `mining-038a.rs` and
`mining_powerups-ae2a.rs`.

Repaired pins:
`src/mining.rs` `4abe6b07f8edf52798ce810cde752a58fb959c30fcb639675371a041e463eb46`
and `tests/mining_powerups.rs`
`507945e9d42d2568572c70bfa3f1c93b2c12ab825c7a56524502fce18359f40a`.
`tests/mining_powerups.rs` is 22 tests, all passing, including the two runtime
placement cases (`[401, 402, 403] -> [403, 401, 402]`,
`[401, 401, 402] -> [402, 401, 401]`) and the direct walk check.
