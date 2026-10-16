---
type: research-note
topic: Free Realms Demo Derby server candidate
updated: 2026-10-13
---

# Derby Service

Status: client-shaped server candidate; focused tests pass. This is not native-client
acceptance, historical server recovery, or gateway integration. The earlier
[[Derby Restoration]] note describes capture-derived claims and remains quarantined; none of its
frame counts, lengths, or values informed this implementation. See
[[Build Guides/Gameplay/Demo Derby]] for the reviewed client evidence and the complete field map.

## Files and scope

- `prototype/osfr-tools/src/derby.rs` implements typed inner-opcode 34 layouts, a bounded
  match service, state progression, pose relay, server-owned combat and pickup effects,
  announcer messages, result state, and once-only reward application on Close.
- `prototype/osfr-tools/tests/derby.rs` replaces capture-based tests with synthetic fixtures
  and checks all 20 body layouts plus the solo/NPC lifecycle, combat, pickups, relay,
  disconnect, and reward-once behavior.
- `src/lib.rs` already declares `pub mod derby;`; no shared routing or gateway file was changed.

The service is a module candidate. It does not itself bind authenticated connections, set the
derby status bit, select arena/vehicle content, or register a gateway route. The outer service
must supply authenticated `ParticipantKey`s and route the resulting `DerbyAction`s.

## Client facts used

Every schema below is grounded in the accepted native map and the client readers/senders in
[[Build Guides/Gameplay/Demo Derby]]. The module records these addresses in `SUBTYPE_SPECS`
and on the matching message variants. Inner opcode 34 and the two-u16 header come from
`009e9f80` and `009ee770`. Directions and names come from the guide's constructors, dispatcher
`009f39b0`, and sender call sites. Unknown means the body bytes are retained without assigning
an unsupported meaning.

| Subtype | Client-observed body shape | Reader/sender provenance |
|---|---|---|
| 1 | i32 state | reader `009ef570`, dispatcher `009f39b0` |
| 2 | u64 key, i32 phase (C>S) | writer `009f0a50`, sender `009f1440` |
| 3 / 4 | i32 player / NPC count | readers `009ef330` / `009ef370` |
| 5 | 128 bytes: key, 16 transform floats, two float4s, three floats, four bytes, u64 | reader `009ef5e0`, serializer `009f0ce0` |
| 6 | two u64 values (C>S); only first is established as local key | sender `009f1790`, activation `009f1b80` |
| 7 | u64 key, u32 countdown | reader `009eedb0`, sender `009f20f0` |
| 8 / 9 | one u64 key each | dispatcher `009f39b0`; corrected reader-size findings `009efc50`, `009efbe0`, `009efe80` |
| 10 | key, length-prefixed name, opaque vehicle definition, four float4s, bool byte, float | reader `009f3800`, vehicle reader `009d7b50` |
| 11 | 51 bytes: key, six u32s, three bytes, four floats | reader `009ee850`, dispatcher `009f39b0` |
| 12 | two u32s, float4, two floats | reader `009ee9e0` |
| 13 | one u32 pickup id | dispatcher `009f39b0` |
| 14 | key, float4 position (C>S) | sender `009f1550` |
| 15 | key, u32 kind, three floats | reader `009eec60` |
| 16 | two keys, impact float | reader `009eeba0`, sender `009f22c0` via `009f1680` |
| 17 | one u32 category | reader `009ef4d0` (speculative label) |
| 18 | one u64 key | reader `009ef520` |
| 19 | f32 plus u32; meanings unknown | reader `009ee190` (likely label) |
| 20 | two u32s and two keys | reader `009eeeb0` |

Additional client facts: local registration is sent by `SetActive` (`009f1b80`); the derby
processor is `009f39b0`; the client accepts states 3 through 11; state 10 opens Derby Results,
and state 11 enables exit/Close. Client text describes knock-out scoring (2 points, or 3 for
the leader), a point loss on wreck, four-second recovery, and five power-up categories. Client
code reads the health/score/result fields from 34/11, but the exact meaning of multiple fields
is still unknown. The client sends contact reports (34/16), not damage awards. See the guide
for exact field offsets, confidence labels, and client behavior.

The state gate predicates are from pseudocode at `009f2d90` and its helpers. States 3 to 8
emit 34/2 after these local gates; states 9 to 11 emit no 34/2:

| Server state | Client gate / action | Native provenance |
|---|---|---|
| 3 | Count of drivers with flag `+0xf00` equals subtype-3 count; acknowledge phase 3 | `009ed680` |
| 4 | Count without that flag equals subtype-4 count; acknowledge phase 4 | `009ed6b0` |
| 5 | Sound elements and vehicle physics are ready; set per-driver round flag, acknowledge 5 | `009ed3d0`, `009ed5a0`, `009ed610`, `009ed640` |
| 6 | Local threshold check passes; set global byte and acknowledge 6 | `009eb380` |
| 7 | Internal flag is set; initialize countdown to 4 and acknowledge 7 | `009f2d90` |
| 8 | Pending byte `+0xcf` is clear; show/reset countdown and acknowledge 8 | `009f2d90` |
| 9 | No wait; hide countdown and start live HUD/timer | `009f2d90` |
| 10 | No wait; show results (only server state 10 ends the derby) | `009f2d90` |
| 11 | No wait; enable the Score Screen exit button | `009f2d90` |

