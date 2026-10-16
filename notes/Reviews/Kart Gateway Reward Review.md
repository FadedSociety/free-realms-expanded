# Kart gateway reward review

Read-only review, 2026-10-09 (UTC). Owned paths: this note and
`lab/reports/kart-gateway-reward-review-20261009/`. No source, test, content, save, UI, build,
release or process file was touched, and no build or test was executed here: the instruction was
read-only while the root cargo gate runs. Every claim below is either a code citation or a citation
of an existing lab artifact. Skills applied: RTK (normal commands), Ponytail (scope discipline),
Caveman (terse findings), reverse-engineering (reader/writer anchors only, no new decompilation).

Reviewed revisions are pinned by SHA-256 in `reviewed-revisions.txt`. Two files changed while this
review was running: `src/kart_gateway.rs` grew to 16,662 bytes, and `tests/kart_gameplay.rs` was
reformatted and then gained a native-Close (`39/7`) block that returns to the world without a second
award (`tests/kart_gameplay.rs:285-292`). Every citation below was re-verified against the final
revision; the hashes in the receipt, not the byte counts in this prose, are the record.
`src/kart_rewards.rs` is byte-identical to the reviewed delivery (`b74e0a6b...`).

## Findings

### 1. HIGH - a refused reward tears the client session down instead of answering the finish

`KartGateway::claim_reward` maps every `RewardError` to `io::Error` (`src/kart_gateway.rs:392-395`),
and both callers propagate it with `?` (`src/gameplay.rs:943` for the finish frame, `src/gameplay.rs:702`
for the per-tick retry). `Gateway::handle_at` returns that error, and the server binary answers any
`Err` by saving the pre-error snapshot and dropping the session
(`src/bin/osfr-gateway.rs:325-327`, and `:344-348` for the tick path).

The reachable refusals are all *legitimate* client input with a server-side refusal, not malformed
wire data: `MissingKartDriverJob`, `ScheduleNotRegistered`, `Unpriced`, `CoinOverflow`,
`ExperienceOverflow`, `Racing(RewardAlreadyClaimed | NotFinished | ClockRegression)`
(`src/kart_rewards.rs:246-306`). So a finish that is refused - a package price that differs from the
hardcoded registry, a wallet at the coin ceiling, an XP value near `u32::MAX`, a class change that
somehow slipped through - disconnects the player mid-finish, discards the finish progress frame
(`gameplay.rs:940-944` assigns `out.replies` before the `?`), and leaves the offer pending. The
binary drops the session on that first refusal; a caller that catches the error instead (as the
private test does) sees the same refusal on every later tick, so the offer can never settle.

Suggested fix (either shape, both non-fatal): return `Ok` with the finish progress reply and no
payout, or answer a typed denial, and keep `finish_flag()` set so a later attempt can pay. Malformed
frames should keep their current fatal treatment.

### 2. HIGH - the payout registry is a second hardcoded copy of the content price

`src/kart_gateway.rs:386` passes `KartRewardRegistry::starter_defaults()`, while the session's own
price comes from the package: `KartService::validate_content` builds `TrackLayout::new(..., content.reward)`
(`src/kart_service.rs:549-555`), and `claim_race_reward` refuses any session whose granted amount
differs from the registry (`src/kart_rewards.rs:263-271`).

The package diff is decisive: `lab/reports/kart-playable-20261009/content/kart_entry.json` has **no
`race` field at all**, and `lab/reports/kart-playable-v2-20261009/content/kart_entry.json` adds
`race.speedway=31, laps=3, reward{coins:50, xp:25}` plus 31 reconstructed `radius: 3.0` gates. The
same 50/25 is hardcoded in `starter_defaults` (`src/kart_rewards.rs:121-137`). Nothing ties the two
together, and the only thing keeping them equal today is that both were authored from the same
number: change the package price (or add speedway 32/33/34/39 with their own numbers) and every
finish fails closed into finding 1.

Suggested fix: derive one `RewardRegistration` per speedway from the package (`KartServiceContent`
already carries `speedway` + `reward`), or have the gateway read the schedule back out of the
finished session and use it as the registration. Keep the built-in table only as the
labelled fallback for content that carries no price.

