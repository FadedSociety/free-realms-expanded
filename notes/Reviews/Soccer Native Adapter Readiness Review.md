# Soccer native adapter readiness review

Read-only independent review, 2026-10-09 (UTC). Scope: current head
`prototype/osfr-tools/src/soccer.rs` (`2C18C712…`) and
`prototype/osfr-tools/src/soccer_match.rs` (`A1406892…`, the N1-N3-verified revision) against
the accepted decode `notes/Research/Soccer Kick Pass Wire Mapping.md` revision 3 (`0EE9BF5F…`)
plus the two accepted independent reviews (wire `046A9768…`, authority `4FA390DD…`). Reviewer
owns this note and `lab/reports/soccer-native-adapter-readiness-20261009T1625Z/` only; no source,
test, gateway, lib, save, content, UI, release or process was changed. Skills: RTK (normal
commands), Ponytail (one small probe reusing the real modules), Caveman (terse report),
reverse-engineering (the decode is a decompile claim; this review reads it as such).

**Verdict: `soccer_match` is a coherent server-side *candidate* slice, not native-playable.**
The client's captured kick/pass channel (sub 14 state codes 16-34) cannot reach it, the sub-16
control bitfield is decoded as geometry, the sub-13 impulse is neither accepted nor emitted, and
no production route/startscene/roster/tick/end/return wiring exists. No native acceptance is
claimed anywhere.

## Findings

**A1 (High) - sub 14 is the client kick/pass channel, but the codec reads the wrong width and the
adapter refuses it.** `soccer.rs:210-217` and `:526-532` decode id 14 as
`{u64 player, u32 state, [u8;34] body, [f32;3] position, f32 trailing}`, while revision 3 pins
`u64 guid, u8 state, u8 aux, u32 sequence, f32[4] x3` (mapping note lines 262-330).
Probe (`probe.rs::sub14_kick_state_cannot_reach_an_intent`): a true rev3 record carrying
`cSoccerPlayerStateKickRight` (state byte 17) decodes into `state = 458769` (`17 | seq << 16`),
`client_packet` returns `ReadOnly`, and `client_intent` fails with
`captured client player state is read-only for the authority`
(`soccer_match.rs:302-304`, `:313-325`). The module doc "Missing. The client kick/pass request
packet" (`soccer_match.rs:15-18`) is stale: the channel and enum are recovered; what is missing is
a captured kick sample and the aux/float4 semantics. Fixing F5 as read-only was correct for the
10-08 capture slice, but revision 2/3 of the mapping changed what id 14 is.

**A2 (Medium) - the sub-16 control bitfield is decoded as a direction component.**
`soccer.rs:218-221`/`:537-540` read `direction: [f32;3]`, but revision 3 pins `f32 x, f32 y, u32
control bits` (mapping lines 86-93). Probe
(`probe.rs::sub16_control_bits_are_read_as_a_direction_component`): captured control words
`{0,1,2,8,17,65,256}` each surface as a denormal f32 in `direction[2]` (`to_bits == bits`;
`65 -> 9.1e-44`), and `Intent::Controls` (`soccer_match.rs:228-232`, `:295-300`) carries that as
geometry. The only consumer records the sequence (`:600-614`) and drops the bits, so nothing
unsafe happens today, but the per-flow control semantics (`{0,1,2}` client vs
`{8,0x11,0x41,0x100}` server relay) cannot be expressed.

**A3 (Medium) - sub 13 has a recovered layout but no adapter or emitter, and its refusal text is
stale.** Id 13 falls to `Unknown` (`soccer.rs:233-235`) and `client_intent` refuses it with
`soccer kick/pass request mapping is missing from the capture` (`soccer_match.rs:306-308`), though
revision 3 recovered the 36-byte `SoccerPacketImpulseBall` layout (mapping lines 47-83; no
captured vector exists and the direction is an inference). The stronger gap is outbound: nothing
in the authority ever emits an impulse or a ball update after a kick; the only encoder is
`soccer::encode` (`soccer.rs:428-438`) and `soccer_match` has no message builder at all.

**A4 (High, integration) - there is no production wiring for soccer anywhere.**
`src/lib.rs` (28 lines) declares no `soccer`/`soccer_match` module, so the production crate cannot
call either; outside the two files the only references are the `piece_018.rs:242-254` registry
string and test `#[path]` includes (`tests/soccer_match.rs:16-19`, `tests/soccer.rs:18-19`,
`tests/soccer_review.rs:19-20`). The gateway hands application bytes to
`gameplay.handle_in_world` (`gateway.rs:663-698`); `gameplay.rs:2164-2233` has no opcode-76 arm
and falls through to chat/combat handling. There is no soccer scene start (the active-minigame
registry `gameplay.rs:394-401` lists forging/smelting/cooking/mining/harvesting only), no
account-guid to `Authority::register` bridge (`soccer_match.rs:465-491` has no caller), no
`advance(dt)` tick driver (gateway `tick_at` only ticks opening flows), and no minigame end/return
(contrast the other minigames, `encounter.rs:560`, `:965-981`, which emit 39/19 LeaveMiniGame).

