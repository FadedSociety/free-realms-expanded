# 052 Enemy patrol and idle

Status: ready for review, 2026-10-09 05:20 UTC. This piece recovers the **NPC movement and animation
wire contract** and adds a **bounded, reconstructed patrol controller** plus a **native replay
checker** for the real idle/run transitions. It is code plus evidence, not a playable claim, and it
does not invent patrol routes, speeds, dwell times or rewards.

Owned paths: `prototype/osfr-tools/src/restoration/piece_052.rs`,
`prototype/osfr-tools/tests/piece_052.rs`, `scripts/pieces/piece_052.py`,
`tests/pieces/test_piece_052.py`, this note and
`lab/reports/piece-052-20261009T052000Z/`. No shared `lib.rs`, gameplay, gateway, Cargo manifest,
launcher, content, state or native UI file was touched.

## Recovered original facts

Movement. The standalone application message is opcode `125`:
`u16 opcode, u64 guid, 3f position, 3f rotation, u8 state, u8 unknown` = 36 bytes. Native reader
`008e5940` (file offset 1030083) consumes exactly those fields, initializes rotation W = 0 and has
**no timestamp field**. The three rotation floats are a **direction vector**, not a truncated
quaternion: native `0095dae0` masks with `[1,0,1,0]` (a nonzero horizontal X/Z length is required),
normalizes XYZ and derives yaw with `atan2(X, Z)`. An all-zero or vertical-only heading is ignored by
the setter. Community NPC movement uses State `1` / Unknown `0`; State bit 0 set passes `0.0` as the
speed parameter, clear calls the entity speed getter `008b3180`. These functions, the preserved
`native-motion-verification.json` record and the 13 compiled-serializer fixtures are re-verified by
the checker rather than restated.

Animation. The message is opcode `35` subtype `8`:
`u16 opcode, u16 subtype, u64 guid, i32 animationId, i32 unknown, u8 flags` = 21 bytes (native reader
`00908a50`; subtype-8 dispatch at 1093435). Flags bit 0 set writes the entity base/idle animation;
clear plays it immediately. The shipped `Resources/CodeAnimationMappings.txt` maps data ids
`1=loc_stand`, `2=loc_walk`, `3=loc_run`, `1000=com_h2h_attack`, `1001=com_h2h_attack_01`.

Spawn fields. The 334-byte AddNpc template carries `Speed` (f32) at 128, `WalkAnimId`/`RunAnimId`/
`StandAnimId` at 137/141/145, `IsInteractable` at 224 and `MovementType` at 233. The pinned
community writer's field and write order (`Speed, WalkAnimId, RunAnimId, StandAnimId, MovementType`)
matches, and the local spawn template
(`references/opening-anvil-placement-v1/content/encounter-template.bin`, SHA-256
`602c66b1a4ea9fac97eaec2d64c380bee17c33ff86e0aa9fe0b10c1da3bbfc19`) still holds `0.0`,
`(-1,-1,-1)`, `1`, `2`. The 13 oracle fixtures re-derive those offsets independently: each mutation
fixture (`npc-speed3`, `npc-physics2`, `npc-locomotion`) differs from `npc-zero` only at the byte
offsets its manifest declares.

Model bindings. The pinned community ADR reader (`fr-adr-toolkit`, revision
`5acfdf8d6006fa6f43d1d3605f076a6be7393e72`) resolves `loc_stand`/`loc_walk`/`loc_run`/
`com_h2h_attack_01` for `wolf_generic.adr` (`wolf_*.gr2`) and `gloamling_m_default.adr`
(`gloamling_*.gr2`). Those are the only two actors this piece checks.

## Reconstructed policy (labelled, not historical)

No recovered content carries per-actor patrol waypoints or dwell times, so `piece_052.rs` validates a
caller-supplied route instead of shipping one. Bounds: 2-16 waypoints, every leg (including the
closing leg) has a nonzero horizontal direction and a planar length greater than the arrival radius,
route extent at most 200 m, speed `0 < v <= 6.25` (the ceiling is the pinned community `Npc.Speed`
default, community evidence rather than first-party), arrival radius at most 5 m, dwell at most 10 s.
The controller updates at most every 100 ms and simulates at most 250 ms per call, so a stalled or
jumping clock cannot teleport an actor; it never overshoots past the arrival radius, so the next
update arrives instead of oscillating. Each update emits the recovered movement message and, on the
locomotion change, the base run (`3`, flags 1) or base stand (`1`, flags 1) animation, which is the
idle/run transition shape the original client plays. It refuses guid 0 and guid 1 (the player).

## Native evidence (wire, sampled)

`scripts/pieces/piece_052.py --run <capture>` decodes and reassembles a preserved capture with the
shared CRC/sequence algorithm and verifies every opcode 125 / 35-8 message in it. On
`client-baseline-20261008T075754Z` (the retained combat-motion retest): 3299 gateway frames, 3297
decoded, all CRCs and reassembly complete; **256** server-to-client movement messages over four enemy
guids (116/72/46/22) plus **2** client-to-server player movement messages; every movement message is
36 bytes with Unknown 0 and a unit horizontal heading with Y = 0. **100** animation messages:
`{3: 8 run-base, 1: 8 stand-base, 1001: 84 attack}`. Per-enemy idle/run transitions are retained with
frames and capture times, e.g. guid `5715144128442986753` run-base at frame 1792, stand-base at 1855
(187.766 s), then a second run at 2601 and stand at 2657 after the retry; all four enemies return to
their base stand animation. Cross-check on `client-baseline-20261008T193318Z`: 133 movement and 34
animation messages with the same transitions, and its client-authored headings are slightly
unnormalized (`0.99947`), which the checker records instead of rejecting because native `0095dae0`
normalizes XYZ itself and only requires a nonzero horizontal X/Z.

This is wire delivery, not rendered playback. The pixel evidence for wolves running/lunging already
exists in [[../Reviews/Native Outdoor Wolf Controls 183551]] and covers **chase** motion for those
actors; it does not cover a patrol loop, does not synchronise poses to packets, and does not establish
collision, terrain support or per-model clip coverage. Those remain gaps.

## Checks

```
cd prototype/osfr-tools
& '..\..\scripts\cargo.ps1' test --test piece_052      # 8 passed, 0 failed

cd ../..
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_052.py" -v   # 15 passed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_052.py'   # writes the lab bundle
```

The Rust target was red first (`couldn't find file tests\..\src\restoration\piece_052.rs`) before the
module existed; the Python target was red first (`FileNotFoundError:
.../scripts/pieces/piece_052.py`).

## Integration and remaining gates

Lead integration: register `mod piece_052;` (and the `restoration` module) in `src/lib.rs`, drop the
`#[path]` include from the test, and let the encounter module construct routes from content. The
module exposes the packet encoders, the recovered animation constants and `locomotion_animation_ids()`
for the spawn fields; it owns no content or economy.

Remaining gates: (1) lead-native idle/run acceptance for a **patrol** actor, not a chase, using the
replay checker on the new capture; (2) route/dwell data must come from recovered content or be
labelled reconstructed in content, never silently shipped; (3) per-model animation coverage beyond
wolf and Gloamling; (4) collision/terrain support for patrolled legs.

Superseded evidence: `lab/reports/piece-052-20261009T051601Z/` was written before the checker stopped
requiring exactly unit client headings; use `piece-052-20261009T052000Z/` (default run) and
`piece-052-20261009T051800Z/` (cross-check run).
