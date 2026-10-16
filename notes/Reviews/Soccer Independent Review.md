# Soccer independent review

Scope: MMO005 soccer slice (`prototype/osfr-tools/src/soccer.rs`, `tests/soccer.rs`)
against the preserved Sacred Grove capture. Reviewer owns
`tests/soccer_review.rs`, this note, and
`lab/reports/soccer-review-20261008T1942Z/`. No other existing file touched.

Verdict: the decoder/protocol reconstruction is confirmed, and the four
authority defects found by this review are repaired and pinned by ordinary
tests. The slice is now a capture-driven reducer with an explicit direction
gate - server-to-client playback is the only path that changes match state.
It is still not physics, not completion handling and not native-verified; those
gates are unchanged below.

## Repair evidence

The four probes were RED before the repair (log
`lab/reports/soccer-review-20261008T1942Z/red-baseline-4probes.log`: 4 failed,
0 passed) and are ordinary guard tests after it (log `postfix-soccer-review.log`:
10 passed, 0 failed, 0 ignored). The existing suite now reports 11 passed
(`postfix-soccer.log`), and the capture verifier reports 21/21
(`postfix-capture-verify.log`).

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/soccer.rs` | `2c18c712a12b9fc003c0529a7146c8185f6ca80fff19fa24273d3e654b27e2d9` (pre-repair `a177ea48…`) |
| `prototype/osfr-tools/tests/soccer.rs` | `ee75c3841ae1eaf518db35ec33e5a9a85eaa89d9f8ae389727969c401516e8bc` (pre-repair `85ac58f2…`) |
| `prototype/osfr-tools/tests/soccer_review.rs` | `eb0d6b1831a163bc60932ae126097fc5f27cd889f0e560e5c805b7c28f9629f3` |
| `lab/reports/soccer-review-20261008T1942Z/verify_soccer_capture.py` | `37a73e96c53788e56b142413b998c6216ff44c8a0071e0c361f111d958f707f9` |

Fix surface: `handle(&mut self, Direction, &[u8])` is now the single entry
point. `Direction::ServerToClient` drives the reducer; `Direction::ClientToServer`
accepts only the packet ids the capture shows the client sending (1 position,
3 registration, 14 state, 16 controls), returns them decoded, and refuses any
server-only packet id such as `GoalMade`. `apply_goal` requires `Phase::Running`;
`UpdateSoccerBall`/`FreeBall` reject non-finite geometry before mutation;
`AcquireBall` requires a registered holder. `tests/soccer.rs` now replays all
eight captured `RegisterPlayer` messages and asserts that client bytes never
move authoritative state.

## Resolved findings

R1 (was high): fixed. A client-direction goal is refused before any mutation
(`client_direction_never_scores_or_moves_authoritative_state`), and the client
position/control vectors change nothing. The same goal still scores when it
arrives as `ServerToClient`, matching the capture, where every goal/clock/ball/
team/config message is flow 0 = `64.37.170.6:20267 -> 192.168.1.145:61759`.

R2 (was medium): fixed. A consistent second goal during the `Restart` clock
hold is refused (`goal_during_restart_hold_is_rejected`); it counts again after
the reconstructed restart resumes the half.

R3 (was medium): fixed. `[NaN, inf, 0.0]` ball transforms are refused with no
state mutation, while finite geometry (the captured spawn shape) still lands
(`nonfinite_ball_geometry_is_rejected_before_mutation`).

R4 (was low-medium): fixed. Possession by a never-registered holder is refused
with no mutation; the captured `RegisterPlayer` for that player must arrive from
the server first (`unregistered_holder_is_rejected_before_mutation`).

R5 (was low, documentation): fixed. `notes/Research/Soccer Restoration.md` now
records the current counts - 37 capture entries / 36 distinct messages plus one
synthetic unknown-opcode vector - and the verifier asserts them
(`notes and tests agree on 36 distinct capture vectors / 37 entries`).

## Still confirmed from the original review

`verify_soccer_capture.py` re-derives 21/21 checks without using the worker's
decoder: pcapng sha256 `9e609bb5…`; 2917 opcode-76 messages; per-sub counts
matching the note table; envelope invariants; the single `GoalMade` at frame
9507 with fields `(0,1,1)`; the 0.0 -> 180.0 kickoff at frame 6424; the
monotone countdown; the `151.412` hold from frame 9530; the 5-acquire/5-free
possession chain; the 4v4 roster; config asset/spawn offsets; colours; and the
flow direction above. `SoccerState.as` `STRINGS` matches `SCOREBOARD_STATS` and
minigame id 42 is "Sacred Grove Soccer". All 36 distinct capture vectors in
`tests/soccer.rs` are unmodified reassembled messages, and the byte-exact
encode/decode round trip still passes for every one of them.

## Gates that remain

No second half, completion, reward or return traffic was ever captured; the
reward proposal stays a `Missing`-amount record shape. Ball physics and
collision are absent and must stay absent - the reducer repeats server-reported
ball state and claims no physical authority. `UpdateGameState` (id 5) remains
opaque; ids 13/17/22/24 have no samples. `GoalMade` semantics still rest on one
sample. Native client acceptance for soccer has not been run and is not claimed.
Integration must pass server traffic as `ServerToClient` and client traffic as
`ClientToServer`; the reducer cannot see the UDP tuple itself, so that mapping
is the gateway's responsibility.
