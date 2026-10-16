# Mining whole-round feasibility probe

Status: **frozen source compiled and driven; the whole-round attempt stops at a dead board —
explained gap, no forced credit.** October 9, 2026 (UTC). Notified freeze `038a135d64fe0c3be6a8e57
eabbe002e7f8b651566b0ca260098175abf16959e` was re-copied byte-for-byte (`source-pin-frozen.txt`,
exact match) and the actual Rust engine compiled and ran in two separate fresh processes.

## Frozen run (actual engine, two separate fresh processes)

| Run | Tick model | Chains | Collected | Phase | Engine wall | Engine remaining | Stop reason |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `model-a-250ms-run2/` **SUPERSEDED** | 250 ms | 1500 (cap) | `[0, 0]` | Playing | 375000 ms | 599000 ms | `chain-cap` |
| `model-b-4000ms-run2/` **SUPERSEDED** | 4000 ms | 450 | `[0, 0]` | Lost | 1800000 ms | 596000 ms | `phase-Lost` |
| `model-a-250ms-run3/` | 250 ms true game tick | 1127 | **`[15, 14]`** | **`LevelComplete`** | 281750 ms | 599500 ms | `phase-LevelComplete` |
| `model-b-4000ms-run3/` | 4000 ms reconstructed, not measured | 450 | `[3, 9]` | `Lost` | 1800000 ms | 596000 ms | `phase-Lost` |

**Superseded:** `run2` hardcoded the fall row as `width - 1 = 13`, so `in_playable(x, 13)` was always
false and no fall was ever reported — its zero-ore conclusion was invalid and its receipts are kept
only as evidence of that defect. `run3` derives the fall row from the reviewed planner's
`PLAYABLE_MAX = 10` (content rect x 3..10 / y 3..10, verified before every run), reports a bottom-row
goal regardless of its idle flag, re-reads engine state before each report, and guards on phase.
`test_settle_model.py` proves both cases first: a row-10 goal yields exactly one `50` and is never
folded into the idle list, and padding row 13 is never reported.

Both runs drove `begin -> accept(39/5) -> OnConnectMsg -> OnStartGameMsg -> OnStartLevelMsg ->
OnChainMsg` with every chain chosen by the reviewed planner, and each chain was settled by the
explicit reconstructed ACK model: `OnIdleMsg` for exactly the live, non-empty, in-playable cells the
gravity pass left `idle = false`, and `OnLevelEventMsg 50 x y` only for a goal value actually present
on the true bottom row, scanned right-to-left. Zero such bottom-row goal tiles existed in either run
(`fall_acks_reported = 0`), so no fall was fabricated and no goal or board was edited.

Result: at the true 250 ms tick the probe completes the whole quota (**collected `[15, 14]`**, 29 fall
reports, 1127 chains) and the engine itself reports **`LevelComplete`** — a full-quota engine
completion, not a native `Won`. At the reconstructed 4 s per chain the run stops at the engine's
**30-minute wall cap** (`wall_ms` 1 800 000, independent of `remaining_ms` 596 000) with quotas unmet
`[3, 9]`; that wall cap is not a 180-second level budget. No collection, win, reward, Close or restart
claim is made beyond the engine's own `LevelComplete` phase.

Owned artifacts:

| Path | Role |
| --- | --- |
| `lab/reports/mining-whole-round-20261009T1645Z/harness/` | private Cargo project compiling a byte-copy of the pinned engine |
| `…/harness/src/main.rs` | lab host: JSON-line protocol over `Type8Engine::load/begin/handle/tick/state` |
| `…/drive_mining_feasibility.py` | driver that imports the reviewed DFS planner to decide every chain |
| `…/source-pin-before.txt`, `…/source-pin-after.txt`, `…/harness-build.log` | pin and failure evidence |
| this note | status, pin table, parity limits, audit path |

No UI, source, test, save, content or release path was touched.

## Pin (provisional)

| Item | Hash |
| --- | --- |
| `prototype/osfr-tools/src/mining.rs` when copied | `153d55ca836a5b9e162c43f0461777d5080ed4190d4a7bf1df6e531ecfd8482a` |
| harness copy `harness/src/mining.rs` | `153d55ca…` (byte-identical to the copy above) |
| live `src/mining.rs` after the failed build | `cba24bbf5bc6579cffbbaa59e43f95280729168bad45894ec6b65aa87f48578e` |
| `opening-harvesting-native-20261009T142624Z/content/mining.json` | `25370f048ec87f12c30b38ac338d1585de3c04bbd2c080185981a1d4c8e33c8b` |
| reviewed planner `lab/tools/play-native-mining.py` | recorded at run time by the driver |

Build attempt: `cargo build --release` in the harness fails with `E0425: cannot find value 'power'`
at `src/mining.rs:1736` and `:1741` — the live module is mid-edit. Log: `harness-build.log`.

## What is ready

* The host compiles *the actual Rust engine* (`mod mining` over the copied `src/mining.rs`) and speaks
  a JSON-line protocol: `begin`, `accept` (39/5), `msg` (39/14 text such as `OnConnectMsg`,
  `OnStartGameMsg`, `OnStartLevelMsg`), `chain` (`OnChainMsg` with the recovered `x,y,` list),
  `tick`, `state`, `quit`. It returns reply text plus the serialized `Type8State`.
* The driver imports the reviewed planner from `lab/tools/play-native-mining.py` and calls
  `mining_active` + `find_path` on the host's state snapshot, so every chain is planner-derived; no
  chain, `OnFall` or board/goal edit is fabricated.
* Bounds are wired: `--max-chains` (default 1500) and `--compute-seconds` (default 60) stop the loop,
  and two **separate** timing models are recorded: model A at the true 250 ms game tick and model B
  at a reconstructed per-chain duration that is explicitly labelled as not measured.

## Parity limits (unchanged by this status)

The probe compares a Rust server engine against a Python planner and JSON snapshots — not against live
mine geometry, the client's grid/movie or an archived capture. A green run would show only that the
pinned engine accepts a planner-derived whole round inside the caps; it would not be a native win, a
faster-time claim, or authority to grant results, rewards, Close or restart. Optional clickables stay
out of scope until root authors content for them.

## Audit path

1. Root notifies the frozen `src/mining.rs` hash.
2. Re-copy the frozen source into `harness/src/mining.rs`, record before/after hashes, and require the
   copy to equal the notified hash.
3. Build, then run `drive_mining_feasibility.py --exe … --max-chains 1500 --compute-seconds 60 --out …`.
4. Write `feasibility.json` (chains, stop reason, engine `remaining_ms`/`wall_ms`, goal progress, both
   timing models) and `host-transcript.jsonl`; update this note with the accepted pin and results.

Cross-links: [[Research/Mining Restoration]], [[Research/Housing Restoration]],
[[../Design/MMO Worker Contract]].
