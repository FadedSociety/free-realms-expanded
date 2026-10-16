# Mining Clickable Powerups Review

Independent read-only review of the mining clickable powerups (950-953)
implementation. No source, content, UI, save or process was changed by this
review; owned artifacts are this note and
`../lab/reports/mining-clickable-review-20261009T162752Z/`.

Revision under review: `src/mining.rs` `a9f66c1a184391f8…` (the builder moved it
during the review: `3e6cc6a5` -> `7c847d4f` -> `a9f66c1a`); `tests/mining_powerups.rs`
was `5bfbf8ad7371c00e…` at the probe run and has moved again since. The builder is
still finalising, so the 14/15 GREEN suite is treated as partial; findings below
are pinned to the `a9f66c1a` module hash.

## Who owns the randomness (recovered)

The server owns the draw; the client's list is a claim. Both sides hold the same
`aRandomNumberQueue`: the server sends batches (`GameServerNetwork.as:26`,
`SendRandomNumberQueue`), the client stores them (`GameClient.as:197`,
`GameClientNetwork.as:72`, `Grid.as:10/101/150`), and both map a draw as
`start + queue.shift() % (end - start + 1)` (`Grid.as:649-651`,
`ServerGrid.as:783-808`). Every clickable action rebuilds its pool and requires
the client list to name the server's next draws in order:

| effect | server reference | pool / check |
| --- | --- | --- |
| 950 dynamite | `ServerGrid.StartShatter` 889-955, `ShatterValidateNextTile` | two-level BFS `aShatterArray1/2`; each next tile must equal the queue draw |
| 951 wildtile | `ServerGrid.StartWildTiles` 988-1046 | idle selectables; each named tile must equal `GetRandomTile()` |
| 952 destruct | `ServerGrid.StartDestruction` 1102-1165 | idle selectables + blockers; same ordered check |
| 953 shuffle | `ServerGrid.StartShuffle` 1166-1249 | pressed tile becomes wildcard, then idle tiles are drawn and rotated |

`GameLevel.OnActionMsg` 507-542 splits the list, takes `power` from the spawn row
and calls those functions; any mismatch returns false and stops the game with
`ActionMsgFailure`. The client builds the same list from its queue
(`Grid.as` 1437-1695) and refuses to press when `aRandomNumberQueue.length < power`
(1526/1573/1627). The wire format is unchanged: `OnActionMsg\t<value>\t` plus
`x,y,` pairs, pressed tile first.

## Independent probes (7 passed, 0 failed, pinned hash)

Private crate `../lab/reports/mining-clickable-review-20261009T162752Z/probe`
compiles the current module and the pre-implementation `mining-da3b.rs` baseline;
`probe-pinned.log` is the run. It adds what the builder suite does not cover:

- `default_percent_matches_the_da3b_baseline` - `powerup_percent = 0` without the
  powerup key produces byte-identical replies and state against `da3b`.
- `queue_derived_list_is_accepted_and_forged_list_is_refused` - the queue-0 draw
  list is accepted; a non-drawn tile and the same list under a different draw are
  refused (ordered shared-queue match now enforced).
- `dynamite_shatter_is_serviced_from_the_shared_queue` - 950 explodes the drawn
  neighbour and rejects a non-drawn one, so all four effects are serviced.
- `forged_eligible_list_must_not_be_queue_independent` - eligibility alone is no
  longer sufficient.
- `collected_powerups_are_refused` - 900/901/902 authoring still fails closed, so
  no inert collected-powerup can spawn.
- `powerup_tiles_are_not_chain_colours` - a chain naming 951s is refused.
- `applied_action_replay_after_reload_is_refused` - a committed action cannot be
  replayed from a reloaded state.

## Findings

1. Duplicate doc-comment line above `apply_powerup_action` ("Applies one
   clickable-powerup action. The four effects mirror the originals" followed by
   the fuller line) - cosmetic leftover.
2. Shuffle placement parity gap: the original `StartShuffle` (1222-1245) and the
   client (`Grid.as` 1620-1695) run a same-value adjustment (`while value == next
   value` shifts the destination and `_loc6_[i+k] = _loc6_[i]`); the Rust
   `shuffle_tiles` performs only the plain previous-to-current rotation. The
   builder's test asserts counts, not placement, so this is unverified.
3. Queue refill boundary: `action_draw` refills when `len <= QUEUE_MIN` before
   consuming; the original consumes first and refills while `len < QUEUE_MIN`.
   The resulting lengths match, but the message ordering within the action frame
   is not byte-compared against a capture.
4. No captured clickable-action frames exist in the lab; every effect check is
   script-derived, so the selection contract is recovered while the exact
   balance values remain caller-authored (`Type8PowerupDefinition` is labelled
   RECONSTRUCTED, values 950-953 are ORIGINAL).

Until the builder freezes the revision and runs the focused sweep, treat the
clickable set as implemented-but-unfrozen; nothing here is a native claim.

## Final verification (frozen revision `038a135d64fe0c3b…`)