**A5 (Low, ownership) - two unconnected state machines.** `soccer::Match` (capture playback,
phases Lobby/Running/Restart/Finished) and `soccer_match::Authority` (physics, phases
Kickoff/Running/Restart/HalfTime/Finished) share no state and no code path; integration must pick
one authority or define the bridge explicitly. Today neither is reachable in production.

## Missing-wiring checklist

| Piece | Status |
| --- | --- |
| Adapter (opcode 76 route, lib export, sub-14 state->intent) | missing (A1/A4) |
| Startscene / minigame start | missing (no soccer entry in `gameplay.rs:394-401`) |
| Roster (session guid -> team/slot) | `Authority::register` exists, no caller/bridge |
| Ball update to client (id 4/12/sub-13) | missing outbound; `soccer::encode` unused |
| Update/tick (`advance(dt)`) | missing; no production caller |
| Phase (kickoff/half/restart) | logic exists inside both modules, no orchestration caller |
| End/return (finish, LeaveMiniGame, world return) | missing; reward stays `Evidence::Missing` |

## Verified clean (re-run this review)

* Scoped suites: `soccer` 11 passed, `soccer_match` 27 passed, `soccer_review` 10 passed, 0
  failed (`scoped-tests.log`).
* Independent probe, 4/4 (`probe-run.log`, built with the repo toolchain
  `lab/tools/cargo/bin/rustc.exe`): sender mismatch refused before victim state; a forward `now`
  spike grants no budget and does not move the clock (`server_time` stays 0); equal-time spam
  refused; `advance` cap enforced (`1.0` refused, `0.25` accepted, only `advance` moves the
  clock); kick without possession refused; NaN bounds fail closed. No new authority defect was
  found; N1-N3 behavior still holds.

## Minimal next integration scope (prioritized)

1. RED first: turn the probe's sub-14/sub-16 vectors into repo tests, then fix the widths
   (sub 14: `u8` state, `u8` aux, `u32` sequence, three `f32[4]`; sub 16: `u32` bits).
2. Adapter: expose a state intent (guid, state 16-34, aux, seq, float4s) from `client_packet`
   with the `apply(sender, ...)` binding; classify kick vs pass from the state enum and keep
   power/target/possession server-side; correct the sub-13 refusal text.
3. Route + tick: register the modules in `lib.rs`, add an opcode-76 arm gated on an active soccer
   match, tick `advance(dt)` from `tick_at`, and emit authority ball/clock/score messages through
   the existing `wrapped` send path (sub-13 only as labelled inference).
4. Lifecycle: startscene + roster bridge + half/finish + LeaveMiniGame/return, mirroring the
   other minigames; keep `reward: Missing` until an original completion message is recovered.
5. Native acceptance stays open until the real client drives a kick/pass and the ball moves.

## Evidence

| Artifact | SHA-256 |
| --- | --- |
| `src/soccer.rs` (reviewed) | `2C18C712A12B9FC003C0529A7146C8185F6CA80FFF19FA24273D3E654B27E2D9` |
| `src/soccer_match.rs` (reviewed, N1-N3 revision) | `A1406892DF98523BECD17ACE6E4E0B8E4F8EE054A5E71126E604A990A022BD50` |
| `tests/soccer.rs` / `tests/soccer_match.rs` / `tests/soccer_review.rs` | `EE75C384…` / `6CF10153…` / `EB0D6B18…` |
| `notes/Research/Soccer Kick Pass Wire Mapping.md` (rev 3) | `0EE9BF5FB5CBC95CA20B74DDCF8542B2C79457E0C0BE25D4968FC05FB026A5AB` |
| `lab/reports/soccer-native-adapter-readiness-20261009T1625Z/probe.rs` | `7D219A72EDDCEE90712EAF01E8F9AD8FD2DEA395977083D37B1999DD97870606` |
| `…/probe-run.log` (4 passed) | `5967BAF312F185B5BF7ECCF20F6E2F4011736669CE45511D7DB91FCBFCEA4DD3` |
| `…/scoped-tests.log` (11/27/10) | `D54BC946DD127F77255A71E9CC6C271D79BB5B454A24FF613B54AF2C26D070E2` |
| `…/static-evidence.txt`, `…/compile.log`, full list | `final-hashes.txt` |

Commands: `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test
soccer_match --test soccer --test soccer_review`; `lab/tools/cargo/bin/rustc.exe --edition 2024
--test probe.rs -o probe.exe` then `probe.exe --nocapture --test-threads=1` (paths from the
private report). All reads are over preserved, read-only inputs; no game, gateway, save, UI or
process was touched.

Limits: the decode it compares against is decompile-based (rev 3) and itself lists gaps - no
captured sub-13 vector, unnamed aux/float4 semantics, no key/mouse binding table; the physics in
`soccer_match` remains a reconstructed slice with caller-supplied field dimensions. This review
adds no native-playability claim and changes no source.

Cross-links: [[Research/Soccer Kick Pass Wire Mapping]], [[Soccer Kick Pass Wire Review]],
[[Soccer Match Authority Review]], [[Research/Soccer Match Authority]].
