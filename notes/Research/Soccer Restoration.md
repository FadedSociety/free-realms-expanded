# Soccer Restoration

## Scope

Bounded, evidence-backed Rust slice for the original soccer minigame
(`BaseSoccerPacket = 76`): gateway envelope, the 21 captured packet payloads,
an authoritative message-driven match state machine (session, kickoff clock,
goal, restart hold, possession) and a labeled reward proposal. No ball
physics, no simulated win, no shared gateway/lib wiring. Root integration and
native UI remain separate work.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Original capture (pcapng) | `lab/references/OpenSourceFreeRealms/ofrserver/Packets/packets_soccer_minigame.pcapng` | `9e609bb526fbefb7aa00164afc694ddd4d27680e55477492127012c8ff493ee8` |
| Reassembled packets | `lab/reports/original-minigame-packets/soccer-*.bin` (2978 records) | see `soccer-application-index.json` |
| Packet id enum | `lab/decompiled/server/LibSOE/SOE/BaseSoccerPackets.cs` | original server build |
| Tunnel opcode | `ofrserver/Server/LibSOE/Core/PacketCodes.cs` (`BaseSoccerPacket = 76`) | pinned emulator `ace6a82` |
| Envelope convention | `prototype/osfr-tools/src/gateway.rs::tunnel` (u16 outer 5\|6, bool, u32 size) | existing project proof |
| Scoreboard columns | `lab/decompiled/client-ui/score-screen/scripts/__Packages/scorescreen/states/SoccerState.as` | original client UI |
| Team field names | `.../scorescreen/soccer/Team.as` (Name, Color, Score, Shots/Knockdowns/Pickups/Steals per half) | original client UI |
| Minigame content | `lab/reports/minigame-content-inventory.json` records 42/43/44 (Sacred Grove/Snowhill/Briarwood Soccer, type 12) | extracted content table |
| Job context | `lab/references/freerealms-wiki/wiki.sqlite`, page "Soccer Star" (Kick Power/Tackle Power/Toughness, Goal Time update 2009-10-02) | read-only wiki DB |
| Zone transition | `lab/reports/soccer-zone-transition.json` (`sg_soccer`, geometry 107) | typed community zoning reader |

Decoders (new, this task):

- `lab/reports/soccer-20261009T0203Z/decode_soccer_captures.py` - parses every
  `soccer-*.bin` envelope and payload, joins the application index, writes
  `soccer_payloads.json` (2978 records).
- `lab/reports/soccer-20261009T0203Z/dump_soccer_fields.py` - aligned field
  interpretations (`field_dump.txt`).

## Recovered protocol

Envelope (byte-exact on all 2978 captures):

```
u16 outer (5 or 6) | u8 flag (0/1) | u32 inner length | u16 0x004C (76) | u16 soccer id | payload
```

`inner length` counts `u16 soccer id + payload`. `flag` is opaque and preserved
(0 on the captured player-position direction, 1 elsewhere).

Traffic direction, independently re-verified by
`lab/reports/soccer-review-20261008T1942Z/verify_soccer_capture.py`: flow 0 is
the remote game server `64.37.170.6:20267 -> 192.168.1.145:61759`; flow 1 is
the client's own traffic. Goal, clock, ball, team, pickup, stats and config
messages appear on flow 0 only, while the client only ever sends player
position (1), registration (3), player state (14) and movement controls (16).
The original server is the sole soccer authority; client packets are requests.
The single captured half contains, per soccer id:

