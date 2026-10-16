# Soccer match authority review

Read-only independent review of the new MMO005 module `src/soccer_match.rs`
(`2B6055EF…`) and `tests/soccer_match.rs` (`DC18E538…`) from `soccer_review`, 2026-10-09.
Reviewer owns this note and `lab/reports/soccer-match-review-20261009T0635Z/` only; no
production, test, gateway, save, lib, UI or release file was touched and no process or window was
driven. Skills: RTK (normal commands), Ponytail (private probe reuses the real modules instead of
re-implementing them), Caveman (terse report), reverse-engineering (re-read the capture decode
evidence for the client-id directions).

Verdict: **the candidate is a coherent bounded authority slice with correct goal, clock, config and
restore handling, and it does not invent historical physics constants. It is not native-ready. Five
concrete authority gaps remain, all reachable without a client: identity is not bound to the sender,
one spoofed intent can lock a player out, movement is unbounded in wall-clock time, an out-of-play
ball is a dead end until the half ends, and two packet ids the capture shows the client sending are
refused with a wrong label.** The reported 17/17 candidate tests were re-run and pass; they do not
cover the findings below.

## Findings

**F1 — High: the authority cannot bind an intent to its sender.** `client_intent` copies the
packet's own `player` field into the intent (`src/soccer_match.rs:242-251`) and `apply` checks only
that the player is registered (`447-450`). Probe `probe_other_player_position_packet_moves_that_player`:
a packet claiming `0x11` moved `0x11` to `[3, 0, 4]` while `0x10` was untouched. Any caller that
passes raw client bytes into `client_intent`+`apply` hands every client control of every other
client's player. Lead action: bind the intent to the authenticated session guid before `apply`
(compare/replace with the sender), or add a checked `client_intent_for(bytes, sequence,
expected_player)` variant. The module alone cannot fix this; the integration must.

**F2 — High: one spoofed control packet locks a player out for the match.** The `Controls` branch
stores the client-supplied sequence (`482-496`, assignment at `492-495`) and `apply` refuses anything
not strictly greater (`451-453`). Probe `probe_spoofed_controls_sequence_locks_the_victim_out`: a
single controls intent for `0x11` with sequence `u64::MAX` made that player's own next intent fail
with `stale soccer intent`. With F1 fixed this is unreachable; without it, it is a permanent denial
of service with no ceiling or server-side counter. Lead action: either rely on the identity binding
for this path, or cap accepted sequences to a server-owned window.