### 3. MEDIUM - race scheduling is gated behind the opening campaign content

`Gateway::tick_at` returns early unless `Gameplay::is_opening()`
(`src/gateway.rs:742-744`), and the countdown schedule that actually starts a race is emitted only by
the kart branch of `Gameplay::tick` (`src/gameplay.rs:696-704`). `KartService::tick` is where the
schedule advances, `start_race` runs, and the `max_race_ms` window is enforced
(`src/kart_service.rs:759-820`).

The private package works only because it also ships `opening.json` (51,602 bytes in
`lab/reports/kart-playable-v2-20261009/content/`). A kart-only content package would never leave
`Countdown`: no gate claim is accepted, no race times out, and the finish never exists. This is an
integration coupling, not a kart-module defect, but it is undocumented and it will bite the first
lean package. Cheap fix: `if !game.is_opening() && !game.kart_active() { return Ok(default) }` with a
small `kart_active()` accessor.

### 4. LOW/MEDIUM - the restored snapshot never cross-checks the two one-shot markers

`KartService::restore` rejects only `reward_taken && phase != Finished`
(`src/kart_service.rs:990-994`). It accepts `session.reward_claimed == true` with
`reward_taken == false`, which leaves `finish_flag()` true forever
(`src/kart_service.rs:677-689`): every claim attempt then fails with `RewardAlreadyClaimed` and, via
finding 1, drops the session. The mirror (`reward_taken == true`, `reward_claimed == false`) loads
and silently never pays. Neither is reachable through the normal flow (the staged commit writes both
together), so this is robustness against a edited/corrupt save, not a live bug. Fix: require the two
markers to agree whenever the layout is priced, and reject `reward_taken` on an unpriced layout.

### 5. LOW - the gateway reward test does not pin the amounts it claims to verify

`tests/kart_gameplay.rs:260-278` asserts the wallet total, the XP total, and the *presence* of
`[50,0,1]`, `[38,0,19,0]` and `[38,0,14,0]`. It never reads the toast's coin/XP fields (inner offsets
4 and 8), never checks the `38/19` amount, and never checks the XP delta carried by `38/14`. The
new native-Close block adds a valuable return-to-world check, but it still reads no payout amounts.
The rank-up packet `38/18` has no gateway coverage at all: `full-gateway-race-bounded.log` shows the
earlier assertion
`final_replies.iter().any(|r| r[7..].starts_with(&[38,0,18,0]))` failing, and the current file simply
omits it instead of adding a rank-crossing fixture.

Source coherence holds by inspection: `kart_rewards.rs` builds the coin reply, the XP packets and the
toast from the same amounts it verified against the registration, in that order
(`src/kart_rewards.rs:279-303`), and the toast is the recovered 72-byte `50/1` empty-entry body with
coins at inner 4, XP at 8 and kind 0 at 16 (`src/kart_rewards.rs:196-230`). The gap is test strength,
not a known mis-payment. Suggested additions: assert the toast fields and the `38/14` XP delta, and
run one race whose starting Kart Driver XP sits just below a rank boundary so `38/18` must precede
`38/14`.

## Verified by inspection (no findings)

* **Whole-bundle atomicity.** `application_messages` decodes every child before any processing and
  refuses empty, nested, over-long and >128-child bundles
  (`src/gateway.rs:386-419`); `handle_at` decodes all children, then processes them against cloned
  character/gameplay state and assigns only after the loop (`src/gateway.rs:580-585`, `:727-728`).
  `tests/kart_gameplay.rs:226-236` sends a two-child `00 25` bundle whose first child is the final
  gate (payout) and whose second is a malformed `33/9`, asserts `handle_at` errors, and asserts the
  snapshot is byte-identical - the strongest rollback shape. `tests/gateway.rs:313-341` covers
  malformed-child rollback generally, and `tests/gateway.rs:350-380` wraps a real five-child bundle
  (login + `13` + `10` + two extended-length children) plus the `0x0019` prefix variants. Five-child
  *rollback* (children 1-4 committed, child 5 failing) is not separately tested; it follows from the
  same single-loop staging but is an inference, not evidence.
