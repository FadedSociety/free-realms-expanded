# Harvesting Native Mouse Driver

Status: private adapter implemented and self-tested, 2026-10-09 13:37 UTC. Evidence label:
**adapter over ORIGINAL layout constants and content; the screen transform is RECONSTRUCTED and still
unmeasured**. No native input was sent, no process was launched and no package or module was changed.
A native run refuses until the root supplies a measured layout record.

Owned paths:

| Path | Role |
| --- | --- |
| `lab/tools/play-native-harvesting.py` | adapter: namespace guards, geometry, planner, layout gate, bounded loop, self-test |
| `lab/reports/harvesting-mouse-driver-20261009-1337Z/` | self-test, dry-run, refusal logs, synthetic fixture, hashes |
| this note | provenance, reconstruction boundary, pending measurements |

The mining driver `lab/tools/play-native-mining.py` is reused read-only by contract and is not edited
here; the harvesting package, `src/harvesting.rs` and the shared `src/mining.rs` engine are untouched.

## Original facts used

Everything below is read from the shipped client movie/scripts, the gateway content the root
prepared, or the shared engine source. Nothing is invented.

- **Content.** `lab/references/opening-harvesting-native-20261009T132512Z/content/harvesting.json`
  (sha256 `280f5cd7fcca711aaf7e8118ef04195b3bb1241413498325288aca10dff94d20`) declares
  `schema_version` 1, `game_id` **111**, `profile_id` **45** (Chef), `preload_file`
  `FarmingAssets.txt`, `launch_movie` `game_farming.gfx`, and a spec of width **10**, height **10**,
  shape `[1 x 90, 4 x 10]`, selectable `[401, 402, 403, 404]`, collectables `[601, 602, 603, 604]`
  with four quota-15 goals. The instance namespace is **58_000_001** (`FIRST_INSTANCE_ID` in
  `src/harvesting.rs`; the gateway binds it for this campaign).
- **Derived geometry (verified, not assumed).** Applying the shared engine's own shape rule
  (`prototype/osfr-tools/src/mining.rs::board_bounds`: `0` is empty, `4` is the dropzone, everything
  else is playable) to that shape gives playable columns **0..=9**, rows **0..=8**, i.e. a **10x9**
  playable rectangle, with the dropzone row at index **9** inside the 10x10 grid. The adapter refuses
  any other shape instead of guessing; the earlier "8x8" guess is wrong.
- **Movie and wire family.** The activity is the shipped MatchThree "Farming" movie
  (`game_farming.gfx`, sha256
  `02bfc077af1257c140cd9d590257cfc2ab0ba7e45e1814be975f42607342c49f`). `GameClientNetwork.as`,
  `GameServerNetwork.as` and `GameData.as` are byte-identical to the mining movie's
  (`lab/reports/harvesting-native-preflight-20261009T133000Z/preflight-evidence.json`), so the
  client-side drag→chain handling is the same engine.
- **Tile geometry (ORIGINAL).** `GameData.as` sets `nTILE_SIZE = 40`, `nTILE_HALFSIZE = 20`,
  `nMINIMUM_CHAIN_LENGTH = 3`, wildcard `200`, selectable `400..=499`, collectables `600..=699`.
  `core/Grid.as` attaches every tile at `x * nTILE_SIZE, y * nTILE_SIZE` (lines 1386..1433) and
  hit-tests a mouse position as `Math.floor(mcRef._xmouse / nTILE_SIZE)` (lines 248/249). Therefore a
  cell's centre in the **grid clip's local coordinates** is `((x + 0.5) * 40, (y + 0.5) * 40)` and
  the cell pitch is exactly 40 movie units. That is the only part of the point mapping that is
  recovered; the clip's screen origin and the movie-to-screen scale are not.
- **Chain legality (ORIGINAL engine).** `src/mining.rs::validate_chain` accepts one path of at least
  three distinct, idle, orthogonally adjacent tiles whose values are all equal selectable values,
  with wildcard `200` bridging and adopting the chain value. The adapter's planner reproduces those
  rules and prefers the mining driver's heuristic (a chain directly below an unmet collectable).
- **Potential native border-compatibility issue (flagged, not patched).** The original farming
  `Grid.checkFalls()` walks columns with `while (_loc5_ > 0)` (line 1103) and rows with
  `while (_loc3_ > 0)` (line 1119), and `addColumnFall()` walks rows with `while (_loc2_ > 0)`
  (line 1235): all three stop before index **0**. The shipped package's playable area starts at
  column 0 and row 0, so those fall passes would skip the top and left edges of this board. Whether
  the native client needs one cell of border padding is **not** established. Recommendation only, for
  the root to authorize separately if native acceptance shows stuck edge tiles: re-emit the
  reconstructed spec with the playable rectangle shifted to `left = 1, top = 1` (or otherwise padded)
  rather than editing `src/harvesting.rs` or the package. The adapter invents no offset; its geometry
  comes straight from whatever shape the content file carries.

## What is reconstructed

- **The screen transform.** `screen = grid_origin + scale * local`, where `grid_origin` is the screen
  position of the grid clip's local (0,0) and `scale` converts 40-unit movie tiles to client pixels.
  The mining movie measured a 44 px pitch (scale 1.1) at the 1279x960 client, but that measurement is
  for a different movie, so it is recorded as a **hypothesis**, never applied here.
