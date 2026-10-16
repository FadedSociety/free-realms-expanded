# Soccer Match Authority

## Scope

Server-owned match authority for MMO005 soccer:
`prototype/osfr-tools/src/soccer_match.rs` with
`prototype/osfr-tools/tests/soccer_match.rs` (both new; the module is **not**
registered in `lib.rs` - lead-owned integration). It reuses the reviewed
capture codec in `src/soccer.rs` (`decode`, `SoccerPacket`, `Evidence`) and the
captured Sacred Grove evidence. The client never asserts a goal: it submits
bounded position, control and kick/pass intents, and the authority owns the
ball, possession, the clock and the score.

## Evidence split

| Class | What | Source |
| --- | --- | --- |
| Original | half length 180 s (`soccer::HALF_SECONDS`), client packet ids 1/3/14/16, ball spawn `(226.7, 2.0, 212.905)` offered as `CAPTURED_BALL_SPAWN` | preserved pcapng and reviewed soccer slice |
| Caller-supplied | `FieldBounds` (half extents, goal mouth) and every config value | required input; the capture has **no** pitch dimensions, so no historical field constants are invented |
| Reconstructed | motion step, gravity handling, control/pass radii, kick power bound, out-of-play rule, kickoff reset, `+z`/`-z` attack convention | equations in the module, all overridable |
| Missing | client kick/pass request packet (`ImpulseBall` 13 never captured), rewards, completion traffic | refused or labelled `Missing`; never guessed |

## F1-F6 repair (2026-10-09)

The independent review `notes/Reviews/Soccer Match Authority Review.md` raised
six findings; all six are repaired here and pinned by tests. The reviewer's own
note stays untouched (it is theirs); this section is the resolution record.

| Finding | Repair | Test |
| --- | --- | --- |
| F1 sender binding | `apply(sender, now, intent)`: the authenticated session guid is a separate argument and a mismatch is refused before any player state is read or written | `sender_identity_is_checked_before_any_victim_mutation` |
| F2 victim lockout | Same ordering fix: a spoofed sequence for another player never reaches that player's sequence; the victim's own next intent still applies | `sender_identity_is_checked_before_any_victim_mutation` |
| F3 no time bound | Movement now costs authoritative server time: a position step must fit both `max_position_step` and `max_speed * (now - position_at)` on the monotonic `server_time`; time can never move backwards | `position_intents_are_bounded_by_distance_field_and_server_time`, `intent_time_cannot_move_backwards` |
| F4 dead out-of-play ball | Leaving the field restarts the ball at the kickoff spawn and enters `Restart`; `kickoff()` resumes. Reconstructed, since the capture never shows one | `out_of_play_restarts_bounded_at_the_spawn` |
| F5 wrong id-14 label | `client_packet` decodes the captured client ids 3 and 14 as `ClientPacket::ReadOnly`; `client_intent` refuses them with the accurate read-only reason instead of "uncaptured" | `captured_registration_and_player_state_are_read_only_decodes` |
| F6 no half-time switch | `HalfPolicy::{SwitchSides, KeepSides}` is a caller-owned config field (default `SwitchSides`); `attack_sign` applies it, so half 2 reverses the goal mapping unless the caller says otherwise | `second_half_switches_attack_sides`, `keep_sides_policy_is_caller_owned` |

Repair evidence: RED probes (`17 passed; 5 failed` - F1, F2, F3, F4, F6
behaviorally failed; F5 is a mapping/label fix with no behavioral probe) in
`lab/reports/soccer-match-authority-20261009T0745Z/red-probes.log`; GREEN
`23 passed; 0 failed; 0 ignored` in the sibling `green.log` (first post-fix
iteration `21 passed; 2 failed` from test-geometry mistakes is kept in
`green-attempt1.log`). `cargo test --test soccer` `11 passed` and
`--test soccer_review` `10 passed` are unchanged.

## N1-N3 recheck repair (2026-10-09)

The recheck in the same review note (`F1-F6` section, lines 161+) raised three
more findings; all three are repaired and pinned.

| Finding | Repair | Test |
| --- | --- | --- |
| N1 epsilon accumulation | the `+1e-4` additive epsilon is gone; the budget is an exact f64 comparison against `max_speed * (trusted_now - position_at)`, and any nonzero displacement at zero elapsed is refused | `equal_time_spam_cannot_accumulate_movement` |
| N2 forward-clock poisoning | intents never move the authority clock: `now` is a gateway-supplied, trusted timestamp that is clamped to `server_time`, so a forward spike grants no extra budget and locks nobody out | `intent_clock_is_trusted_and_clamped`, `forward_clock_spike_cannot_poison_the_clock` |
| N2 restore clock domain | `restore` keeps the receiver's `server_time` and rebases every player's `position_at` by the same delta, so elapsed allowances survive a reconnect whose monotonic base differs; the match clock and revision are restored verbatim | `restore_normalises_the_clock_domain_and_preserves_allowances` |
| N3 snapshot validation | `restore` now runs a fail-closed `validate_snapshot`: finite clock/ball state, `position_at <= server_time` per player, finite positions. Corrupt snapshots are **not constructible** today (private fields, no deserialization), so the remaining corrupt-format cases are not applicable; no serialization was added | covered by the existing restore/config/stale tests; the checks are defense in depth |

