# Kart reward registration

Status: implementation + focused tests complete, 2026-10-09 (UTC). Owned paths:
`prototype/osfr-tools/src/kart_rewards.rs` (new), `prototype/osfr-tools/tests/kart_rewards.rs`
(new), this note and `lab/reports/kart-rewards-20261009/`. No shared
`gameplay`/`gateway`/`lib`/`content`/`save` file, and no `kart_racing.rs`, `profiles.rs`,
`kart_entry.rs` or other worker's module, was modified. Skills: RTK (normal commands), Ponytail
(one registry, one staged commit, no new dependency), Caveman (terse reporting, complete persisted
notes), reverse-engineering (the reader anchors below), game-research (pinned notes first; no new
web work).

**No native acceptance is claimed.** No captured frame reaches a kart lap limit, so no finish and no
payout has ever been observed; the numbers here are reconstructed by construction.

## What was built

`kart_rewards.rs` is the payout seam between a finished `RaceSession`, the live character
`Inventory` and the live `Profiles`. Registration and payout are one small table plus one function.

| Item | Shape |
| --- | --- |
| `RewardRegistration` | `{ speedway, coins, xp, provenance }`, plus `reconstructed(speedway, coins, xp)` and `schedule()` so the session's `TrackLayout` is built from the registered price rather than a second copy |
| `KartRewardRegistry` | `new(rows)` (validates the whole table), `starter_defaults()`, `registration(speedway)`, `contains`, `rows()` |
| `claim_race_reward(&registry, &mut RaceSession, &mut Inventory, &mut Profiles, source_guid, player_guid)` | `Result<RewardPayout, RewardError>`: the only entry point |
| `RewardPayout` | `{ speedway, coins, xp, previous_xp, replies: Vec<Vec<u8>>, changed }`, where `replies` are inner packets in wire order |
| `RewardError` | `Unregistered`, `MissingKartDriverJob`, `Unpriced`, `ScheduleNotRegistered`, `EmptySchedule`, `DuplicateRegistration`, `ZeroSpeedway`, `AwardTooLarge`, `CoinOverflow`, `ExperienceOverflow`, `Racing(KartRacingError)`, `Inventory/Profiles/Toast(kind)` |

The three replies are the real thing, not a summary:

1. `Inventory::grant_coins` → the `38/19` coin update (skipped when the registered coins are zero);
2. `Profiles::add_experience` + `Profiles::experience_packets` → `38/18` on a rank-up and `38/14`
   (skipped when the registered XP is zero), always for the active job;
3. the recovered ordinary reward toast, family `50` subtype `1`.

## Evidence base