- **The layout record schema** (below) and the operational caps: 45 s budget, at most 10 chains, a 3 s
  all-idle settle window, a 3 s chain-acknowledgement window and a one-second per-chain reserve.
- **The planner heuristic** (below-collectable preference) and the pause/resume discipline, both
  modelled on the reviewed mining driver.

## Pending root measurements

A native run needs a measured layout record passed as `--layout-file`. The adapter refuses without
one, and refuses a record that is missing evidence, binds the wrong movie, lies about the client
rect, carries a calibration that disagrees with its own transform by more than 1 px, or has fewer
than two calibration points:

```json
{
  "schema_version": 1,
  "name": "<operator label>",
  "client_rect": [1279, 960],
  "movie_sha256": "02bfc077af1257c140cd9d590257cfc2ab0ba7e45e1814be975f42607342c49f",
  "grid_origin": [0.0, 0.0],
  "scale": 1.0,
  "pause_button": [0, 0],
  "evidence": {
    "kind": "measured-screenshot",
    "screenshot": "<path>",
    "screenshot_sha256": "<64 hex>",
    "calibrations": [
      {"cell": [0, 0], "screen": [0, 0]},
      {"cell": [9, 8], "screen": [0, 0]}
    ]
  }
}
```

The numbers above are placeholders, not measurements. `grid_origin`, `scale`, `pause_button` and the
two calibration points must come from a screenshot of a running harvesting board at a fixed window
size. Nothing in this repository currently measures them, so a native run stays refused.

## Verification

Commands run from `project/` with `../lab/tools/wiki-env/Scripts/python.exe`:

| Command | Result | Log |
| --- | --- | --- |
| `play-native-harvesting.py --self-test` | 13 checks green: mining-driver contract, content namespace, shipped-rect geometry, parameter-adapted geometry, geometry rejection, state namespace, below-collectable planner, chain rules, all-idle guard, layout validation, layout refusal, timeout/cleanup, CLI guards | `self-test.log` |
| `make-dry-run-fixture.py` | synthetic state fixture built from the real content spec (recorded as synthetic) | `fixture-build.log`, `state.harvesting-synthetic.json` |
| `play-native-harvesting.py --state <fixture> --content <real harvesting.json> --dry-run` | exit 0: geometry 10x10 / playable 0..9 x 0..8 / dropzone 9, planned chain `(2,1)(2,2)(2,3)` value 401 below collectable 601, `screen_mapping: unresolved; pending root screenshot` | `dry-run.log` |
| `play-native-harvesting.py --state <fixture> --content <real harvesting.json> --max-chains 1 --budget-seconds 5` (no layout file) | exit 1, `no measured layout record: native input is refused ...`; the run directory is never created, so zero input commands were sent | `refusal.log` |

The self-test's mapping checks use a deliberately synthetic `selftest-fixture` layout record (origin
400,300, scale 1.1) only to prove the arithmetic: cell (0,0) maps to (422,322) and cell (9,8) to
(818,674), a drifted calibration is refused. That fixture is **not** a measurement and is never used
for a native run.

## Dependency

The task pinned the mining driver at sha256 `476620409f...`; the mining reviewer's narrow
timeout-cleanup/partial-summary revision (`DriverError` on helper timeout, `fg`/`mup` cleanup, partial
`drive_summary`) is the frozen successor, sha256
`735f8eceb1f14e4decbf95d3ae88c9ff55d389799cb8b0a06c6c47cf3f3db594`, and it is what this adapter
binds. The pre-edit revision is preserved at
`lab/reports/native-mining-driver-20261009T1334Z/play-native-mining.47662040.py` and still hashes to
`476620409f...` (the self-test verifies that copy too).

The adapter requires the frozen hash and additionally verifies the reused contract
(`run_input(commands, report_dir, run_dir, label)`, `read_state(path)`, `run_driver(args)`,
`main(argv)`, `PROCESS`), so an out-of-band edit fails the self-test instead of silently changing the
reused behaviour. The self-test output carries both hashes, the preserved-copy check and
`matches_frozen` for the lead; the current run reports `matches_frozen: true`.

## Deliberate gaps and gates

1. **Screen transform unmeasured.** The adapter sends no input until the root supplies the layout
   record above; the dry-run is the only way to inspect the plan today.
2. **Zero-edge border compatibility.** Flagged above with the exact `Grid.as` loops; a spec padding or
   `src/harvesting.rs` geometry change is the root's decision and is explicitly out of this file's
   scope. Do not treat the 10x9 rectangle as native-verified until a real board is driven.
3. **No native acceptance.** Nothing here proves the client accepts a drag, the pause button, or the
   acknowledgement sequence; a root-run single-chain pass is the next gate.
4. **Reused dependency.** The adapter accepts the mining driver's owner revisions by contract; if its
   interface changes again, the self-test refuses and the fix is a one-line contract update.

Cross-links: [[Harvesting Restoration]], [[Mining Restoration]], [[../MMO Work Queue]],
[[../Design/MMO Worker Contract]].
