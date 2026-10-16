# Soccer Service Integration

Status: implemented, RED then GREEN, with the frozen adapter reused unchanged
and its 14 lab tests re-run against the modified authority. No native
playability, no gateway registration, no packet sent. Owned paths:
`prototype/osfr-tools/src/soccer_service.rs` (new),
`prototype/osfr-tools/src/soccer_match.rs` (one additive `Authority::unregister`),
`prototype/osfr-tools/tests/soccer_service.rs` (new), this note and the private
bundle `lab/reports/soccer-service-repair-20261009/` (a sibling of the
repository under `CartoonMMO`). `lib.rs`, gameplay,
gateway, game_store, Cargo, content, save, launcher, release and UI files were
not touched.

## Verified artifacts

| Path | sha256 |
| --- | --- |
| `src/soccer_service.rs` | `5e156eaa…` |
| `tests/soccer_service.rs` | `92a88677…` |
| `src/soccer_match.rs` (after) | `eb73cc17…` (before `33f642e2…`, the adapter-receipt pin) |
| `src/soccer.rs` | `c68bb857…` unchanged; no fix was required |
| `src/zone_transition.rs` | `0ecf2bca…` unchanged; read-only dependency |
| frozen adapter `lab/reports/soccer-state-adapter-20261009T174320Z/src/adapter.rs` | `11d291c2…` unchanged |
| captured `soccer-01283-flow0-frame690-op31-subNone.bin` | `a32d3315…` (93 bytes) |

The only production edit is the additive `Authority::unregister` (leave seam):
it drops the roster entry, releases possession held by that player, clears a
live pass naming them and clears the touch flag. No existing line changed; the
existing `soccer` / `soccer_match` / `soccer_native_wire` / `soccer_review`
targets and the adapter's own 14 tests all still pass.

## What the service is

`SoccerService` composes the reviewed pieces; it does not re-derive them.

- Wire codec: `soccer::decode`/`encode` (byte-exact against the preserved
  capture, see [[Soccer Native Wire Repair]]).
- Authority: `soccer_match::Authority` (server-owned clock, possession, ball
  and score; see [[Soccer Match Authority]]).
- Sub-14 adapter: `state_request_intent` / `apply_state_request` copied
  verbatim from the frozen private adapter (`notes/Reviews/Soccer State Input
  Adapter Review.md`), so the proven `aux / 255` charge and vecA direction math
  keep their verified behavior.

API surface (all bounded; every refusal leaves the authority untouched):
`join`/`leave`, `start`/`second_half`/`tick`, `handle_client`, `snapshot`/
`restore`, read-only accessors, `entry_frame` and `emit_packet`.

## Ordered integration seams for the lead

1. **Construct.** `SoccerService::new(MatchConfig, AdapterPolicy)`. Pitch
   bounds are caller-supplied and validated; the capture has no pitch
   dimensions, so nothing here invents one. `MatchConfig::kickoff_spawn` can be
   `soccer_match::CAPTURED_BALL_SPAWN`; `half_seconds` defaults to the captured
   `soccer::HALF_SECONDS` (180 s).
2. **Scene entry.** `entry_frame(position, rotation)` returns the original
   opcode-31 `ClientBeginZoning` frame for `sg_soccer`, encoded by the reviewed
   `zone_transition` codec ([[Native Zone Transition Codec]]). The scene name,
   type `2`, sky `sky_wilds24.xml`, unknown byte `1`, geometry id `107`,
   override `true` and flag `1` are the captured frame's values; position and
   rotation are the caller's spawn. The env-gated test decodes the preserved
   file, re-encodes it byte-exact and compares it to `entry_frame()`.
3. **Roster entry.** `join(session, team, slot)` registers the authenticated
   session guid (the project's identity guid is the authority's player id) and
   returns the captured relays `Registered` (sub 3, marker `[0, 1]`) and
   `TeamAssigned` (sub 6). Send each with `event.wire()`.
4. **Route client family-76 bytes.** `handle_client(sender, now, bytes,
   sequence, pass_target)`:
   - sub 1 position and sub 16 controls become authority intents; controls stay
     advisory bits (no bit is named kick/pass by the client);
   - sub 14 kick/pass states go through the frozen adapter, and an accepted
     kick returns one `Impulse` event (sub 13);
   - sub 3 registration and non kick/pass sub-14 states are refused with their
     read-only reason; sub 2 `GoalMade` and sub 13 inbound are refused.
   `sender` is the authenticated session and `now` the gateway's monotonic
   server clock; the payload guid is never trusted.
5. **Tick.** `tick(dt)` advances the deterministic simulation and returns the
   authoritative events: a `Goal` (only the server tick can score), the
   reconstructed out-of-play `FreeBall` relay, a `Possession` change, then the
   half clock. `dt` is bounded by `PhysicsConfig::max_tick`.
6. **Output.** `ServiceEvent::packet()` / `wire()` produce captured-envelope
   family-76 frames; `emit_packet` is the fail-closed gate.
