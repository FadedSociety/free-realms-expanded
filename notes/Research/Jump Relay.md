---
type: research-note
task: T008 (integrates the D09 draft)
updated: 2026-10-20
status: implemented and self-reviewed in the T008 worktree; lead review and native two-client acceptance pending
---

# Jump Relay (Opcode 164)

## Decision

Opcode 164 is a separate 40-byte jump update and is relayed to observers. The
opcode-125 position path is unchanged. The D09 draft's claim that 125 does not
carry the jump bit is background, not re-verified by T008; T008 verified only
the 164 packet path below and does not depend on the 125 claim.

## Recovered client facts (T008 IDA pass, 2026-10-20)

The 2026-10-20 checks are disassembly/decompilation of the pinned 2014 client;
every address is in `lab/working/ida/FreeRealms.exe.i64`.

- Constructor `008b3b70`: stores the packet vtable, dword `[this+4] = 0xA4`
  (164), the GUID pair at `+8/+0xc`, 4-float blocks at `+0x10` and `+0x20`,
  bytes `+0x30/+0x31`, velocity `+0x34`.
- Writer `009127f0`: 2 bytes from `[this+4]`; `sub_8fb810(&this+8)` (u64 id);
  f32 `[this+0x10]`, `[+0x14]`, `[+0x18]`, `[+0x20]`, `[+0x24]`, `[+0x28]`;
  byte `[+0x30]`; byte `[+0x31]`; 4-byte value from `[+0x34]`. That is
  `2 + 8 + 12 + 12 + 1 + 1 + 4 = 40` bytes on the wire. `+0x1c` and `+0x2c`
  are not written.
- Reader `008e5ba0`: int16 to `+4`; 8 bytes to `+8/+0xc`; f32 to
  `+0x10/+0x14/+0x18`; f32 to `+0x20/+0x24/+0x28`; byte `+0x30`; byte
  `+0x31`; dword `+0x34`. It fills `+0x1c` with `1.0` and `+0x2c` with `0.0`
  itself, so those fourth components are wire-absent.
- Handler `0090e470` (disassembly; IDA does not define this body as a function):
  constructs the jump record (`008b3b70`), reads it, rejects a short/trailing
  body (the bounded reader's cursor must reach the end), looks up the actor by
  the 8-byte id (`sub_984960`), rejects a missing actor or the local player
  (`[edi+0xc8]`), then applies the motion.
- Sender `0092e670`: writes the local actor id into the record's `+8/+0xc`
  from the actor vtable, the character position `+0x10..+0x1c`, the direction
  `+0x20..+0x2c`, a state byte at `+0x30` (bit 0 from a norm compare, bit 2
  forced), the opaque byte at `+0x31` from client `+0x3e4`, and the velocity at
  `+0x34` from character `+0x674`; it sends only when connected.

So the application body is:
`u16 164, u64 guid, 3 x f32 position, 3 x f32 direction, u8 state,
u8 unknown, f32 vertical_velocity`.

- Pinned Free Realms Legacy reference (read for behavior, no code copied):
  `PlayerUpdatePacketJumpHandler.cs` sets `packet.Guid = connection.Player.Guid`,
  updates the player's position, then sends the tunneled packet to that
  player's visible set. Our server's directed-cap equivalent is the reverse
  set: only sessions whose own directed view contains the sender (they were
  sent its AddPc). Sending to the sender's own view would reach clients that
  never received the sender's AddPc under a per-viewer cap; the native handler
  discards updates from unknown actors, so that traffic would be wasted.

## Reconstructed server policy (R)

- Accept exactly the 40-byte body whose opcode is 164, whose body GUID equals
  the session's character GUID, whose position and direction are finite and
  within the 125 parser's 1,000,000-unit bound, and whose vertical velocity is
  finite and at most 10,000 units/second.
- Validate position and direction with `movement::validate` (the 125 speed
  envelope and discontinuity rules). Relay the validated pose, never the raw
  body: the direction is normalized and a soft-over-speed jump is clamped
  (and is not relayed).
- Relay only to the sender's directed visible peers (nearby grid plus
  `PlayerRegistry::knows_character`); the sender never receives its own jump.
- One relay per session per 100 ms. This is server policy, not a recovered
  client constant. It is consumed at the fanout
  (`PlayerRegistry::allow_jump`), so a throttled jump still applies its
  validated pose exactly like opcode 125; only the observer broadcast is
  throttled. `leave` drops the window with the session.
- A later committed position child in the same bundle supersedes an earlier
  jump body (`GatewayOutcome::jump` is cleared), so observers never receive a
  stale 164 pose after the final 125 pose fanout.
- `vertical_velocity`, `state` and `unknown` are carried through from the
  validated client body; their meanings are the client handler's
  (`0090e470`) and are opaque beyond the recovered bits.

## Implementation

- `src/jump_relay.rs`: `decode` (untrusted body), `relay` (server-bound body),
  `OPCODE`, `BODY_LEN`, `MIN_INTERVAL_MS`, `MAX_VERTICAL_SPEED`; byte-level
  unit tests derived from reader `008e5ba0`.
- `src/gateway.rs`: opcode-164 parse arm, an `Option<f32>` jump velocity on the
  existing `Event::Position` so the jump shares the 125 gate/validation/pose
  commit, and `GatewayOutcome::jump` (set only on a non-clamped accept).
- `src/bin/osfr_gateway/delivery.rs`: `jump_fanout` applies the directed
  visibility rule and the throttle.
- `src/players.rs`: the per-session `jump_ms` window and `allow_jump`, removed
  by `leave`.
- Tests: `tests/players.rs` (window, per-session independence, leave reset),
  `src/bin/osfr_gateway/tests.rs` (real `dispatch_message`: accepted relay and
  decoded wire body, directed cap, no sender echo, throttle and next window,
  refusal cases with an unchanged pose and window, clamp, later-125 supersede,
  pre-Ready gate), `tests/app_fuzz.rs` (hostile 164 seed), and the
  `src/jump_relay.rs` unit tests. The gateway-level `tests/gateway.rs` hunks
  from the failed integration attempt were dropped: this worktree is cut from
  `568ac24`, before F235 (`f5b1961`) moved that file's `Reference` helper to
  `tests/common/gateway_reference.rs`, so those hunks could not compile on the
  integration tip. Their cases moved to the real-dispatch test, which compiles
  on both trees.
- The salvaged `tests/jump_relay.rs` was retired: it compiled private
  `#[path]` copies of the module and used the draft's `outcome.jumps` /
  `visible_peers` API. Its cases live in the suites above, which use the
  registered module and the real gateway.

## Known gaps

- No native two-client acceptance: the observer's jump animation, collision
  and gravity are client behavior and were not run in this worktree.
- The 100 ms window and the 10,000 units/second velocity guard are
  reconstructed (R) compiled values; no movement-policy content table exists
  (the 125 speed envelope is likewise compiled and documented (R)).
- The strict body-GUID equality assumes the client's actor id equals the
  server-loaded character GUID. That is what the server sends in self data and
  AddPc, and the body GUID is never trusted for routing; a future client with
  different actor ids would have its jumps refused (deliberate anti-spoofing).
- A combined bundle containing two valid jumps keeps only the last
  `GatewayOutcome::jump`; both poses are validated and the final pose is what
  observers receive through the normal 125 fanout.
- The state/unknown bytes are relayed opaque; their meanings are not fully
  recovered.
