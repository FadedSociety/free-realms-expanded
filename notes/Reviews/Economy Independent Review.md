# Economy independent review

Read-only review of economy-safety pieces 081, 069 and 086, 2026-10-08 23:05 Pacific. Verdict:
**both production defects are reproduced with real production tests; both candidate modules are
sound in isolation and remain unwired.** No production, shared, content, save, launcher or release
path was edited. Owned paths: `prototype/osfr-tools/tests/economy_review.rs`,
this note and `lab/reports/economy-review-20261009T0545Z/`. Skills: RTK (normal commands),
Ponytail (reuse the candidate modules, no new production code), Caveman (terse report);
no native UI, no process kills and no external messages.

## Reproduced production defects

**E1 — instance-id reuse (piece 081, high).** `Inventory::grant_with_tint` mints
`max(item.id) + 1` (`src/inventory.rs:499-501`), and `consume` deletes a row at zero
(`src/inventory.rs:530`), so retiring the top row frees its id. Repro: a row minted at id 4596,
consumed, then handed out again for a different instance; reloading the same rows from JSON repeats
it. `integration_instance_ids_are_never_reused_after_a_top_row_retires`,
`integration_restart_never_reuses_a_retired_top_instance_id` and
`integration_a_replayed_consume_cannot_retarget_a_reused_id` (ids are wire-addressed, so a replayed
client consume for the retired id consumes the new row). The live campaign already shows retired ids
below the top (30 rows, ids 1..=31, coins 269 in
`lab/state/opening-mmo-continuation-20261009T023433Z/character.json`), so the top-retires case is not
hypothetical. No native proof exists that the client holds a stale id across the retire:
reconstructed-risk boundary, not a native claim.

**E2 — turn-in bundle is not atomic in the state machine (piece 069, high at module level).**
`gameplay::apply_quest` (`src/gameplay.rs:1775-1790`) credits `grant_coins` and then each item as
separate commits; `grant_reward` (`src/gameplay.rs:1605-1619`) has the same shape for opening and
encounter payouts. With a production-legal full bag (4096 rows, `src/inventory.rs:14`) the third
training defeat grants 25 coins, marks quest 900001 `rewarded`, then fails on the item. Repro:
`integration_quest_turn_in_is_atomic_when_the_bag_is_full` (coins 25 vs 0) and
`integration_a_failed_turn_in_is_not_persisted_as_a_partial_reward` (the partial state reloads from
JSON). Actual-integration boundary: the gateway binary stages every message
(`src/bin/osfr-gateway.rs:198-206`), so an `Err` drops the partial state and the *file* keeps the
previous turn-in; the player still loses the completed kill to a session error instead of a
retryable refusal, and any non-staged caller sees the partial state.

**E3 — coin sink side is still absent (piece 086, verified clean, residual).** Re-ran the audit
checker: `violations 0`, `implemented_sources = quest_reward, checkpoint_trial, encounter_clear,
opening_stage, opening_discovery`, findings include `quest-reward-atomicity`. Source scan confirms
the single production balance mutation `self.state.coins = coins` inside `grant_coins`
(`src/inventory.rs:437`) with `checked_add` plus the `i32::MAX` guard; production has no debit site.
The queued sinks carry their own arithmetic over parallel balances (piece_084 `checked_add`/subtract,
piece_085 `saturating_add` plus explicit underflow check, piece_058 revive debit, piece_083 coin
map), so each must route through the one `Inventory` balance when registered.

Non-finding, recorded so it is not patched: at `MAX_ITEMS` the full bag serializes to 650,201 bytes
(inventory) / 650,482 bytes (whole gameplay state) against the 1 MiB snapshot bound, so the two
bounds agree today.

## Candidate fixes assessed in isolation

`candidate_*` tests include the candidate modules by `#[path]` and pass today (10 passed in the
review target): the counter is monotonic across retire and restart, burns an id whose grant never
landed, repairs a stale floor durably, refuses duplicates, a behind counter and corrupt documents,
and exhausts at `i32::MAX` without rewriting. The turn-in ledger records the intent before the
caller applies it, answers the exact request with the same staged bundle, commits idempotently,
settles from persisted state without a second grant, and refuses stale or unproven rewards. Neither
module is referenced from production, and the ledger cannot be wired until quest progress can be
completed-but-unrewarded: `quests.rs:444-445` flips `completed` and `rewarded` together and
`quests.rs:422` skips completed quests.

## Golden invariants preserved