| id | name (BaseSoccerPackets) | n | payload (original evidence) |
| --- | --- | --- | --- |
| 1 | UpdatePlayerPos | 1191 | `u64 player, f32 xyz, 45B orientation/velocity tail` (tail constant at spawn) |
| 2 | GoalMade | 1 | `u32[3]` = (0, 1, 1); the slice treats it as the post-goal scoreboard (field3 == field1+field2) |
| 3 | RegisterPlayer | 8 | `u64 player, 2 marker bytes (00 01)` |
| 4 | UpdateSoccerBall | 83 | row-major `f32[16]` transform (translation row 3), `f32[3]` velocity, 15B tail |
| 5 | UpdateGameState | 17 | `u32[18]`, `-1` sentinels pre-kickoff, reset pattern at the goal; kept opaque |
| 6 | SetPlayerTeam | 8 | `u64 player, u32 team, u32 slot` - host+0x10+0x20+0x30 team 0 slots 0-3, 0x40-0x70 team 1 slots 0-3 |
| 7 | SetClientConfig | 1 | 221B arena config; ball asset length u32 @124, `"sg_soccerball_bbe.adr"` @128, ball spawn f32[4] @157 = (226.7, 2.0, 212.905, 1.0) |
| 8 | TeamColours | 1 | len+`"sweetpea"`, len+`"ocean"`, RGBA A `(255,100,35,0)`, RGBA B `(15,50,255,0)`, ids 411073/411070, 1B |
| 9 | SpawnPickUp | 8 | `u8 state, u32 definition(0x354/0x355), u8 marker, f32 xyz, f32 scale(1.0), u32 instance` |
| 10 | AcquirePickUp | 2 | `u8 flag, f32 20.0, u64 player` |
| 11 | AcquireBall | 5 | `u8 flag, u64 holder, u64 previous` |
| 12 | FreeBall | 5 | `u8 flag, f32 xyz, f32 scale(1.0), u64 previous` |
| 14 | UpdatePlayerState | 250 | `u64 player, u32 state, 34B body, f32 xyz @46, f32 1.0` |
| 15 | UpdatePlayerStats | 4 | `u64 player, u32[5]` |
| 16 | UpdatePlayerControls | 1216 | `u64 player, f32[3]` movement direction (e.g. (0,-1,0)) |
| 18 | HighArcBall | 1 | `f32[4], 5B` |
| 19 | PlayerVariableStats | 10 | `u64 player, u32[9]` |
| 21 | SetSoccerPlayerOneTimeStats | 8 | `u64 player, f32[42]` per-player tuning (Kick/Tackle/Toughness family; names not proven) |
| 23 | UpdateGameTime | 78 | `f32 remaining, f32 elapsed`; 0.0 pre-kickoff, 180.0 at kickoff, countdown to 151.412, frozen after the goal |
| 25 | BallBounce | 20 | `u32[2]` = (0,1) burst at frames 8915-9088 |

Not captured in this session: ids 13 ImpulseBall, 17 StatRollInfo, 22
SpecialAnimTime, 24 SendAIInfo, and no second half, completion, reward or
return traffic.

Session facts from the capture: 4v4 roster, kickoff clock `180.0 s`, one goal
(score 0-1), possession chain `0x40 -> 0x30 -> 0x20 -> 0x40 -> 0x60` with
`FreeBall` clearing the holder, and the clock held at `151.412` for frames
9503-9818 after the goal.

## Implementation

`prototype/osfr-tools/src/soccer.rs`

- `decode` / `encode`: byte-exact envelope and payload codec for the packet ids
  above, unknown ids carried as `Unknown { sub, payload }`.
- `Match`: message-driven state with an explicit `Direction`. Only
  `ServerToClient` playback mutates state. `ClientToServer` messages with a
  captured client packet id are decoded and returned but never change the
  match; a client message carrying a server-only packet id (for example a
  spoofed `GoalMade`) is refused, so untrusted bytes cannot score. Session from
  `TeamColours`/`SetPlayerTeam`/`RegisterPlayer`; clock from `UpdateGameTime`
  (`Lobby -> Running -> Restart -> Finished`); goal from `GoalMade` only while
  `Running` (the restart hold is not a scoring window) with the scoreboard
  consistency check (`fields[2] == fields[0] + fields[1]`, advance by exactly
  one); possession from `AcquireBall`/`FreeBall` with the previous-holder chain
  check plus a registered-holder gate; ball position only from finite
  `UpdateSoccerBall`/`FreeBall` geometry - no physics step exists, and the
  reducer never claims physical authority.
- `restart()`: reconstructed resume that emits the captured `UpdateGameTime`
  shape with the held clock (`Evidence::Reconstructed`).
- `reward_proposal()`: reconstructed record (`minigame_id = 42`, zone
  `sg_soccer`, score, winner, teams, client scoreboard columns); `reward_table`
  stays `Evidence::Missing` because no completion or reward traffic exists in
  the capture.