Frozen pins: `src/mining.rs` `038a135d64fe0c3be6a8e57eabbe002e7f8b651566b0ca260098175abf16959e`,
`tests/mining_powerups.rs` `ae2a7f657fd4a01e0cb2e727d949569262579e2faacdff5c2f4326f6ef5cb50c`,
implementer note `d3d2945e1cf398f701befae32f7955883c13502251dbe9973001ac3821b5ddb2`,
pre-edit baseline `da3b5b567bdcaf05…`. New private report:
`../lab/reports/mining-clickable-review-final-20261009T170936Z/` (`probe/`,
`probe.log`). The first review receipt above is preserved unchanged.

Independent runtime probes drive `Type8Engine::handle` (not helpers); all six
passed at the frozen hash:

| correction | probe | result |
| --- | --- | --- |
| no S2C `OnActionMsg` | `no_server_to_client_action_echo` | pass: no `OnActionMsg` in replies; the board still mutated |
| shatter down/up/right/left | `shatter_order_is_down_up_right_left` | pass: queue 0 accepts the down tile, queue 1 the up tile, the swap is refused |
| consume then `while len < 50` refill 20 | `queue_consumes_then_refills_below_min` | pass: 51 draws no refill, 50 and 49 draw one batch |
| pressed wild excluded, equal-value walk | `pressed_wild_is_excluded_from_shuffle_targets` | pass: an equal-value rotation is accepted; reusing the pressed tile is refused |
| implied counts from power/pool | `implied_counts_come_from_power_or_pool` | pass: a short list with a full pool is refused; a one-tile pool accepts the shorter list |
| default path parity | `default_percent_matches_pinned_da3b_baseline` | pass: replies and `Type8State` identical to `da3b` with `powerup_percent = 0` |

Verdict: all five corrections are independently verified at the frozen
revision, and the default path is byte-identical to the pinned pre-implementation
baseline. Remainder: no captured clickable-action frames exist, so the effect
semantics stay script-derived; queue message ordering inside a frame is not
capture-compared; balance values remain caller-authored; these probes exercise
the Rust engine only, not the native client.

## Bounded extra verification (queue PASS, shuffle placement RED)

Root review of the six probes above found two overclaims. Extra private probe
`../lab/reports/mining-clickable-review-final-20261009T170936Z/probe/tests/placement_queue.rs`
ran at the same pin `038a135d64fe0c3b…`:

- Queue loop: PASS. Lengths 1/20/50 consume first and refill 20 while `< 50`:
  3/2/1 batches, final lengths 60/59/69, and every batch value equals the
  xorshift sequence from `rng = 3309872643` in order.
- Shuffle placement: RED. The runtime does not implement the original walk
  (`ServerGrid.StartShuffle` 1166-1249: positions unshifted,
  `tilearray[(i+1)%n] = drawn[i]`, the `while` compares the static
  `aCurrentGrid` value at the mutable destination list starting `look = 0`,
  then `_loc6_[i+k] = _loc6_[i]`):
  - drawn `[401,402,403]` -> runtime `[402,401,403]`, oracle `[403,401,402]`;
  - drawn `[401,401,402]` -> runtime `[401,401,402]`, oracle `[402,401,401]`;
  pressed stays `200` and no `953` is recycled in both.
  The earlier `pressed_wild_is_excluded_from_shuffle_targets` probe proved only
  acceptance and the pressed exclusion, not placement; the final-verification
  table's equal-value-walk row is superseded by this RED until the builder
  repairs `shuffle_tiles`/`shuffle_placements`.

Oracle boards: `oracle-expected.json`; receipt: `extra-red-receipt.json`
(probe source `67e1bdee2317563dc5b3957693e579fb…`, oracle
`e1800b17e2465d7d0acea3f8ff958139…`, RED log `dbd30103ab960688095aac859da6e…`).

## Repaired freeze accepted (`4abe6b07f8edf527…`)

Frozen pins: `src/mining.rs` `4abe6b07f8edf52798ce810cde752a58fb959c30fcb639675371a041e463eb46`,
`tests/mining_powerups.rs` `507945e9d42d2568572c70bfa3f1c93b2c12ab825c7a56524502fce18359f40a`
(unchanged before and after the run). New private report:
`../lab/reports/mining-clickable-repair-verify-20261009T171910Z/` (`probe/tests/verify.rs`,
`verify.log`). The RED section and its sources/logs above are untouched.

- Shuffle placement: GREEN against the same direct AS2 oracle for three vectors
  — distinct `[401,402,403]`, two-equal `[401,401,402]` and all-equal
  `[401,401,401]`; pressed stays `200`, `953` is never recycled and the drawn
  values survive exactly once in every vector.
- Queue refill loop: unchanged and re-verified — lengths 1/20/50 produce 3/2/1
  batches, batch values match the xorshift sequence from `rng = 3309872643` in
  order, final queue lengths 60/59/69.
- Zero-default parity: replies and `Type8State` are identical to the pinned
  `da3b` pre-implementation copy with `powerup_percent = 0`.
- Cleanup safety: the new probe never preemptively deletes a computed path; it
  uses a unique `create_dir` (fails if the name exists) and a drop guard that
  canonicalizes the path and verifies it is inside `std::env::temp_dir()` with
  the expected prefix before `remove_dir_all`.

Verdict at this freeze: the shuffle placement RED is closed and the queue
semantics have an independent runtime proof. Remainder is unchanged: no captured
clickable-action frames, so effects stay script-derived; balance values remain
caller-authored; no native claim.
