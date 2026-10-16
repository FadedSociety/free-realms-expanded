# Native Mining Mouse Driver

Independent read-only review, 2026-10-09. Reviewed revisions, both on
`lab/tools/play-native-mining.py`: the original frozen driver
`c453636d78062d3b68d26cdd2748ba2bda1b2371d0219014e84ee58864380f0e` (37,287 bytes) and the housing
successor `c54c624d144451bde14ae0976263498e690e9f6d1e60e63da44c73083258302a` (40,169 bytes). The file
moved twice during the session (`c453636d…` -> `ed7bd923…` at 06:03:31 -> `c54c624d…` at 06:03:57);
the middle snapshot briefly called a removed `parse_rect_reply` in its self-test and was superseded
within the minute, so only `c54c624d…` carries the fix below and its hash should be re-pinned before
handoff. Supporting revisions: `lab/tools/WinDrive-Dpi.ps1`
`009d3aed9e256f9a79e1ba9d86381a6b7e99a58c5f97e92eece88f59c833d08c`,
`project/scripts/drive-local-game.py`
`29e24dcd5cac4ea8340adc5341e3de86988578952789ac30d2931320d3c82e5c`.

Companion bundle: `../lab/reports/mining-driver-review-20261009T125949Z/` (self-test and dry-run
logs, the native rect transcript, PNG size proof, source hashes, manifest). No source/helper edit,
no native input, no save/content/process/UI change; the only executed driver modes were
`--self-test` and `--dry-run`.

**Verdict: the successor `c54c624d…` fixes both first-step blockers and its self-test now pins the
native transcript; the planner, geometry mapping, guards and bounding remain sound. The remaining
open issues are the failure-path mouse release, the un-sequenced pause/ACK ordering, and the
terminal-phase reporting gap.** In `c453636d…` the driver could not complete its first native step:
`verify_window_rect` parsed the helper's `ready window` handshake as the rect reply (safe stop
before input), and the required client rect `1280x960` did not match this client, which reports and
renders `1279x960` in two independent runs. `c54c624d…` implements the evidenced fix reviewed below.

## Findings

1. **Blocking (first native step): the rect parser takes the helper handshake line.**
   `run_input` records the helper's whole stdout, whose first line is the handshake
   `ready window`/`ready no-window` (`WinDrive-Dpi.ps1:242`); the command reply is the line after it.
   `verify_window_rect` (`play-native-mining.py:448-460`) parses `replies[0]`, so the native
   transcript `ready window\n192 41 1279 960\n` produced
   `window rect reply is malformed: 'ready window'` and the driver stopped after `fg` + `rect` with
   no input commands recorded (bundle `native-rect-transcript.txt`; run
   `lab/runs/client-baseline-20261009T125556Z/driver-chain1/driver-inputs.jsonl`). Recommended fix:
   skip the `ready …` handshake and parse the first real reply strictly (exactly four integers,
   `rect == none` still refused), and pin the exact native stdout as a self-test vector. Do not make
   the parser tolerant of arbitrary lines.
2. **Blocking (data): `REQUIRED_RECT = (1280, 960)` is wrong for this client.** The native `rect`
   reply is `192 41 1279 960`, and the run's own PNGs are 1279x960
   (`mining-intro.png` `8bfaf9b4…`, `world.png` `3365310f…`, `mining-offer.png` `0d6fce26…`), as is
   the earlier mapping-proof run's `mining-board.png` (`96466aac…`, 1279x960). A qualified rule is
   justified: the row mapping depends on the 960-pixel height, and the proven x mapping only spans
   columns 3..10 (x 482..812), so a one-pixel client width difference cannot move any planned tile;
   but the accepted size set must be evidenced (accept the observed 1279x960, and 1280x960 only if
   an actual 1280-wide client is captured), not relaxed by tolerance. The `--layout 1280x960` flag
   should name the logical layout while its check uses the evidenced rect.
3. **Medium (user safety): a refused `mup` can leave the left button held.** The helper guards every
   `Send` with the foreground check (`WinDrive-Dpi.ps1:134-148`) and catches per-command errors, so a
   lost focus between `mdown` and `mup` yields `error …` for the release and no release happens;
   `run_input` then raises and `drive_chain` (`play-native-mining.py:525-531`) has no finally-path
   release. The 45-second `subprocess.run` timeout is also uncaught, with the button possibly down.
   Recommended: an unguarded release command in the helper plus a `try/finally` release attempt in
   the driver; presses keep the foreground guard.