* **Duplicate finish.** A replayed finish frame is refused by the phase guard before any counter or
  state moves (`src/kart_service.rs:1115-1125`), `finish_flag()` is false once `reward_taken` is set
  (`src/kart_service.rs:677-689`), and the test asserts the replay yields no replies and an identical
  snapshot (`tests/kart_gameplay.rs:281-284`).
* **Class guard.** Entry unlocks and selects the Kart Driver (48) for the opt-in campaign
  (`src/gameplay.rs:913-914`), a `26/13` class change while a race is active is answered with a chat
  refusal and no state move (`src/gameplay.rs:950-956`), and the payout itself refuses any other
  active job (`src/kart_rewards.rs:246-252`). The test asserts the active class is still 48 after the
  attempt (`tests/kart_gameplay.rs:120-130`).
* **Restart / load-save.** `GameplayState.kart_entry` is an optional serde field
  (`src/gameplay.rs:78-79`); `KartGateway::load` re-verifies the content hash, the active-entry
  ownership and `instance_id + 1 == next_instance` before restoring the race
  (`src/kart_gateway.rs:136-176`); the test round-trips through `game_store::save` /
  `load_or_create` and asserts both banks survive and `reward_taken` is true
  (`tests/kart_gameplay.rs:287-306`).
* **Reconnect world pose.** `abandon_kart_for_reconnect` runs on load and on session reset
  (`src/gateway.rs:478`, `:569`), returning the pre-race `return_location`; the test asserts
  `reconnect.state() == before.character` and `kart_entry.active == None`
  (`tests/kart_gameplay.rs:74-84`).
* **Reward toast coherence.** The toast is emitted from the same amounts that were verified against
  the registration and actually granted, after the progress frame and the `38/19`/`38/18`/`38/14`
  economy replies (`src/gameplay.rs:940-944`, `src/kart_rewards.rs:279-303`). The two `u64`s are the
  race's `player_guid`, which the entry binds to the character guid, so source and player agree by
  construction (`src/kart_gateway.rs:384-391`).
* **Reconstructed geometry, not native.** "RECONSTRUCTED content: one ordered gate volume. Original
  gate geometry is unrecovered" (`src/kart_service.rs:96-98`), `TrackLayout` repeats the same label,
  and the package proof states `historical_geometry_recovered: false` and
  `native_finish_verified: false` with the change list "reconstructed track 31 reward 50 coins and 25
  Racer XP" and "reconstructed 3m gate volumes" (`lab/reports/kart-playable-v2-20261009/package-proof.json`).
  `Kart Race Finish Protocol.md` still records that no captured frame reaches the lap limit. Nothing
  here is native acceptance.

## Root claim checked

`lab/reports/kart-playable-v2-20261009/full-gateway-race-correct-packets.log` shows
`tests/kart_gameplay.rs` at 2 passed / 0 failed (both tests are `#[ignore]`-gated on
`OSFR_KART_PLAYABLE`, so they were run explicitly with the private package). That log is taken as
given evidence of the root's run; this review re-executed nothing. The staging claim
("claim stages own race+inventory+profiles") is confirmed in source: `KartGateway::claim_reward`
clones the gateway, the inventory and the profiles and commits all three only after the payout
succeeds (`src/kart_gateway.rs:369-397`), and the caller already runs on a staged `Gameplay`
(`src/gameplay.rs:782-789`).

## Limits

Read-only, no builds or tests executed, no UI, no native run. Findings 1-3 are failure-mode
analyses grounded in the cited lines; whether the lead wants F1/F2 fixed before the next native
attempt is a scope decision. `src/kart_rewards.rs` was reviewed as delivered; the other kart
modules moved during the review, so anything edited after the pinned hashes is not covered.

Cross-links: [[Kart Reward Registration]], [[Kart Race Finish Protocol]],
[[Reward Toast and Deferred Bank Findings]], [[Kart Entry Implementation]],
[[../Research/Kart Racing Restoration]].