**F3 — Medium: movement is bounded per intent but not per unit of time.** `Position` accepts any
step up to `max_position_step` (`456-481`) and `apply` takes no timestamp at all (`441`). Probe
`probe_position_step_has_no_time_bound`: nine accepted intents moved the player 90 units with match
`elapsed() == 0.0`; the only rejected case was an 11-unit single step. A client can therefore cross
the pitch at reliable-layer speed while every intent is individually "bounded". Lead action: pass a
server arrival timestamp (or the reliable layer's clock) into the intent path and cap displacement
per second on top of `max_position_step`.

**F4 — Medium: out of play is a dead ball with no restart path.** `advance` zeroes velocity and
returns as soon as the next position leaves the field (`661-667`); the automatic possession scan at
`680-696` is unreachable for that ball, `kickoff` is refused outside `Kickoff`/`Restart` (`417-425`),
and `reset_ball` is only reachable through those two calls (`720-727`). Probe
`probe_out_of_play_ball_is_stranded_until_the_half_ends`: with `half_x = 10`, the ball stopped at
`[15, 0, 0]` with velocity zero while the phase stayed `Running`; the field edge at `x = 10` is five
units from the ball (control radius 2), a kick was refused for want of possession, `kickoff` errored,
and the only recovery was the half clock reaching `HalfTime`. Every over-the-line miss strands the
match. Lead action: define a bounded out-of-play restart (throw-in/corner/goal kick, or at minimum a
kickoff reset with possession to the non-touching team) instead of a frozen ball.

**F5 — Medium: two captured client ids are refused, one with a wrong label.** `client_intent` refuses
`RegisterPlayer` as "server-assigned" (`253-255`) and lets id 14 fall into
"packet id is not a captured soccer client request" (`259`), but the reviewed decoder itself pins
`CLIENT_PACKET_IDS = [1, 3, 14, 16]` (`src/soccer.rs:70-73`) and the capture summary counts 8 id-3 and
250 id-14 messages on client flow 1 (see `capture-id-evidence.txt`). Refusing client self-registration
is a defensible harden; calling id 14 uncaptured is factually wrong, and id 14 carries a position
triple that the authority currently ignores. Lead action: decide explicitly — map id 14's bounded
position like id 1 (or document the narrowing) and fix the error text so the integration is not
misled about the captured direction set.

**F6 — Low: no half-time side switch.** The attack convention is fixed at +z for team 0 and -z for
team 1 for the whole match (`61-62`, `755-781`); `start_second_half` resets the clock and ball but
not the sides (`428-438`). Probe `probe_second_half_keeps_the_same_attack_direction` scored into the
same +z goal in half 2. The capture never shows half 2, so this is unproven either way and the
research note does not list it as a gap. Lead action: implement the swap or label the fixed
convention as a known deviation before any playable claim.

## Verified clean

* Ground clamp and goal crossing in one step: probe `probe_goal_counts_after_the_ground_clamp` shot
  downward over the line, `y` clamped to `0.0`, goal counted `[1, 0]` (`631-634` before `651-659`).
* Clock first: a half can never end with a crossing in flight; probe
  `probe_half_end_preempts_a_crossing` ended at `HalfTime` with the ball unmoved at `[0, 0, 9.4]` and
  score `[0, 0]` (`590-606`).
* Malformed caller configs fail closed: NaN/negative gravity, infinite `max_tick`, zero
  `pass_radius`, NaN `half_seconds`, below-ground and NaN kickoff spawns, infinite `half_z`, NaN
  `goal_height` and a goal wider than the pitch are all refused; `goal_half_width == half_x` is
  accepted at the boundary (`72-88`, `150-176`, probe `probe_malformed_and_extreme_configs`).
* Terminal state is once-only and cannot be rewound: probe `probe_finished_match_cannot_be_rewound`
  restored a finished snapshot into a fresh authority (same `Outcome`) and refused an earlier
  snapshot as `stale soccer snapshot` (`593-605`, `709-718`).
* Kick/pass guards behave as documented: possession, proximity, power range, registered teammate,
  verified target position and `pass_radius` (`497-563`) are covered by the candidate's own tests,
  which all pass.

## Evidence

| Artifact | SHA-256 |
| --- | --- |
| `prototype/osfr-tools/src/soccer_match.rs` | `2B6055EFC44EF866642504FF69A52EAFB442C759EEC852E5E95A398869FD4BBF` |
| `prototype/osfr-tools/tests/soccer_match.rs` | `DC18E5386A6C8859513DAB98B1DC9F336DCFD413DF24A1D0D6F290CEB7DE6BB8` |
| `prototype/osfr-tools/src/soccer.rs` (evidence base) | `2C18C712A12B9FC003C0529A7146C8185F6CA80FFF19FA24273D3E654B27E2D9` |
| `lab/reports/soccer-match-review-20261009T0635Z/probe.rs` | `3A334F0413E1418485CBF9A7A2C0B9711D6E905F20C8B8E00F1A7ABF4A0B9EB1` |
| `lab/reports/soccer-match-review-20261009T0635Z/probe-run.log` | `3F7F6C466BC9EB1C1DD0B01377EA7E12D90074BE3358E3D99836D40E4EE7C388` |
| `lab/reports/soccer-match-review-20261009T0635Z/scoped-tests.log` | `7542FD397E23EA4598B8C8837579DB29494873BEF55592E2010F037BCCB8E501` |
| `lab/reports/soccer-match-review-20261009T0635Z/capture-id-evidence.txt` | `D49EC11878B8B5CC3E837B442B2705D9EE998E7EDE457B0033ACEBA007CA6995` |

Commands and results:

```
rustc --edition 2024 --test probe.rs -o probe.exe        # private toolchain, exit 0
probe.exe --nocapture --test-threads=1                   # 10 passed; 0 failed
scripts/cargo.ps1 test --test soccer_match --test soccer --test soccer_review
    # soccer_match 17 passed; soccer 11 passed; soccer_review 10 passed; 0 failed
```

The private probe includes both real modules by absolute path; it is not part of the repository and
changes no source. Two probe expectations were corrected during the review (both had used the wrong
`half_z`); the final log is the one hashed above.

## Boundaries

Nothing here is native acceptance. The module is not registered in `lib.rs`, the client kick/pass
request (sub 13) is still uncaptured, no pitch dimension exists in the capture, and the physics is a
reconstructed deterministic slice with no terrain, collision, spin or bounce. Possession, pass
completion, the out-of-play rule and the attack convention remain server-side inventions that need
the lead's decisions from F1-F6 before integration; the reviewed candidate itself is internally
consistent and its 17 vectors are meaningful.

Cross-links: [[Research/Soccer Match Authority]], [[Soccer Independent Review]],
[[Design/MMO Worker Contract]], [[MMO Work Queue]].

## Repair recheck (revision 326db82a, 2026-10-09)

The builder repaired the module and the findings above were re-verified with an adapted private
probe against the new `apply(sender, now, intent)` / monotonic-clock API, recompiled from the
reviewed sources by absolute path (no repo file touched). Reviewed revision:
`src/soccer_match.rs` `326DB82A…`, `tests/soccer_match.rs` `6CEA0461…`, evidence base
`src/soccer.rs` `2C18C712…` (unchanged). **F1-F6 are repaired**: the private probe passes 10/10
(`soccer-match-review-recheck-20261009T0700Z/probe-run.log`), and the candidate's own suite grew to
23 passing vectors.

* F1 repaired — `apply` refuses `sender != intent.player()` before touching any player state
  (`515-527`). Probe: a packet claiming `0x11` with sender `0x10` is refused, `0x11` keeps
  `position: None` and `sequence: 0`, and the same intent from sender `0x11` lands.
* F2 repaired — the same guard removes the spoofed-sequence lockout. Probe: a controls intent with
  `u64::MAX` from another sender is refused and the victim's own next intent (sequence 1) applies.
  A player can still self-advance their own sequence; only they are affected.
* F3 repaired — movement now costs authoritative server time
  (`max_speed * (now - position_at) + 1e-4`, `560-575`). Probe: a 10-unit step 10 ms later is
  refused with `exceeds the server time budget`; the same step after 0.25 s is accepted.
* F4 repaired — out of play resets the ball to the kickoff spawn and enters `Restart` (`750-758`
  region). Probe: ball at the spawn, velocity zero, phase `Restart`, `kickoff()` resumes and the
  ball is re-acquired on the next tick.
* F5 repaired — `client_packet` returns captured ids 3 and 14 as `ReadOnly` with accurate reasons
  (`258-313`). Probe: both decode `ReadOnly`; `client_intent` refuses with
  `captured client registration is read-only` and `captured client player state is read-only`, and
  the false `not a captured soccer client request` label is gone.
* F6 repaired — `HalfPolicy` (default `SwitchSides`) plus `attack_sign` (`42-51`, `854-861`). Probe:
  in half 2 a -z shot scores `[1, 0]` under `SwitchSides` and credits team 1 `[0, 1]` under
  `KeepSides`.

### New gaps found by the recheck

* **N1 (Low): the time-budget epsilon accumulates under equal-`now` spam.** Budget
  `max_speed * (now - position_at) + 1e-4` is per intent, so 1000 intents at the same `now`
  (`now == server_time`) moved a player 0.0496 units with `server_time` unchanged (probe
  `recheck_f3_time_budget_applies_and_equal_now_spam_still_accumulates`). The speed cap is therefore
  bypassed by roughly 1e-4 units per accepted packet at an unchanged clock. Recommendation: drop the
  additive epsilon (or use only a relative float-noise tolerance) and re-base the budget on the last
  accepted server time.
* **N2 (Medium): a forward `now` spike poisons the shared clock.** Probe
  `recheck_new_forward_clock_spike_poisons_the_match_clock`: `apply(now = 1000.0)` is accepted
  (`server_time` becomes 1000), `advance(0.25)`, then a real `now = 2.0` intent from the spiking
  player *and* from an unrelated player is refused with `soccer intent time moved backwards`.
  Probe `recheck_new_restored_clock_domain_locks_a_fresh_clock`: a snapshot taken after the spike
  restores `server_time = 1000`, and a reconnect whose monotonic base restarted at 1.0 gets
  `time moved backwards` for every intent until the new clock exceeds the old value. A gateway that
  passes event time, or a clock that jumps after suspend/resume or a process restart, can therefore
  block all intents. Recommendation: clamp accepted `now` to `server_time + max_tick` (or treat a
  larger jump as an implicit advance), and normalise the persisted clock domain on restore.
* **N3 (Low, note only): `restore` still validates only configuration and revision** (`806-816`);
  `position_at` and `server_time` are adopted verbatim. That is unreachable corruption today —
  `Authority` has private fields, no serde path, and `apply` only stores finite `now >= server_time`
  — but any future save/deserialization format must validate finiteness and clock monotonicity
  before `restore`.

Re-verified clean in the same probe run: ground clamp plus one-step goal crossing counts `[1, 0]`
(`recheck_verified_clean_core_behaviour`); a half end preempts a crossing and leaves the ball
unmoved; `max_speed` 0/NaN are refused by config validation; terminal state is once-only and an
older snapshot cannot rewind it (`recheck_verified_clean_terminal_state`). Scoped suites on this
revision: `soccer_match` 23 passed, `soccer` 11 passed, `soccer_review` 10 passed, 0 failed
(`scoped-tests.log`).

| Recheck artifact | SHA-256 |
| --- | --- |
| `src/soccer_match.rs` (revision 326db82a) | `326DB82A091E77136913B98AF8FA55F9975774F0D8F9C33EEDC5233730E72B19` |
| `tests/soccer_match.rs` | `6CEA046131A42BE98BC9943A1F1D02622AFB02BA850598A6C1C70263FFAAC248` |
| `lab/reports/soccer-match-review-recheck-20261009T0700Z/probe.rs` | `6399ADC51113F6D13C4A634435ACA4C0E7F275C5A2272E0E00B17BE2F9E450A8` |
| `lab/reports/soccer-match-review-recheck-20261009T0700Z/probe-run.log` | `1E0488BC9B26EA83FBA45E96501C1424CC49E861A6763B3141FAFC4B5AD31432` |
| `lab/reports/soccer-match-review-recheck-20261009T0700Z/scoped-tests.log` | `6CD3568DEC8CE6FA56AA57A7F8FDFD3FBE88554B509CEE04D5BFD675FBF90FA0` |

```
rustc --edition 2024 --test probe.rs -o probe.exe     # private toolchain, exit 0
probe.exe --nocapture --test-threads=1                # 10 passed; 0 failed
scripts/cargo.ps1 test --test soccer_match --test soccer --test soccer_review
    # soccer_match 23 passed; soccer 11 passed; soccer_review 10 passed; 0 failed
```

Still missing and explicitly untouched: the client kick/pass request (sub 13) layout, any original
pitch dimension, native client acceptance of any of this, and a real save format that would exercise
N3. All three recheck artifacts are private to `lab/reports/soccer-match-review-recheck-20261009T0700Z/`.

## N1-N3 focused recheck (revision a1406892, 2026-10-09)

Focused re-verification of the three findings the previous recheck raised, against revision
`src/soccer_match.rs` `A1406892…`, `tests/soccer_match.rs` `6CF10153…` and evidence base
`src/soccer.rs` `2C18C712…` (unchanged). Private probe:
`lab/reports/soccer-match-review-n123-20261009T0745Z/` (3/3 passed). Contract confirmed first:
`advance(dt)` is the only thing besides `restore` that moves the authority clock, and `apply(now)`
clamps `now` to the current `server_time` and never advances it.

* **N1 fixed** — the additive budget epsilon is gone
  (`max_speed * (trusted_now - position_at)`, no epsilon). Probe
  `n1_equal_authority_time_allows_zero_movement_only`: at equal authority time, displacements 1e-6,
  1e-3 and 1.0 are all refused with `soccer position exceeds the server time budget`; 1000
  equal-time 1e-6 intents are refused, the position stays `[0, 0, 0]` and `server_time` stays 0.0; a
  zero-length intent at the same time is still accepted; after `advance(0.001)` the same 1e-6 step
  is accepted. Equal-time spam can no longer move a player.
* **N2 fixed** — a forward timestamp spike grants nothing and poisons nothing. Probe
  `n2_forward_spike_cannot_advance_or_poison_the_clock`: a `now = 1000.0` intent with a 1-unit step
  is refused (clamped to the clock, budget 0) and `server_time` stays 0.0; an unrelated player's
  intent at the spiked time is handled normally; after `advance(0.25)` a 10-unit step lands exactly
  once and a repeat at the same trusted time is refused; NaN, infinite and negative intent times are
  refused. The old `time moved backwards` poison path is gone.
* **N3 fixed (observable part)** — `restore` rebases the saved clock domain onto the receiver and
  keeps each player's elapsed allowance. Probe
  `n3_restore_normalises_the_clock_domain_and_keeps_allowances`: a snapshot at clock 1.0 restored
  into a fresh receiver yields `server_time = 0.0` and `position_at = 0.0`, and 0.25 s of authority
  time then moves the player 10 units; a receiver whose clock is ahead (1.25 with an older revision)
  gets `server_time = 1.25`, `position_at = 1.25` and the same allowance; a stale snapshot and a
  replayed intent are both refused after the rebase. `restore` now runs a fail-closed
  `validate_snapshot` (`src/soccer_match.rs:841-890`: clock/elapsed finite and non-negative, ball and
  player positions finite, `position_at <= server_time`). That validation is still unreachable
  through the public API — `Authority` keeps private fields and has no serde path — so it stays
  code-verified until a real save format exists; it is the correct hook for that format.

Candidate suites on this revision: `soccer_match` 27 passed, `soccer` 11 passed, `soccer_review` 10
passed, 0 failed (`scoped-tests.log`). Native gates remain explicit and unchanged: the sub-13
kick/pass request layout is still uncaptured, no original pitch dimension exists, and no native
client has accepted any of this.

| N1-N3 artifact | SHA-256 |
| --- | --- |
| `src/soccer_match.rs` (revision a1406892) | `A1406892DF98523BECD17ACE6E4E0B8E4F8EE054A5E71126E604A990A022BD50` |
| `tests/soccer_match.rs` | `6CF10153530B9F06E7154FA35ED143BA07ED20C1720A82C85BD23F0501B59931` |
| `lab/reports/soccer-match-review-n123-20261009T0745Z/probe.rs` | `6ABA6BEDAF80456A3711530BACC86D71E0639F0027DAF387BEBFCA9052351956` |
| `lab/reports/soccer-match-review-n123-20261009T0745Z/probe-run.log` | `C15CEBF32E24E1A5BB1AACFBF0E2A134ADC4BDE4E65D92D2A7707E2C713ECF94` |
| `lab/reports/soccer-match-review-n123-20261009T0745Z/scoped-tests.log` | `1C30E7EAC0FE78C28BB35F9E28A2059AC958081BF400605F90516E626D577D9D` |

```
rustc --edition 2024 --test probe.rs -o probe.exe      # private toolchain, exit 0
probe.exe --nocapture --test-threads=1                 # 3 passed; 0 failed
scripts/cargo.ps1 test --test soccer_match --test soccer --test soccer_review
    # soccer_match 27 passed; soccer 11 passed; soccer_review 10 passed; 0 failed
```

Probe-authoring notes kept for the lead: three probe revisions were needed because the API contract
requires the caller to track the authority clock — the initial probe reused stale `new`/`apply`
shapes, used `advance(1.0)` above `max_tick`, and reused an already-consumed sequence. The final
probe tracks `server_time()` between calls; the saved log is the 3/3 run hashed above.
