# Mining independent review

2026-10-08. **Both reviewed defects are repaired and green: the engine now accepts the recovered client's trailing-comma lists, and content validation matches the run-state caps. Native acceptance remains open.**

Independent review first reproduced two counterexamples against the mining worker's delivered revision; the lead then authorized a scoped repair of `src/mining.rs`, `tests/mining.rs` and `tests/mining_review.rs`. Nothing else was touched: no `lib.rs`, shared content file, economy, UI, launcher, manifest or release artifact.

## Repaired: native coordinate lists end with a comma

The recovered client builds every coordinate list by appending `x,y,` per tile: `core/Grid.as:998` (`_loc7_ += _loc2_.getPositionString() + ","`, chain), `core/Grid.as:520` (idle), `core/Grid.as:1079` (verify). The original server mirrors that contract by splitting and popping the empty tail (`GameLevel.as:500`, `GameLevel.as:550`). The delivered `coordinates` (then `src/mining.rs:1343`, now `:1361`) and `number_list` (then `:1363`, now `:1387`) parsed every comma field as a number, so a real `OnChainMsg\tx,y,...,` returned `invalid mining coordinate` before `validate_chain` ran; `OnIdleMsg` and `OnVerifyMsg` failed the same way.

Repair: both parsers strip exactly one trailing empty field (`strip_suffix(',')`, commented as the recovered client contract). Internal empty fields and a second trailing empty stay invalid. Regressions: `tests/mining.rs::native_client_coordinate_lists_tolerate_one_trailing_comma` (native chain, idle and verify accepted; internal/duplicate empties refused without state change), `tests/mining_review.rs::trailing_comma_is_tolerated_once_and_internal_empties_stay_invalid`, `native_client_chain_idle_and_verify_wire_text_is_accepted`, `native_client_verify_tail_and_non_numeric_fields_are_handled` (a full 14x14x3 zero-check verify payload with the native tail, plus `nan`/`inf`/`1.5`/`4097`/u16-overflow/invalid-boolean refusal).

## Repaired: specs that load but can never run

`validate_spec` (`src/mining.rs:810`) and the run validator disagreed. The repair shares two reconstructed caps: `MAX_BOARD_CELLS` (`:63`, 448 = `SIDE * 32`) and `MAX_REMAINING_MS` (`:67`, 600000), used by `validate_spec`, `validate_state` and the chain time bonus (`:554`), whose arithmetic now uses `saturating_sub`. Durations above 600 s, boards above 448 cells and shape codes the tile factory cannot execute are refused at load; the shape grammar accepts `0..=4` markers, positive category codes whose `floor(v/100)*100` is 400 or 600, and negated specific values `valid_value` accepts. The now-unreachable inner negative-range check was removed. The module doc lists the two caps as reconstructed, and the shape comment marks the recovered `ServerGrid.buildMap` grammar. Regressions: `tests/mining.rs::content_bounds_align_with_the_run_state_bounds`, `tests/mining_review.rs::unusable_specs_are_rejected_at_load`, `duration_ceiling_matches_the_run_state_cap` (600 s accepted, chain bonus bounded to zero at the cap, expiry still authoritative), `board_ceiling_matches_the_run_state_cap` (448-cell board plays and validates).

## Verified claims

Protocol provenance holds. `39` is `BasePackets.BaseMiniGamePacket` and sub 16/18/47 are the original server's `MiniGameInfoPacket`/`MiniGameGameOverPacket`/`MiniGameGameEndScorePacket` (`lab/decompiled/server/LibSOE/SOE/BaseMiniGamePackets.cs:9-17`); sub 17/19 are the pinned Sanctuary ack/removal packets. The header matches the pinned `BaseMiniGamePacket` layout, and the pinned client frames (`community-serializer-vectors.json`, Sanctuary writer with the native GameId=-1 sentinel) were replayed byte-for-byte against the engine and acked with game id 18 (`pinned_oracle_client_frames_drive_the_handshake`). Payload framing is `len+1` plus NUL in both directions, and `info_packet` (`src/mining.rs:703`) is a field-for-field port of the natively accepted forging packet (`src/forging.rs:582`) of the same engine family.

Chain authority is server-side. The engine reproduces `ServerGrid.checkCurrentTile`/`checkNextTile` (minimum 3, orthogonal, no repeats, one selectable value, wildcard adoption), `CheckGrid` zero-skip verification and `FallOffBottom`'s bottom-row/dropzone check; client `FRServer_ScoreSet/GameWon/GameLost/GameQuit` text cannot move score, quota, phase or transaction (`client_reported_score_and_win_texts_never_change_authoritative_state`). Event ids, score constants and the inverted pause notification match `GameNetworkEvent.as`, `GameData.as` and `SoeClient.as:590`.