7. **Save.** `snapshot()` returns the authoritative state; `restore(&snapshot)`
   applies the frozen rules (configuration mismatch, corrupt state and stale
   revisions refused; the receiver's clock domain is preserved). The service's
   own session map is gateway state and is deliberately not part of the
   authority snapshot.

## Emissions: recovered vs reconstructed

| Event | Sub | Class |
| --- | --- | --- |
| `Registered` | 3 | layout + server marker `[0, 1]` captured (the one captured client registration used `[0, 0]`, so the entry is server-assigned) |
| `TeamAssigned` | 6 | layout captured |
| `Possession` | 11 | layout + flag `0` captured (5/5 records) |
| `FreeBall` | 12 | layout, flag `0`, unit scale `1.0` captured (5/5 records) |
| `Goal` | 2 | layout captured; the wire carries `[a, b, a + b]` (single captured goal `(0, 1, 1)`) |
| `Impulse` | 13 | layout recovered; field mapping **reconstructed** (below) |
| `GameTime` | 23 | layout captured (78 records) |

The sub-13 impulse is the one reconstructed emission. `word = 0` (the located
client dispatch wrote zero), `vector` is the authority's own accepted unit
direction with `w = 0`, `value_a` is the proven `aux / 255` charge, `value_b`
is the accepted power, `action` is `0` shot / `1` pass (the located switch
groups `1` against `0/2/3`), and the unnamed tail float is `0.0`. No captured
sub-13 exists (0 of 2978 records) and the located client dispatch of that slot
passes want-to-send = 0, so the direction of travel and these labels stay
inference until a capture settles them.

Fail-closed refusals: sub 1 `UpdatePlayerPos` and sub 4 `UpdateSoccerBall`
carry opaque trailing blocks that vary per captured record (checked in the
capture corpus: 43 distinct sub-4 tails, 45-byte sub-1 tails that track motion),
so the service refuses to author them instead of inventing bytes; the
unrecovered game-state (5), config (7), team colour (8), pickup (9/10), client
relay (14/16), statistics (15/19/21), high-arc (18), bounce (25) and `Unknown`
families are refused for the same reason.

## Validation coverage

Ownership, replay, time and finite values are enforced by composition:
`sender == payload player`; strictly increasing per-player transport sequences
(replays refused with the snapshot unchanged); the gateway clock clamped to the
authority's own server time so a forward spike grants no budget; bounded `dt`,
finite positions/directions/ball geometry; possession required for a kick;
verified teammate position for a pass; and no client packet can set the score,
the goal or a reward.

## Missing proof: start/result body

The capture reaches neither half end nor a result: the last recorded soccer
frame is 9821 and the last clock sample is frame 9818, `remaining = 151.412`,
`elapsed = 65.70` inside the post-goal hold (the only goal is frame 9507,
`(0, 1, 1)`). There is therefore **no captured match-completion, result or
reward body**, and no recovered start body beyond the captured kickoff clock
(`0.0 -> 180.0` at frame 6424, already modelled by `soccer_match`). This slice
delivers the verified loop seam instead of fabricating completion:
kickoff -> play -> goal -> `Restart` -> `start()` -> clock. `Outcome.reward`
stays `Evidence::Missing` and `emit_packet` refuses the unrecovered result
families. The exact missing proofs for the lead are (a) a captured result or
reward packet, (b) a captured start body beyond the kickoff clock, and (c) a
captured sub-13 impulse that would settle the reconstructed field mapping.

## Checks

| Check | Command (cwd `prototype/osfr-tools`) | Result | Log |
| --- | --- | --- | --- |
| RED before the module existed | `..\..\scripts\cargo.ps1 test --test soccer_service` | `couldn't find file tests\..\src\soccer_service.rs` | `../lab/reports/soccer-service-repair-20261009/red.log` |
| GREEN service target | same | 13 passed, 0 failed, 1 ignored | `green.log` |
| Captured entry seam | `... test --test soccer_service -- --ignored` with `OSFR_SOCCER_ENTRY_CAPTURE` | 1 passed: captured frame decodes, re-encodes byte-exact, equals `entry_frame()` | `green-capture.log` |
| Existing soccer suite | `... test --test soccer --test soccer_match --test soccer_native_wire --test soccer_review` | 11+1 ignored / 27 / 8 / 10, 0 failed | `regression-existing-soccer.log` |
| Frozen adapter regression | `... test --manifest-path ..\..\..\lab\reports\soccer-state-adapter-20261009T174320Z\Cargo.toml` | 14 passed, 0 failed against the modified authority | `frozen-adapter-regression.log` |
| In-crate registration | `... check --manifest-path ..\..\..\lab\tmp\soccer-registration-check\Cargo.toml` | exit 0: the module type-checks as an in-crate module with `crate::` paths | `registration-check.log` |
| Formatting | `rustfmt --edition 2024 --check src/soccer_service.rs tests/soccer_service.rs` | exit 0 | session |

## Limits

- No native playability, world entry, gateway wiring or live client run; the
  module is unregistered and the test includes it by path. The lead owns
  `lib.rs` registration and native acceptance.
- The impulse emission is reconstructed policy, not captured behavior.
- Pitch dimensions, physics tuning, the kick power scale and the zero-charge
  floor stay reconstructed and caller-owned.
- No economic value exists anywhere in this slice; rewards remain `Missing`.