Trusted-clock contract: `apply(sender, now, intent)` takes `now` from the
gateway's clock, never from a packet - captured client packets carry no time
and `client_packet` has no time parameter (`client_packets_never_supply_time`).
`server_time` moves only through `advance(dt)` and the `restore` rebase, and it
is stored in f64 (with per-player `position_at` in f64) so a rebase preserves
allowances exactly instead of accumulating f32 error.

N-recheck evidence: RED probes `23 passed; 3 failed` (N1 accumulation, N2 spike
poisoning, N2 domain lockout) in
`lab/reports/soccer-match-authority-20261009T0830Z/red-probes.log`; GREEN
`27 passed; 0 failed; 0 ignored` in the sibling `green.log`.

## Authority model

- `client_intent(&[u8], sequence)` accepts only the captured client ids:
- `client_packet(&[u8], sequence)` decodes the captured client ids:
  `UpdatePlayerPos` -> `Intent::Position`, `UpdatePlayerControls` ->
  `Intent::Controls`, and the captured `RegisterPlayer`/`UpdatePlayerState`
  (ids 3/14) -> `ClientPacket::ReadOnly` with the decoded packet. `GoalMade`
  and every other server-only id are refused; sub 13 fails with the explicit
  missing-mapping message. `client_intent` is the intent-only view and refuses
  the read-only ids with their actual reason. The captured client packets carry
  no sequence field, so the caller supplies the monotonic per-player sequence
  (reliable-layer ordering).
- `Authority::register(player, team, slot)` is the server roster path
  (client bytes cannot reach it).
- `apply(sender, now, intent)` guards, in order: phase must be `Running`;
  `sender` must equal the payload player (refused before any player state is
  read); `now` must be finite and not move backwards on `server_time`; the
  player must be registered; values must be finite; the sequence must be newer
  than the last accepted one; then per-kind rules. Rejected intents leave the
  match untouched and do not consume the sequence.
  - Position: inside the caller-supplied field and within
    `max_position_step` of the last verified position (or of the kickoff spawn
    for a first position), and within `max_speed * (now - position_at)` of the
    authoritative server-time budget (F3).
  - Controls: decoded and recorded for ordering only; the capture does not
    define server movement from them.
  - Kick/Shot: possession plus `control_radius` proximity, power in
    `(0, max_kick_power]`, non-zero direction; velocity = normalised
    direction * power.
  - Kick/Pass: same guards, target must be a registered teammate with a
    verified position within `pass_radius` of the ball; the direction is
    derived from the verified positions (the intent's direction is validated
    but not trusted for the pass vector).
- `advance(dt)` is the deterministic step, bounded by `max_tick`, and it is
  the only other way `server_time` moves: clock first (a half can never end
  with the ball in flight), then
  `position += velocity * dt`, then `velocity.y -= gravity * dt`, ground
  clamp, then in order: goal crossing, out-of-play (velocity zeroed), pass
  completion (`live_pass` target within `control_radius` takes possession),
  server-side possession assignment to the nearest registered player within
  `control_radius`. While a player holds the ball the ball follows the holder.
  The last toucher cannot immediately re-take the ball until it leaves their
  control radius.
- Goals: a signed crossing of the team's attack goal line with the
  interpolated crossing point inside the caller-supplied mouth. First-half
  convention: team 0 attacks `+z`, team 1 `-z` (reconstructed); `HalfPolicy`
  decides whether the interval swaps them (F6). A goal is counted once per
  crossing: the phase becomes `Restart`, velocity zeroes, and only `kickoff()`
  resumes play. Out-of-play crossings restart bounded at the spawn (F4).
- Clock: `HalfTime` at the end of half 1, `start_second_half()` resets the
  ball and clock, `Finished` at the end of half 2 with a once-only `Outcome`
  (`reward: Evidence::Missing` - no price, item or currency is invented).
- Reconnect: `snapshot()` is an exact clone; `restore()` rejects a mismatched
  configuration and a stale revision, and restored sequences make replayed
  intents stale.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| RED (test first) | `scripts/cargo.ps1 test --test soccer_match --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | `could not compile ... couldn't find file tests\..\src\soccer_match.rs` - log `lab/reports/soccer-match-authority-20261009T0315Z/red-test.log` |
