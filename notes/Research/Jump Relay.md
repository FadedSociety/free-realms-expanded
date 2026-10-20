---
type: research-note
task: T008 / F266 / F290 / F309 / F323 (integrates the D09 draft)
updated: 2026-10-20
status: implemented; F266 blockers fixed in F290, F290 blockers in F309 and its review blockers in F323; focused gates green; native two-client acceptance pending
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
  (`[edi+0xc8]`), then applies the motion. At `0x0090e5f5` (`shr al,2`) and
  `0x0090e5f8` (`and al,1`) it tests bit 2; `0x0090e5fe jz 0x0090e60e`: bit 2
  set uses the packet velocity (`[esp+78h]`), bit 2 clear stores the `xmm0`
  zeroed at `0x0090e5cc`. Bit 1 is `0x0090e5ec mov cl,dl`, `0x0090e5ee shr
  cl,1`, `0x0090e5f2 and cl,1`; the apply path at `0x0090e614` is taken when
  bit 2 (`test al,al`) or bit 1 (`test cl,cl`) is set. At `0x0090e622 test
  dl,1; 0x0090e625 jz 0x0090e62f`: bit 0 **set** stores 0.0 (`0x0090e627`),
  bit 0 **clear** calls `sub_8B3180` (`0x0090e62f`) for the derived value.
- Sender `0092e670`: writes the local actor id into the record's `+8/+0xc`
  from the actor vtable, the character position `+0x10..+0x1c`, the direction
  `+0x20..+0x2c`, a state byte at `+0x30` (bit 0 from a norm compare, bit 2
  forced by `v22 |= 4`), the opaque byte at `+0x31` from client `+0x3e4`, and
  the velocity at `+0x34` from character `+0x674`; the write is gated by client
  `+0x868 = clock + 50` (a 50 ms client-only gate) and by a connected check.

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
  finite. A finite velocity outside +/-100 units/second is clamped, not
  refused: a refused body would end the session. Non-finite position, direction
  or velocity is refused.
- Validate position and direction with `movement::validate` (the 125 speed
  envelope and discontinuity rules). Relay the validated pose, never the raw
  body: the direction is normalized and a soft-over-speed jump is clamped (and
  still relayed at the clamp target).
- Build one relay body per request, after the last child. Position and
  direction come from the final committed pose, so a `[164, 125]` bundle relays
  at the 125 pose; `state`, `unknown` and velocity come from the accepted 164.
  The 164's state bit 2 must survive, because handler `0090e470` applies a 0.0
  velocity when bit 2 is clear. A later application child (a relocation/zone)
  drops the impulse so no jump is relayed at the destination.
- Relay only to the sender's directed visible peers (nearby grid plus
  `PlayerRegistry::knows_character`); the sender never receives its own jump.
- (R) A per-character relay budget of 10 relays/second with a burst of 4,
  through the shared `net_policy::KeyedBuckets` limiter, on the server's
  monotonic wall clock. Over budget the extra 164 is dropped silently; the
  accepted pose still fans out through opcode 125. The key is forgotten when
  the character's session ends. A normal jump cadence (about one per 0.5 s) is
  far below the budget.
- The client's 50 ms sender gate (`0092e670`, `+0x868 = clock + 50`) is not a
  server boundary and is not replicated; the budget is.

## Implementation

- `src/jump_relay.rs`: `decode` (untrusted body), `JumpImpulse`, `relay`
  (server-bound body), the (R) `RELAY_RATE_PER_SECOND`/`RELAY_BURST` constants,
  `OPCODE`, `BODY_LEN`, `MAX_VERTICAL_SPEED`; byte-level unit tests derived from
  reader `008e5ba0`, plus the budget test over `net_policy::KeyedBuckets`.
- `src/net_policy.rs`: `KeyedBuckets::remove`, so a left character's key is
  forgotten.
- `src/gateway.rs`: opcode-164 parse arm; `Event::Position` carries an
  `Option<JumpImpulse>` so the jump shares the 125 gate/validation/pose commit;
  the final body is built once at the end; `reset_session` clears it.
- `src/bin/osfr-gateway.rs`: `Security` owns the per-character `KeyedBuckets`
  (shared across staged world clones, like the action budget); `end_session`
  removes the key; the T199 broadcast and the jump fanout share the post-commit
  delivery hook.
- `src/bin/osfr_gateway/delivery.rs`: `jump_fanout` applies the directed
  visibility rule and the per-character budget; `post_commit` queues the
  broadcast and the relay only after a successful commit. The budget uses the
  server's monotonic wall clock (`Instant::now()`), the lead-accepted F290
  behaviour; F309 briefly anchored it to the dispatch clock and F323 reverted
  that unsigned clock change.
- Tests: `src/bin/osfr_gateway/tests.rs` (real `dispatch_message`: accepted
  relay and decoded wire body, directed cap, no sender echo, flood cap, pose
  fanout while relays drop, normal-cadence recovery, refusal cases with an
  unchanged pose, clamp, `[164, 125]` state/pose bundle, pre-Ready gate, the
  F309/F323 `end_session` leave cleanup, the saved-pose reconnect and the
  wall-clock pin),
  `src/jump_relay.rs` (byte layout, budget), `tests/gateway.rs` (reset and a
  later application child), and `tests/app_fuzz.rs` (hostile 164 seed).

## Known gaps

- No native two-client acceptance: the observer's jump animation, collision
  and gravity are client behavior and were not run in this worktree.
- The 100 units/second velocity clamp and the 10/s burst-4 relay budget are
  reconstructed (R) compiled values; no movement-policy content table exists
  (the 125 speed envelope is likewise compiled and documented (R)).
- The strict body-GUID equality assumes the client's actor id equals the
  server-loaded character GUID. That is what the server sends in self data and
  AddPc, and the body GUID is never trusted for routing; a future client with
  different actor ids would have its jumps refused (deliberate anti-spoofing).
- A combined bundle containing two valid jumps keeps the last accepted impulse;
  both poses are validated and the final pose is what observers receive through
  the normal 125 fanout.
- The state/unknown bytes are relayed opaque beyond the recovered bits (bit 2
  velocity selector; bits 2/1 select the apply path; in `0090e470` bit 0 set
  selects 0.0 and bit 0 clear selects the `sub_8B3180` value).