4. **Medium (lifecycle): the pause click is not sequenced behind the chain acknowledgement.** The
   engine rejects `OnChainMsg` while `paused` (`src/mining.rs:576-579`), and the gateway ends the
   whole session on a frame error (`src/bin/osfr-gateway.rs:199, 321-328`), so the failure mode of a
   pause-message-before-chain-message race is a client disconnect, not just a stop. The current
   order (drag, then `ensure_paused`, then `await_ack`) is safe only because the pinned helper takes
   ~150 ms per command, which lets the chain reach the server first; that timing is not verified by
   the driver. Recommended: poll for `chains+1` and a changed board **before** pausing, then pause
   for the stable snapshot.
5. **Medium (terminal transitions/reporting): quota completion and loss end the run as an error and
   discard the record.** `validate_context` requires phase `Playing` (`:179-180`), so the read after
   a quota-completing chain (phase `LevelComplete`) or a timer loss (phase `Lost`) raises
   `unexpected phase …`; `await_ack`/`ensure_paused` propagate it, `main` prints
   `{"ok": false, "error": …}` (`:832-834`), and the `executed` list plus `quotas_met` summary never
   materialize. Recommended: treat a non-`Playing` phase inside the loop as a normal bounded stop
   and report phase, quotas and the executed chains.
6. **Low (reporting): `below_ore`/`ore` describe only the first tile.** `rank()` scores
   `any((row-1,col) in ore_cells …)` over the whole path (`:280`) while `find_path` fills
   `ore`/`below` from `tiles[0]` only (`:337-344`), so a path that clears below an ore under a later
   tile would be reported as `below_ore: false`. Not triggered by either dry-run board, but
   reachable. Recommended: report the tile/ore that produced the ranking, or the list of clears.
7. **Low (hardening): a missing `idle` array disables idle gating.** `validate_context` allows
   `idle is None` (`:184-190`), `idle_flags` returns `None` (`:251-260`) and `chainable` then skips
   the idle check (`:272-277`), contradicting the stated rule that every planned tile must be idle.
   The frozen engine always serializes `idle`, so this is defense-only; refuse a run without it.
8. **Low (hardening): the window rect is verified once per invocation** (`:546-548`), not before
   each chain, so a mid-run resize is not re-checked; `await_ack` also requires `chains` to be
   exactly `+1` (`:516`), so any duplicate/cascading ack times out. Both are cheap to tighten.

## Verified sound (bounded paths, pause/resume, ACK, budget)

- Geometry: `physical()` (`:94-99`) maps client pixels with the native-proven drag
  `(4,7)(4,8)(4,9) -> (526,530)(526,574)(526,618)` and the playable box 3..10; the helper converts
  client to screen (`WinDrive-Dpi.ps1:150-155`), so the window position cannot shift a plan.
- Guards: native runs require an explicit `--layout` and a live rect check (`:535-536, 448-460`);
  `check_foreground` re-focuses and rechecks (`:436-445`); every helper `Send` re-checks the
  foreground/no-foreground-over-window condition (`WinDrive-Dpi.ps1:104-148`).
- Plan legality versus the engine: the DFS (`:263-345`) is a strict subset of
  `validate_chain` (`src/mining.rs:1348-1390`) - at least 3 orthogonal, distinct, idle
  selectable-or-wildcard tiles of one colour; it never plans an all-wildcard chain (the engine
  allows one) and never a value outside 401..405, and it checks bounds before indexing.
- Bounds/budget: path cap 12, node budget 50 000 with loop break, `--max-chains` 1..10 and
  `--budget-seconds` <= 45 enforced in the CLI (`:73-74, 285-321, 811-814`).
- Pause/resume: the driver reads state before every toggle, never double-toggles, and bounds each
  settle wait to 10x250 ms (`:476-504`); the acknowledgement check requires both `chains + 1` and a
  changed board while paused (`:507-522`).
- No protocol/save/content writes: input is mouse-only through the pinned helper; the only writes
  are `driver-inputs.jsonl` and the optional run `external-inputs.jsonl` (`:393-433`); nothing
  clicks Quit/Close. `--self-test` and `--dry-run` perform no input (`:539-540, 602-787`), and the
  dry-run path never calls `run_input`.

## Native-run observations (dry-run only; input belongs to the root)

- `--self-test` passes all 14 checks; the CLI refuses a native run without `--layout` (exit 2) and
  a stale instance (exit 1) - raw outputs in the bundle.
- Fresh board (instance 18000002, paused, 0 chains): the best legal plan is the 4-tile vertical
  401 run `(7,7)-(10,7)` with `below_ore: false`; no legal same-colour chain currently clears below
  a visible unmet ore, so a single bounded chain may add score but not change `collected`.
- 121834 before-chain board: the plan `(4,9),(4,8),(5,8),(5,7)` does clear below the unmet 601 at
  (3,9), matching the earlier accepted class of path.

## Re-review criteria for the successor hash