The existing ignored private turn-in oracle
(`private_complete_loop_reward_once_atomic_bundle_and_fresh_restore`, canonical lab
reference/content/capture) passes 1/1, and the module-level guard
`guard_reward_producers_stay_once_only_in_the_golden_path` passes: 25 coins and exactly one item on
the third defeat, nothing on a trailing defeat. No rewards are minted by this review.

## Lead patch recommendation

1. `src/inventory.rs:497-502`: take the id from the durable counter instead of `max(id) + 1`; open
   it where gameplay state loads (`src/gameplay.rs:105`) over the live ids plus content
   `initial_state` ids; never lower it in `consume`/`equip`/load. Retire the two characterisation
   tests in `tests/piece_081.rs` when the call site changes.
2. `src/gameplay.rs:1775-1790` and `1605-1619`: apply the whole bundle into one staged `Inventory`
   (and staged quest state for the quest path) and commit once; on failure return ordinary refusal
   feedback so the kill and the turn-in stay retryable instead of erroring the session.
3. `src/quests.rs:444-445` (with `422`): stop flipping `rewarded` inside `enemy_defeated`; keep the
   completed-but-unrewarded pending state and let the turn-in (piece 069 `consider`/`commit`) own the
   flip, so a retry can pay and a crash can settle.
4. Lead-owned `lib.rs` registration for `piece_069`/`piece_081` once 1-3 land.

Verification: `scripts/cargo.ps1 test --test economy_review -- --nocapture` → 10 passed, 5 failed
(the five `integration_*` failures are the reproduced defects and are the acceptance tests for the
patch); `--test piece_081 --test piece_069 --test piece_086` → 31 passed, 0 failed, 2 ignored;
`scripts/pieces/piece_086.py` → violations 0.

Evidence: `lab/reports/economy-review-20261009T0545Z/` — `review-tests-rerun.log` `D5BC36D3…`,
`review-tests-first-run.log` `6AD9FDDC…`, `candidate-suites.log` `2D7DFDFD…`,
`golden-private-turnin.log` `999720E9…`, `piece-086-checker.log` `5001A925…`,
`coin-mutation-scan.log` `3665B36C…`; review test SHA-256 `2C17C4B9…`.

Cross-links: [[Research/Pieces/081 Inventory instance lifecycle]],
[[Research/Pieces/069 Quest turn-in transactions]],
[[Research/Pieces/086 Currency and economy audit]], [[Design/MMO Worker Contract]], [[MMO Work Queue]].

## Lead repair review (append, 2026-10-08 23:25 Pacific)

The lead's repair is independently re-verified on the 23:15:35 Pacific revision: `inventory.rs`
SHA-256 `5DEACE9351F9…`, `gameplay.rs` `FD1A9A1D0266…`, `quests.rs` `64ABCE60F663…`, `lib.rs`
`1DA6866E8E48…`, `piece_081.rs` `0FB30FD60897…`, `tests/economy_review.rs` `EDBFDD678E5A…`. All
five previously RED integration tests now pass and the three lead-added tests pass: 18/18
(`review-target-v2.log`).

Verified in the code: `InventoryState.next_instance_id` is `Option<i64>` with serde default and
skip-if-none and is validated to lead every live id (`inventory.rs:56-61`, `347-353`); the mint calls
`piece_081::allocate_from_snapshot` instead of `max(id) + 1` (`inventory.rs:506-512`); `grant_bundle`
stages one clone for coins plus every item (`inventory.rs:539-552`); `apply_quest` clones inventory
and quest state, acknowledges only after the whole bundle lands and otherwise replies an ordinary
refusal with the reward pending (`gameplay.rs:1770-1795`); `acknowledge_reward` owns the `rewarded`
flip (`quests.rs:311-318`). Focused suites pass 76 with zero failures plus the 18 in this target;
the private golden turn-in, the byte-for-byte original-client capture replay and the 1386-instance
captured-self round trip all pass, so the optional field leaves wire and default bytes unchanged.

Residuals: a pending reward re-offers on any authoritative defeat because the reward push is not
gated by the defeated definition (`quests.rs:455-467`; only the training enemy exists in production
today), and each blocked retry repeats its "Completed" line; opening/encounter bundle failures still
surface as session errors rather than refusals, with no persisted partial state because the binary
discards the whole staged gateway clone and the stage stays re-triggerable; piece 069's ledger
correctly stays unwired; the unused sidecar half of `piece_081.rs` still compiles into the crate but is
never constructed. Reviewed-hunk detail, an applyable proposed review test patch (not applied) and
hashes: `lab/reports/economy-review-20261009T0625Z/`.
