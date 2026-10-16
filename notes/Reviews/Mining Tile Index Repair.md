# Mining Tile Index Repair

Independent review (2026-10-09, reviewer workspace `/root/mining_review`).
Read-only on production, tests, shared notes, saves, UI and processes; owned
artifacts only: this note and
`lab/reports/mining-tile-index-review-20261009T124833Z/` (private probes).

## Reviewed revision (frozen; hashes verified in-session)

| Path | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/mining.rs` | `e6e421d8e7cd0e923b9f033638f24fb04d648613e9b6719e78a28c3fc45e090e` |
| `prototype/osfr-tools/tests/mining_tile_rendering.rs` | `500c6a828cbf6a0eae62068455d6d9233a8b56a4e82d801d799b51a35e8b5a68` |

Builder evidence bundle: `lab/reports/native-mining-tile-fix-20261009T124532Z/`
(RED 3 failed against the pinned old source, GREEN 3/3, focused
`mining` 21 / `mining_review` 13 / `type8_gateway` 10, scoped `rustfmt --check`
logs empty).

## Verdict

Correct, minimal fix. The recovered movie contract is
`SendTileInfo(category + type, type)` — selectables `400 + GetType()`
collectables `600 + GetType()`, bonus `700 + type`, lootdrop `1000 + type` —
and the client resolves the second field as the per-kind index
(`"SelectableTile" + index`, `aCollectableNames[index * 2]`). Emitting
`kind.category` was the renderer bug; emitting `value - category` with a
checked subtraction is exactly the recovered index, refuses underflowing
content instead of wrapping, and leaves every other byte of the run and its
replies unchanged. No material finding.

## Independently verified

Recovered scripts read directly:

- `lab/decompiled/minigames/mining/game-scripts/__Packages/core/TileGenerator.as`
  (`95554996ebd8a108…`): `SetTileInfo(value, index)` → selectable
  `sIdName = "SelectableTile" + index`; collectable
  `aCollectableNames[index*2]` / `[index*2+1]`; bonus `aBonusNames[index]`.
- `.../core/ServerTileGenerator.as` (`e4c7c0790957e152…`): selectable
  `SendTileInfo(TILE_SELECTABLE + GetType(), GetType())` (line 70),
  collectable `TILE_COLLECTABLE + GetType()` (line 44), bonus/lootdrop the same
  shape (lines 31/57).
- The movie's own server script sends `"OnTileInfoMsg", value, type`; the
  label inventory has `SelectableTile1..5` and
  `Iron_LowQual`/`Iron_MedQual`/`Iron_HighQual` — no `SelectableTile400` and no
  `aCollectableNames[1200]`.

Pinned-diff proof: the private reconstruction script
(`reconstruct-before.py`) reverts only the frozen `initialize_board` hunk of
`mining.rs`; the result hashes to the pinned old revision
`a66073ca463d84dc6e293a5ee3c8eef8ca861a40d23d7a6eb4acd0e4a0d6e4e2`. Because
any other byte difference would break that hash, the shipped change is
provably confined to that one hunk (13 insertions / 1 deletion).

Differential probe (private crate `probe/tests/differential.rs`, 4/4 passed,
log `logs/02-differential.log`): the same synthetic content and the same client
frames drive the reconstructed old source and the frozen fix.

1. `review_differential_start_level_state_and_replies` — after
   `OnStartLevelMsg` the serialized run state is identical (board, RNG cursor,
   `tile_queue`, `random_queue`, score, timers, chains, instance ids); exactly
   seven `OnTileInfoMsg` frames are emitted in both; the old second field is
   the category (`401/400 … 601/600`, `602/600`) and the new second field is
   `value - category` (`1..5`, `1`, `2`); every other emitted text and every
   non-tile-info reply packet is byte-identical.
2. `review_differential_chain_step_keeps_state_identical` — a probed seed with
   a legal comma-format chain (`OnChainMsg x,y,x,y,x,y`) produces identical
   state and identical replies in both versions.
3. `review_malformed_value_below_category_refuses_without_mutation` — a kind
   with `value 399 < category 400` still loads (the spec validator does not
   police `tiles`, as requested); the frozen build refuses
   `OnStartLevelMsg` with "mining tile value below its category" and leaves the
   run at `LevelOffered` with no partial mutation; the pinned old build
   accepted the same content and emitted `OnTileInfoMsg\t399\t400`. No broader
   renderer restrictions were added.
4. `review_old_content_without_optional_asset_names_still_emits_the_index` —
   content without the optional `preload_file` / `launch_movie` fields loads
   through the serde defaults and emits the same seven indices: old content
   stays compatible.

Focused targets re-run in a private target directory, all green
(`logs/03-focused-set.log`): `mining` 21, `mining_review` 13,
`mining_tile_rendering` 3, `type8_gateway` 10.

## Iron bins (601/602)

Both bins reading **Iron** is the original client behaviour, not a mislabel.
`Resources/CodeStringMappings.txt` maps `miningValue601/602/603` to locale
`117144` = "Iron" (one metal name shared by its three quality rows), while the
frame pairing maps index 1 → `Iron_MedQual` and index 2 → `Iron_HighQual`.
After the fix the two collectables therefore render two *different* iron
quality sprites under two "Iron" labels — the correct quality distinction the
board was meant to show. (Whether the live server numbered iron from 0 or 1
remains an open native question from the diagnosis note; the frozen 601/602
values are consistent with Med/High.)

## Boundaries

- Native visual re-verification (five distinct selectable colours plus the two
  iron-quality collectables) belongs to the lead; this review is source-level
  plus engine-level.
- Goals, rewards, timers, pause, score and economy are untouched: the
  differential state equality, unchanged non-tile-info replies and the focused
  regressions cover them.
- The harvesting adapter (`src/harvesting.rs`) builds its kinds as
  `value = 601 + slot`, `category = 600` and shares the mining engine's
  emitter (no `OnTileInfoMsg` of its own), so the same fix applies to it
  without a harvesting source change.