## Reconstructed service policy (R)

These are explicit implementation choices, not recovered server behavior:

- Up to eight drivers; fill empty slots with deterministic NPC keys. Bots have no client
  session and this candidate supplies no NPC driving AI.
- The native type-11 minigame offer/start uses common opcode 39 lifecycle packets. The exact
  unnamed generic defaults, authenticated-key binding, outer entry/leave behavior, and
  reward semantics have not been accepted against a client.
- Registration emits config, matching human/NPC counts, and each create-vehicle record before
  state 3. The service advances states 3 through 8 from the acknowledgements the client emits
  after its documented readiness gates. State 9 has no client acknowledgement: the service
  enters Running when the coordinated countdown reaches zero. State 10 has no client
  acknowledgement either; the service sends state 11 after its configured 10-second results
  hold (R). The original server's countdown coordination and results hold duration are unknown.
- Pose updates are relayed to other connected humans. The service echoes contact records to
  peers and treats the first key as attacker, although the client does not establish the
  server's relay policy or attacker/victim roles.
- Damage scale, contact cooldown, health maximum, shield blocking, attack/ultimate multipliers,
  score updates, leader tie handling, four-second respawn timing, and score/time end conditions
  are configured/reconstructed. Knockouts emit state, announcer, and reset messages.
- A pickup request selects the nearest configured pickup inside the configured radius; it is
  consumed once. Types 1 to 5 map to health, boost, shield, attack-up, and ultimate based on the
  client sound-event ordering. Effect amounts/durations and pickup selection radius are R.
- Default implementation values (all R): eight drivers, 100 health, one damage per impact
  unit with a 500 ms pair cooldown, 3 m pickup radius, +25 health/boost, 5-second timed
  effects, 2x attack/ultimate multiplier, 4-second respawn, knockout points 2/3 and one-point
  wreck loss. The configured results hold defaults to 10 seconds. Tests use smaller synthetic
  health/reward fixtures and a 1-second results hold.
- Results sort by score, then damage dealt minus received, then key. The service sends final
  34/11 records before state 10, holds the result phase for its configured duration, sends
  state 11, and applies the supplied reward grant once for each human Close. Sort tie-breaks,
  result duration, reward timing, and the exact server reward amount are R.

Unknown wire fields remain raw words/bytes; subtype 10's vehicle-definition body remains
opaque. Config defaults are zero because the two subtype-19 values have unknown meanings; tests
use synthetic values only. No community capture-derived defaults are retained.

## Verification

Commands run from `prototype/osfr-tools`:

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W44'
& '..\..\scripts\cargo.ps1' test --test derby
```

Result: 5 passed, 0 failed. The tests cover all 20 subtype round-trips and reader widths; one
solo match with NPC fill, client-gated states 3-8, unacknowledged states 9/10, timed state 11,
and one reward; contact damage/knockout/respawn;
all five pickups and expiry; pose recipient filtering and disconnect abort. The build reports
an existing unused `housing.rs::instance_index` warning.

```powershell
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' --edition 2024 --check 'src\derby.rs' 'tests\derby.rs'
```

Result: pass. Read-only native-map inspection used `python -I scripts/native_map/nmq.py dis`
and `show` on the listed reader/sender addresses, including `009f3800`, `009d7b50`,
`009ee850`, `009ee9e0`, `009eeba0`, `009eec60`, `009eeeb0`, `009f0ce0`, `009f1550`,
`009f1790`, `009f2d90`, and `009f39b0`. In particular, the `009f3800` reader confirms
the length-prefixed name, opaque vehicle definition, four float4s, bool, and float; the
`009ee850` reader confirms the 51-byte subtype-11 body.

An earlier focused build stopped at an unrelated in-progress `src/gateway.rs:1071` type
error (`serde_json::from_slice(...).map_err(invalid)`, E0631); no gateway edit was made here.
The requested rerun completed successfully after that shared file changed. Intermediate
errors in this candidate (trait derives and a duplicated test offer) were corrected before
the passing run.

## Open gaps

- No native client run or server capture acceptance. This verifies the implementation
  against the reviewed client layouts and synthetic fixtures only.
- No gateway route, authenticated player-key adapter, status-bit-11 carrier, arena/catalogue
  content selection, or outer minigame teardown is implemented here.
- No source establishes NPC AI, contact key roles, impact units, collision orientation,
  health/damage tuning, pickup timers/drop odds, config values, match end condition, job
  progression, reward amounts, or whether these ack barriers match the original server.
- Native client tests must confirm vehicle-definition payloads, pose/impact effects, pickup
  visuals and effects, state 10/11 leave flow, framework Close packet and reward persistence.