| Input | Use |
| --- | --- |
| `notes/Research/Kart Race Finish Protocol.md` | "No captured frame is at or above the lap limit", so the finish sequence and its payout are unobserved; `Finished` is client-derived from the server-sent lap limit. `SessionSnapshot.reward_claimed` is the only one-shot marker |
| `notes/Research/Reward Toast and Deferred Bank Findings.md` | reward family = base `50` with a **byte** subtype (`FUN_00b88ad0`, reader `0x00b89190`); subtype 1 = reward bundle; reader `0x008e7930` places coin award at inner offset 4, XP/stars at 8, reward kind at 16; an empty-entry inner packet is 72 bytes; kind 16 selects the item-notification panel |
| `src/mystery_chest.rs` | the compiled, natively accepted `50/1` writer for the same subtype: the toast body order, multiplier `1.0`, guide/name fields, zero entries and the appended final word are copied from it so the two writers cannot drift |
| `lab/decompiled/client/FreeRealms/pseudocode.c` | `FUN_008d86e0` (38/14) and `FUN_008d8910` (38/18) client readers, reused through `Profiles::experience_packets` |
| `src/gameplay.rs` (`apply_mining` → `grant_reward` → `Inventory::grant_bundle`) | the existing reward path grants through the real inventory and emits only the `38` coin/item updates; it does **not** emit an ordinary `50/1` toast, which is why this seam builds that packet explicitly |
| `src/kart_racing.rs` | `RaceSession::claim_reward` is the only grant authority (award derived from the session's own validated state); `SessionSnapshot`/`restore` are the supported clone/restart path |

## Authority chain (and what is reconstructed)

ORIGINAL: family/subtype framing and the `50/1` offsets; the `38/14`, `38/18` and `38/19` reply
family; the Kart Driver class id 48 (`kart_racing::KART_DRIVER_JOB_GUID`); `claim_reward`'s
finished-laps gate; the five speedway ids; the absence of a captured finish.

RECONSTRUCTED, and labelled in code: every starter number (`starter_defaults`: 50/25, 60/30, 75/35,
90/40, 100/50 coins/XP for speedways 31/32/33/34/39); the currency reward kind `0` (the note fixes
kind 16 for the item panel but does not name the currency value); the multiplier `1.0` for a
currency toast; the price-must-equal-the-registration rule; the job gate and every error string.

The payout order is a deliberate split of authority:

1. the speedway must be registered → `Unregistered`;
2. the active job must be the Kart Driver → `MissingKartDriverJob` (the seam never switches jobs and
   never writes XP to another class);
3. `RaceSession::claim_reward` runs on a **restored clone** of the session, so an unfinished race,
   a clock regression, a re-claim or an unpriced layout leaves the live session untouched;
4. the granted amount must equal the registered price → `ScheduleNotRegistered` (a session opened
   with an arbitrary price cannot mint coins);
5. coins and XP are granted on clones with explicit overflow checks → `CoinOverflow` /
   `ExperienceOverflow`;
6. the toast is built before anything commits;
7. only then are the session, the inventory and the profiles committed, all three by infallible
   assignment.

A refusal therefore leaves the session (`reward_claimed` included), the wallet and the class XP
exactly as they were.

## Exact lead hooks

1. **Finish.** When the lead's finish path marks a session finished, call
   `claim_race_reward(&registry, &mut session, &mut gameplay.inventory, &mut profiles, source_guid,
   character_guid)` and wrap each returned inner packet exactly like the other gameplay replies.
   `changed == true` means the canonical snapshot (which already carries `InventoryState` and
   `ProfileState`) must be saved before the replies are released.
2. **Registration.** `KartRewardRegistry::starter_defaults()` is the explicit reconstructed
   fallback; build the table from a package `content/` row set when one exists. Open a race with
   `TrackLayout::new(speedway, laps, checkpoints, Some(registration.schedule()))` so the session
   price and the registration agree by construction.
3. **Job gate.** The seam refuses to pay while another class is active. The lead should also deny a
   class change while a race is live, so a player cannot take the payout decision away from the
   Kart Driver; the reconstructed reward toast is sent by this seam rather than by the class
   progression replies.
4. **Restart.** Persist `SessionSnapshot` (already `Serialize`) with the rest of the gameplay state.
   A restored session whose `reward_claimed` is true refuses a second payout, as the restart test
   shows.

## Checks and limits

Run with `scripts/cargo.ps1`:

* `--test kart_rewards`: 11 passed, 0 failed (`green.log`). RED chain: `red-compile.log` (module
  missing) and `red-behavior.log` (4 failures with the first implementation: an unregistered price
  was payable, both overflows returned untyped inventory/profile errors instead of the typed
  refusal, and the staged rollback did not exist).
* `--test kart_rewards --test kart_racing --test kart_entry --test kart_lobby_native
  --test kart_progress_native --test kart_review --test profiles --test inventory`: 87 passed,
  5 ignored (private gates), 0 failed (`focused-neighbors.log`).

Limits: a furniture-style stack rule does not apply here, but no captured race ever finished, so
nothing in this seam is native-proven; the client-side XP adjustment helper (`0x00be8980`) noted in
the toast research is not modelled; the toast carries the registered XP verbatim. Reward numbers,
the currency reward kind and the "registered price only" rule stay reconstructed until finish
captures or server content exist. The player-side job gate and the finish packet order remain the
lead's integration work.

Cross-links: [[Kart Racing Restoration]], [[Kart Race Finish Protocol]],
[[Reward Toast and Deferred Bank Findings]], [[Mystery Chest Restoration]],
[[Kart Entry Implementation]].
