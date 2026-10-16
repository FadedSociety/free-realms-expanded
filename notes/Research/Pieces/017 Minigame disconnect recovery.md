# 017 Minigame disconnect recovery

Status, October 9, 2026: semantics recovered and conservation checked against
preserved native evidence. **No new native acceptance.** The deliverable is a
read-only checker (Rust + Python) over the existing reconstruction, not a new
recovery implementation.

## Original facts

- The shipped mini-game framework has no client-side resume or refund path.
  `lab/decompiled/minigames/<movie>/scripts/__Packages/com/sonyonline/game/SoeGameFramework.as`
  lines 451-453: `OnClientDisconnect()` calls `GameClientDisconnect()`, whose
  default body at line 541 is empty. That file is byte-identical across
  Cooking, Smelting, Forging, Mining and Harvesting (SHA256
  `6488f1b88ff081c971ac857bda91ea7c64a83418ca0af00bbe5a49744524a879`), and no
  movie overrides the hook (`rg GameClientDisconnect` matches only it).
- Win/Loss is only produced through server messages: `OnNextLevelMsg` compares
  the level goal with the score before calling `GameLost`/`FinishLevel`, and
  `GameWon`/`GameLost` (lines 266-285) are additionally guarded by
  `IsInProgress()`. A dropped connection reaches none of them; the framework's
  `StopGame(reason)` path only stops the level, with no refund or reward code
  anywhere in the movie.
- The supplied community server carries no mini-game session state. In
  `lab/decompiled/server/Gateway`, the only mini-game hit is the
  `clientItemDefinition.NonMiniGame` item flag in `Gateway.Player/PlayerCode.cs`;
  `lab/decompiled/server/LibSOE/SOE/BaseMiniGamePackets.cs` is a packet-type
  enum. There is no server-side resume, timeout or refund handler to recover.
- Native evidence - mid-round Cooking disconnect,
  `lab/runs/client-baseline-20261008T162238Z/`: the client's own malformed
  `FRServer_StatUpdate` (39/14, frame 2972, missing closing quote) ended the
  session (`session_ended`, `invalid float literal`) during Chop. Before/after
  gameplay differs only in the cooking attempt bookkeeping
  (`active` Offered 46000001 -> Playing 46000002, `next_instance_id`
  46000002 -> 46000003). All 18 inventory instances, 4x each of
  2862/2875/2884/2900, wallet 110, equipment, quests and opening state are
  identical. Proof `analysis/native-cooking-launch-proof.json` SHA256
  `14e687108c2e1c5cd88750f981a0c252db235bd6cac073f62a6d20e9abf880cc`.
- Native evidence - abandoned Forging run and server restart,
  `lab/state/opening-after-smelting-forging-incomplete-173814.json` ->
  `lab/state/opening-after-smelting-restart-before-retry-181038.json`. The
  complete gameplay object differs in exactly one field:
  `forging.active` (Playing 139000001) becomes `null`. Wallet 130, every
  ingredient stack (5x 2752/55/56), equipment, quests, `completed` counters and
  the allocated `next_instance_id` are unchanged. Character pose differs
  separately (native float rotation round trip, see
  [[../Reviews/Native Forging Cancellation Restart 181038]]).

## Inferred meaning

A dropped connection cannot be distinguished from an abandoned session by the
recovered code, and nothing in the shipped movie or supplied server can resume
one. Cancelling the persisted run is therefore the only semantics the evidence
supports; the open question is only **when** the cancel is applied and what it
may touch. A pre-win run owns no consumed materials, so cancel must not consume,
award, advance quests or reuse an attempt id.

## Community corpus check

`um` is not on PATH in this checkout, so the pinned read-only corpus
`../lab/references/freerealms-wiki/wiki.sqlite` was queried directly (SQLite FTS
over `chunks_fts`). Queries `disconnect`, `crash`, `minigame`,
`"connection lost"` and `"lost items"` return **no** statement about an
interrupted mini-game, lost ingredients or reconnect behaviour; the hits are
unrelated (Crash Cup, Abacus Coinmiser's "coin crash", Demolition Derby). No
community claim exists to corroborate or contradict the reconstructed cancel
policy - that gap stays explicit.

## Reconstructed policy (local, not historical)

`prototype/osfr-tools/src/gameplay.rs:425-441`: `initialize_world` cancels
`smelting`, `cooking` and `forging` on every world load, "Keep attempt
IDs/completions, cancel without consuming or awarding". The service `cancel()`
methods (`cooking.rs:257`, `smelting.rs:245`, `forging.rs:239`) take the run and
emit the native state-removal 39/19 for its instance unless a result page was
already presented. `src/bin/osfr-gateway.rs` re-loads the persisted snapshot for
every new session and saves the staged state before queuing replies, so a
disconnect persisted mid-movie is cleared on the next `Loaded` event.

## Checker and tests

- `prototype/osfr-tools/src/restoration/piece_017.rs` -
  `audit_conservation(before, after)` and `audit_recovery(before, after)` over
  persisted snapshot JSON. They reject lost or extra materials per
  `(definition_id, tint)`, wallet/equipment/quest changes, regressed attempt
  counters, reused instance ids, runs that survive a reload, and terminal runs
  (those belong to the crafting transaction tests).
- `prototype/osfr-tools/tests/piece_017.rs` - seven always-on contract tests
  (positive and tamper-negative) plus two `#[ignore]` tests that audit the two
  real evidence pairs above. Included by `#[path]` until the lead registers the
  module; no shared file is touched.
- `scripts/pieces/piece_017.py` - the same audit over the lab corpus, reporting
  `forging_abandon_restart_conservation`,
  `cooking_midround_disconnect_no_consumption` and
  `rust_module_checker_present`, with input SHA256s.
- `tests/pieces/test_piece_017.py` - stdlib tests for the checker, the negative
  cases and the real evidence pair.

## Commands and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test piece_017` before the module existed | RED: `couldn't find file tests\..\src\restoration\piece_017.rs` |
| `python tests/pieces/test_piece_017.py` before the script existed | RED: 1 failure, 8 errors (missing `scripts/pieces/piece_017.py`) |
| `scripts/cargo.ps1 test --test piece_017` | GREEN: 7 passed, 2 ignored |
| `scripts/cargo.ps1 test --test piece_017 "--" "--ignored"` | GREEN: 2 passed (both real evidence pairs; PowerShell consumes a bare `--`) |
| `python tests/pieces/test_piece_017.py` | GREEN: 9 passed |
| `python scripts/pieces/piece_017.py --output <lab report>` | GREEN: 3 checks, status pass |
| `scripts/cargo.ps1 test --lib` | GREEN: 24 passed |
| `scripts/cargo.ps1 test --test cooking_gameplay -- --ignored` | GREEN: 2 passed (production reload/cancel/conserve path) |
| `scripts/cargo.ps1 test` (whole workspace) | Blocked by other workers' in-flight targets: `piece_013`/`piece_021` module files do not exist yet and `piece_019` has a type error. None of them include piece 017. |

Logs and report: `../lab/reports/piece-017-20261009T025526Z/`.

## Open gates

- Fresh native disconnect/reconnect on the current release: launch a crafting
  movie, kill or disconnect the client mid-round, reconnect with the same save,
  confirm visible world restore, unchanged materials and a fresh attempt id.
  The existing native pairs prove the semantics on the captured revision only.
- Mining, harvesting, fishing, kart racing, soccer and derby recovery stay with
  their own work items; this checker deliberately covers only the three
  material-consuming crafting movies.
- Terminal (Won/Lost) boundaries and their rewards remain owned by the crafting
  transaction tests; this checker refuses them instead of guessing.