1. The parser skips the `ready …` handshake, parses exactly one four-integer reply, still refuses
   `none`/malformed replies, and carries the exact native stdout
   `ready window\n192 41 1279 960\n` as a self-test vector.
2. The rect rule accepts only the documented, evidenced client size(s) - at minimum 1279x960 with
   the PNG/hash proof - and refuses everything else; height 960 stays exact.
3. The other guards, planner rules and bounding are unchanged; the new hash is recorded and the
   self-test plus both dry-runs are re-run and compared against this bundle.
4. Findings 3-5 above are either fixed or explicitly accepted by the lead; if the pause/ACK order
   stays as is, the record must state that a paused chain is a session-ending server error.

## Successor review (`c54c624d…`, verified)

Re-review criteria 1 and 2 are satisfied; criteria 3 and 4 remain open.

- **Handshake parsing fixed and covered.** `parse_rect_stdout` (`:469-492`) drops only the exact
  `ready window` line (`HANDSHAKE_LINE`, `:73`), requires exactly one remaining reply, and parses
  exactly four integers; empty, multi-reply, `ready no-window`, `none` and three-integer inputs are
  refused. The self-test now pins the real transcript `ready window\n192 41 1279 960\n`, a nominal
  1280-wide reply, and all refusal cases (`:805-819`, check `rect_transcript_parser`).
- **Rect rule is evidenced, not relaxed.** `REVIEWED_RECT = (1279, 960)` (`:72`);
  `check_reviewed_rect` (`:495-508`) accepts exactly that size and refuses 1280x960, 1278x960 and
  1279x959, citing the 121834/125556 PNGs. `REVIEWED_LAYOUT_LABELS = ("1279x960", "1280x960")`
  (`:70-71`) keeps the nominal lab label working: a dry-run with `--layout 1280x960` reported
  `"layout": "1279x960"` and `"reviewed_rect": [1279, 960]`.
- **Ran on the successor (read-only modes only).** `--self-test` -> `{"ok": true}` with all 14
  checks including `rect_transcript_parser` (`successor-self-test.txt`, exit 0);
  `--dry-run --layout 1280x960` on the paused 18000002 board -> exit 0 with the plan reported under
  the canonical 1279x960 layout (`successor-dry-run-1280-label.txt`). The dry-run's best legal
  plan is `(9,8),(9,9),(10,9)` with `below_ore: false` on the post-manual-chain board.
- **Still open in the successor** (line numbers for `c54c624d…`): the in-band `mup` with no
  finally-path release (`drive_chain`, `:560-566`); the pause click that still precedes the ACK
  poll (`await_ack` requires `paused`, `:549`); the `Playing`-only context gate so a
  quota-completing or losing chain still ends as `{"ok": false}` without its record; `ore`/`below`
  still taken from `tiles[0]` (`:338-349`); missing `idle` still accepted (`:191`, `:281`); the
  invocation-level rect check (`:584`) and exact `chains+1` requirement (`:549`). The `--layout`
  argument help still says "required (1280x960) for a native run" (`:851`) while the canonical
  layout is 1279x960 - cosmetic drift to correct with the next touch.

## Successor-2 review (`4766204…`, verified)

Reviewed revision: `476620409f12caf452c10e0db8d4315100e64240085c3bfbe7bb137b52b60a2c` (45,046
bytes). Housing's same-harness record is `../lab/reports/native-mining-driver-20261009T1309Z/`; my
independent evidence is in this review bundle (`successor2-*`).

**Fixes verified in the code.** (1) ACK before pause: `await_ack` (`:560-604`) now polls while the
board is unpaused, sends no input, accepts `chains >= before+1` plus a changed board, and treats a
terminal phase/`end_sent`/exhausted remainder as a clean stop; the pause click happens only after
the acknowledgement (`run_driver` `:690`, `:716`). (2) Terminal and budget stops: `run_driver`
returns `ok: true` with `stopped: "terminal <phase>"`, a `terminal` block and the chain record
(`:702-714`), or `stopped: "budget"` (`:663-665`, `:682-684`) with a per-chain reserve;
`ensure_paused`/`resume` check the deadline before their clicks (`:532`, `:549`) and `await_ack` is
deadline-bounded. (3) Guarded release: `release_mouse` (`:607-626`) re-checks `fg` and only then
sends `mup`, reporting `released with guarded mup`, `release skipped … (left button may be held)` or
`release FAILED …`; `drive_chain` calls it on a drag failure (`:635-639`). (4) Mandatory idle: the
legacy `None` bypass is gone (`idle_flags` returns the 196 booleans; validation refuses missing
ones). (5) Reporting: `ore`/`below_ore` now come from the tile that actually clears an unmet ore
(`clearing = next(...)`, `anchor`, `:355-364`), and the `--layout` help names 1279x960.