Interface summary: `decode(&[u8]) -> io::Result<Message>`,
`encode(&Message) -> Vec<u8>`,
`Match::{new, handle(Direction, &[u8]), restart, reward_proposal, session,
phase, clock, score, possession, ball_position, goals, player_team}`,
`Direction::{ServerToClient, ClientToServer}`, `CLIENT_PACKET_IDS`
(1/3/14/16, the ids the capture shows the client sending) and
`Evidence::{Original, Reconstructed, Missing}`.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| RED (test first) | `scripts/cargo.ps1 test --test soccer --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | failed to compile: `couldn't find file tests\..\src\soccer.rs`, exit 101 - log `lab/reports/soccer-20261009T0203Z/red-test.log` |
| GREEN | same command after the module | `10 passed; 0 failed`, exit 0 - log `lab/reports/soccer-20261009T0203Z/green-test.log` |
| GREEN after the direction repair | same command | `11 passed; 0 failed` (adds `client_direction_cannot_change_authoritative_state`) - log `lab/reports/soccer-review-20261008T1942Z/postfix-soccer.log` |
| Authority review RED -> GREEN | `scripts/cargo.ps1 test --test soccer_review ... -- --ignored` before the fix: `4 failed`; after the fix the same probes are ordinary guard tests: `10 passed; 0 failed; 0 ignored` | logs `lab/reports/soccer-review-20261008T1942Z/red-baseline-4probes.log`, `postfix-soccer-review.log` |
| Format | `rustfmt --edition 2024 src/soccer.rs tests/soccer.rs tests/soccer_review.rs` (no `cargo fmt`, no other agents' files) | clean |
| Fidelity | all 37 embedded capture entries (36 distinct messages, plus 1 synthetic unknown-opcode) decode -> encode byte-identically, including the `flag = 0` sub-1 message; the independent verifier also confirms every capture vector is an unmodified reassembled message | `envelope_round_trips_captured_messages_byte_exact`; `verify_soccer_capture.py` 21/21 - log `postfix-capture-verify.log` |

Tests (`prototype/osfr-tools/tests/soccer.rs`): 4v4 sweetpea/ocean session,
180 s kickoff clock, goal -> score 0-1 -> clock hold -> restart, possession
chain with FreeBall, ball transform/control decode, config asset/spawn, byte
round trip, unknown opcode passthrough, no-physics authority, timeout ->
finished + labeled proposal, and the client-direction guard (client requests
never mutate state; a client goal is refused while the same captured goal is
accepted from the server). The independent review harness
`prototype/osfr-tools/tests/soccer_review.rs` keeps the four promoted guard
tests plus clock, possession, team and decode checks.

## Artifacts

| Artifact | sha256 |
| --- | --- |
| `prototype/osfr-tools/src/soccer.rs` | `2c18c712a12b9fc003c0529a7146c8185f6ca80fff19fa24273d3e654b27e2d9` (pre-repair `a177ea48…`) |
| `prototype/osfr-tools/tests/soccer.rs` | `ee75c3841ae1eaf518db35ec33e5a9a85eaa89d9f8ae389727969c401516e8bc` (pre-repair `85ac58f2…`) |
| `prototype/osfr-tools/tests/soccer_review.rs` | `eb0d6b1831a163bc60932ae126097fc5f27cd889f0e560e5c805b7c28f9629f3` |
| `lab/reports/soccer-20261009T0203Z/decode_soccer_captures.py` | `123c0cb630a61eb7e523115207946ef279ad25f67a478e42530778a68066f893` |
| `lab/reports/soccer-20261009T0203Z/dump_soccer_fields.py` | `e8059ae0b86b15e97c16b0da5b0b04e3c6c2a8afa91cc465cc8ad0c60de1cbfa` |
| `lab/reports/soccer-20261009T0203Z/soccer_payloads.json` | `abf8010ffe74c6965ae4c6917de3e1b4fbdf86c1f61b42312bd83f77a2fb4fc6` |
| `lab/reports/soccer-20261009T0203Z/red-test.log` | `4a0354a75755b3a3acc814502073c39da76c33ae357b456862dc978c951b43ad` |
| `lab/reports/soccer-20261009T0203Z/green-test.log` | `1534b6859e0d7c11933cda8f3b2621aec4bc69b84dd1599c777afdaa563cf1ce` |
| `lab/reports/soccer-review-20261008T1942Z/verify_soccer_capture.py` | `37a73e96c53788e56b142413b998c6216ff44c8a0071e0c361f111d958f707f9` |

## Limitations and next acceptance

- No compiled original soccer serializer exists to diff against; the packet ids
  come from the preserved `BaseSoccerPackets` enum and the byte shapes from the
  capture itself. There is no community payload serializer for soccer in the
  pinned `Sanctuary`/`FreeRealms-Legacy` sources.
- Independent review found and the repair fixed four authority gaps: direction
  gating (client bytes can no longer score or move state), goals during the
  restart hold, non-finite ball geometry, and possession by unregistered
  players. Evidence: `notes/Reviews/Soccer Independent Review.md`,
  `lab/reports/soccer-review-20261008T1942Z/` (RED logs before, GREEN after).
- `GoalMade` field semantics are inferred from the single captured goal plus
  the scoreboard invariant; the last pre-goal holder being a team-1 player
  corroborates the 0-1 direction, but a second goal or a loss would still
  falsify the label.
- The direction gate is an API contract: `handle` requires the caller to state
  where the bytes came from. The gateway integration must pass server traffic
  as `ServerToClient` and client traffic as `ClientToServer`; the reducer
  itself cannot see the UDP tuple.
- `UpdateGameState` (id 5) stays opaque: 17 samples are not enough to name its
  18 words. No packet is synthesized from it.
- The completion/reward/return messages were never captured, so the reward
  proposal is a record shape with `Evidence::Missing` amounts; the tutorial
  flow is also absent from this capture (the wiki documents it exists at the
  Snowhill field entrance).
- Next acceptance: capture or recover a full match through completion, then
  replace the reconstructed restart/resume and filled reward row with original
  evidence; wire `Match` into the gateway from the root integration path,
  passing only server traffic as `ServerToClient` and client traffic as
  `ClientToServer`.