Queue and session bounds hold. Queue discipline 50/20/70 and random range 0..999 match `ServerGrid` constants; ignoring the client's 60/61 accept/reject is consistent because the engine never sends `reset >= 0`, the only case the client answers (`GameClient.as:170-196`). A scripted 8-chain run keeps the tile queue at or below 70, the board at 196 cells and the state passing `validate_state`; tampered queues above 140 and boards above 448 cells are refused (`tampered_state_outside_the_declared_bounds_is_rejected`). Reconnect drops the run, emits 39/19, allocates a fresh instance and ignores stale-instance messages (`reconnect_cancel_drops_the_run_and_stale_instance_messages_are_ignored`).

No historical reward is invented. `mining_spec()` ships `outputs: []` (`src/mining.rs:797`) and `reward_transaction` returns `None` without a mapping (`src/mining.rs:1220`); completing every default goal yields `completed: true` with `transaction: None` (`default_spec_completes_without_granting_an_unrecovered_reward`). The only granted item in either suite is the synthetic fixture inside `tests/mining.rs`.

Malformed input handling is safe. Truncated headers, a wrong request length, an inflated payload length field, control characters, coordinates above 4096, odd coordinate counts, a u16 overflow in a verify value, unknown payload text, another minigame family, an unknown sub-op and a wrong instance are each refused or ignored without mutating state (`malformed_payloads_are_rejected_without_mutating_state`).

## Non-blocking deviations from the recovered original

Structural differences, none fabricating a historical value: the chain timer bonus is capped at 600 s absolute where `CalculateChainTimeBonus` is uncapped 2 s per tile (`GameLevel.as:553`); the collectable score is not doubled once quota is reached (`GameLevel.as:637`); `LimitToCollectLimit` maximums are not modelled (spawn data unrecovered); `Type8Goal.index` is parsed but the wire index is the goal's vector position; an invalid fall returns `Err` where the original stops the game with `FallOffBottomFail` (`GameLevel.as:464-467`), leaving that policy to the caller; `OnLevelInitMsg` level/type are hardcoded `1`/`3` because the level tables are unrecovered.

## Remaining native gates

`mining` is still not registered in `lib.rs` and nothing calls `Type8Engine`, so no shipping path exists yet; the research note's contract stands (register the module, add an optional snapshot field, ship `mining.json`, call `begin` only after the accepted Miner checks, `cancel()` on reconnect, `tick()` for the timer). Native acceptance was not attempted. The parser repair unblocks the first native drag, but unverified work remains: client entry/tutorial, drag chains against the mirror, queue/gravity desync after falls, ore bins and cluster display, `OnStopLevel`/`OnNextLevelMsg`/`OnGameWonMsg` timing, results close, and the item grant once a mapping is recovered or accepted.

## Evidence

- `lab/reports/mining-review-repair-20261009T031715Z/red-before-fix.log` — both prior counterexamples reproduced failing before the repair (`invalid mining coordinate`; duration 601 accepted at load).
- `lab/reports/mining-review-repair-20261009T031715Z/green-after-fix-mining.log` — `cargo test --test mining`: 19 passed, 0 failed, 0 ignored.
- `lab/reports/mining-review-repair-20261009T031715Z/green-after-fix-mining-review.log` — `cargo test --test mining_review`: 13 passed, 0 failed, 0 ignored (no ignored tests remain).
- `lab/reports/mining-review-repair-20261009T031715Z/green-after-fix-harvesting.log` — `cargo test --test harvesting`: 17 passed, 0 failed, 0 ignored.
- `lab/reports/mining-review-repair-20261009T031715Z/artifact-hashes.json` — final hashes of the repaired source, both test files, this note and every log.
- `lab/reports/mining-review-20261009T025458Z/evidence-index.json` — decompiled client/server line quotes behind each finding.
- `rustfmt --edition 2024 --check src/mining.rs tests/mining.rs tests/mining_review.rs` exits 0; formatting was applied only to the three owned files.

Environment note: `scripts/cargo.ps1` is blocked by this host's execution policy, and `powershell -ExecutionPolicy Bypass` promotes cargo's stderr progress to a terminating `NativeCommandError`. Runs above set `CARGO_HOME`/`RUSTUP_HOME` from `../lab/tools` and invoke `lab/tools/cargo/bin/cargo.exe` directly with the same effect.

This review is the independent gate only. Green focused tests do not establish playable native Mining; the lead owns native acceptance and any further integration.