**Independent reproduction.** Re-running the same mock harness against the preserved predecessor
copy (`native-mining-driver-20261009T1309Z/play-native-mining.c54c624d.py`,
`c54c624d…`) reproduces 8 failures (`successor2-mock-red-independent.json` `5915f176…`), and against
the frozen driver 8 passes (`successor2-mock-green-independent.json` `bcf8065a…`) - byte-identical
to housing's own red/green JSONs, so the pair is deterministic and hash-pinned. `--self-test` on the
frozen driver exits 0 with the 14 checks (`successor2-self-test.txt` `f572ec78…`).

**Live evidence (root-owned, read-only).** `driver-chain2/driver-inputs.jsonl` (`abd34e5d…`) records
foreground-check, window-rect, `resume`, `drag-b8` (focus, `mdown 702 618`, `move 746 618`,
`move 746 662`, `mup`, move away) and `pause` last, matching the ACK-before-pause order;
`driver-batch1/driver-inputs.jsonl` (`b3a7be67…`) shows four more chains, each
`resume -> drag -> pause`, all exit 0. The live state (`state/mining-tile-native-20261009T124608Z/
character.json`, `7c8967e1…`) is paused with `chains 6`, `score 3000`, `collected [0, 0]`.

**New operational finding: the live board is now chainless, and zero-path is a hard error.**
`successor2-live-board-stats.txt` (`2406373a…`) shows all 64 playable tiles idle with values
`404x17, 402x14, 405x12, 403x6, 401x3, 200x1, 601x5, 602x6`; a dry-run on the frozen driver returns
`{"ok": false, "error": "no legal chain on the board (zero valid path; budget 50000, visible unmet
ores=11)"}` (`successor2-dry-run-live.json` `d8966bc9…`). The board genuinely has no 3+ same-colour
orthogonal chain, and `collected` is still `[0,0]`: none of the six chains cleared below a visible
ore, so the driver cannot move this board toward the quota. Recommended follow-ups for the lead:
treat zero legal path as a clean `stopped: "no chain"` summary that carries the executed chains
(same shape as the terminal/budget stops); consider stopping after N chains that clear no ore so a
bounded test does not burn a board without progress; and use a fresh board to continue native
testing. Remaining minor items: the window rect is still verified once per invocation, the 3.0 s
acknowledgement timeout and 1.0 s per-chain reserve are tuned to the ~150 ms helper cadence (they
stop rather than send late input), and a helper `TimeoutExpired` is not a `DriverError`, so a hung
helper would bypass the cleanup-release path - low likelihood, worth catching explicitly.

## Timeout and receipt repair (`735f8ece…`, 2026-10-09)

Bounded driver-only patch for the two remaining gaps above (private probes
`lab/reports/native-mining-driver-20261009T1334Z/`; predecessor `47662040…`
preserved there as `play-native-mining.47662040.py`):

- `run_input` now converts `subprocess.TimeoutExpired` into a `DriverError`
  ("helper timeout after 45s on <label>") and still records the attempt in
  `driver-inputs.jsonl` (`timeout: true`, `exit_code: null`), so a hang after
  `mdown` reaches `drive_chain`'s guarded cleanup: `fg` is re-checked and `mup`
  is only sent while `FreeRealms` is foreground; a lost foreground sends
  nothing and reports `release skipped`.
- `run_driver` attaches the drive summary (per-chain receipts, `stopped:
  "error"`, `stop_error`) to any `DriverError` raised inside the batch, and
  `main` prints it as `{"ok": false, "error": …, "summary": …}`, so a
  no-legal-chain stop or a mid-batch timeout no longer discards earlier
  receipts.

RED (predecessor `47662040…`) -> GREEN (`735f8ece…`), mocked probes only
(`timeout-probes-red/green.json`):

| probe | predecessor | successor |
| --- | --- | --- |
| `timeout_release` | FAIL raw `Command 'helper' timed out after 45 seconds`, no cleanup | PASS (`cleanup-fg` + `cleanup-mup`, error reports timeout and release) |
| `timeout_release_lost_focus` | FAIL same escape, no guard | PASS (no `mup`, reports skipped) |
| `no_chain_receipts` | FAIL `{"ok": false}` only | PASS (`summary.executed` keeps the chain receipt) |
| `timeout_receipts` | FAIL uncaught `TimeoutExpired` after chain 1 | PASS (timeout reported, receipt kept, guarded release) |

Re-run on the successor: the earlier 8 mocked review probes pass with no
failures, `--self-test` passes all 14 checks, `python -m py_compile` exit 0.
The piece028 Harvest adapter interface is unchanged (`run_driver(args)`,
`main(argv)`, CLI flags, `DriverError`); the only additions are the `summary`
key on failure payloads and the optional `DriverError.drive_summary` attribute.