| GREEN (first slice) | same command | `17 passed; 0 failed; 0 ignored` - logs `soccer-match-authority-20261009T0315Z/green-attempt1.log` (13/16 after the first iteration), `green.log` (final) |
| RED (F1-F6 probes) | same command with the five review probes against the reviewed source | `17 passed; 5 failed` - log `soccer-match-authority-20261009T0745Z/red-probes.log` |
| GREEN (F1-F6 repair) | same command after the repair | `23 passed; 0 failed; 0 ignored` - logs `soccer-match-authority-20261009T0745Z/green-attempt1.log` (21/23), `green.log` (final) |
| RED (N1-N3 probes) | same command with the three recheck probes against the F1-F6 revision | `23 passed; 3 failed` (epsilon accumulation, forward-spike poisoning, restore domain lockout) - log `soccer-match-authority-20261009T0830Z/red-probes.log` |
| GREEN (N1-N3 repair) | same command after the repair | `27 passed; 0 failed; 0 ignored` - log `soccer-match-authority-20261009T0830Z/green.log` |
| Format | `rustfmt --edition 2024 src/soccer_match.rs tests/soccer_match.rs` (no `cargo fmt`, no other files) | clean |
| Prior soccer targets | `cargo test --test soccer` / `--test soccer_review` | `11 passed` / `10 passed` unchanged |

Tests cover: spoofed `GoalMade`/ball-transform/unmapped-kick rejection, captured
client packet mapping and read-only ids 3/14, unregistered/stale/non-finite
guards, sender-identity and victim-lockout ordering, distance/field/server-time
bounds, backwards time, possession-required and proximity kicks, deterministic
motion with and without gravity, server-side possession assignment, pass
progression and pass guards, goal crossing once (inside vs outside the mouth),
bounded out-of-play restart, kickoff reset with score preserved, half transition
and once-only finished outcome, side-switch policies for both `HalfPolicy`
values, captured vs caller half labelling, snapshot/restore with replayed-intent
rejection, bounds and registration validation, and captured spawn coordinates.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/soccer_match.rs` | `a1406892df98523becd17ace6e4e0b8e4f8ee054a5e71126e604a990a022bd50` (F1-F6 revision `326db82a…`) |
| `prototype/osfr-tools/tests/soccer_match.rs` | `6cf10153530b9f06e7154fa35ed143ba07ed20c1720a82c85bd23f0501b59931` (F1-F6 revision `6cea0461…`) |
| `lab/reports/soccer-match-authority-20261009T0315Z/red-test.log` | `1bfdd0d5edbf6e4cbb4470c787407b9873b7f1a26988663dc2d7fe95443bb13b` |
| `lab/reports/soccer-match-authority-20261009T0315Z/green.log` | `377483bca2831896f24b43c587cfcc7fa544e4bab8f15386166b0908332d6989` |
| `lab/reports/soccer-match-authority-20261009T0745Z/red-probes.log` | `55c28908cc662d3221a2655ff723c469e86f08676e82024c5d6cdeeda8363bf8` |
| `lab/reports/soccer-match-authority-20261009T0745Z/green.log` | `e0e76db0f0b7cb44161eaf1c2d9b4c00c99ddbc564bba57ad4f3a8570ae8d7ca` |
| `lab/reports/soccer-match-authority-20261009T0830Z/red-probes.log` | `7dd6ca93bd1cd8e6f15c33340d1f305a51edc093e3094489d77ee6f2141a077b` |
| `lab/reports/soccer-match-authority-20261009T0830Z/green.log` | `a30d0610511bfa5895c9bca9f5ccf8b055c5028a62e30f70ac9ff98d248cd7c3` |

`lab/reports/soccer-match-authority-20261009T0315Z/summary.json` and
`lab/reports/soccer-match-authority-20261009T0745Z/fix-summary.json` and
`lab/reports/soccer-match-authority-20261009T0830Z/summary.json` hold the same
facts in machine-readable form; each directory keeps its first-iteration log
next to the final one.

## Limitations and next acceptance

- The physics is a reconstructed deterministic slice, not recovered
  ForgeLight simulation: no terrain, no collision, no player-vs-player
  contact, no ball spin or bounce. Nothing here claims physical authority.
- Field and goal dimensions must come from a verified source before any real
  match; the tests use arbitrary caller-supplied bounds on purpose.
- Kick and pass have no captured wire layout (sub 13 missing), so a native
  client cannot yet drive them; the mapping stays an explicit gap.
- Possession, pass completion and goal crossing are server rules invented for
  a bounded slice; only the clock length, packet ids and spawn are original.
- The F1-F6 repairs are pinned by tests but still reconstructed policy: the
  server-time budget (`max_speed`), the out-of-play restart and the second-half
  side switch have no captured counterpart.
- The N1-N3 policy is also reconstructed: `now` is a trusted gateway clock
  clamped to the authority clock, the movement clock is f64 for exact rebases,
  and corrupt snapshots are unreachable today (private fields, no
  deserialization) so `validate_snapshot` is fail-closed defense, not a tested
  corrupt-file path.
- Native acceptance, visual verification and integration into the gateway are
  not part of this work. Integration must pass the authenticated sender and a
  trusted, monotonic gateway timestamp into `apply`, tick the authority clock
  through `advance`, feed `client_packet`/`client_intent` with the reliable
  layer's per-player sequence, register players server-side, and keep every
  server-only packet id out of the client path.
