# 053 Aggro and leash

Status: ready for review, 2026-10-09 05:10 UTC. This piece ships one typed aggro/leash state machine,
the recovered eligibility facts it consumes, and a fail-closed checker over the preserved native
pursuit capture. It is not a playable claim: the original server's AI is gone, the numbers below are
reconstructed, and the leash return has never appeared on the wire.

Owned paths: `prototype/osfr-tools/src/restoration/piece_053.rs`,
`prototype/osfr-tools/tests/piece_053.rs`, `scripts/pieces/piece_053.py`,
`tests/pieces/test_piece_053.py`, this note and `lab/reports/piece-053-20261009T051057Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Target eligibility is native and client-side, not a server table. The preserved executable's
Tab-target selector `00a2ac20` (reached from the native input dispatcher `00b02880`, default action
39 `TabTargetSelect` = `0xff09`) walks nearby actors and requires all of:

| Condition | Evidence in `lab/reports/native-combat-controls.json` |
| --- | --- |
| hostile `Disposition` | `*(int *)(*(int *)(iVar9 + 0x40) + 0x178) == 0` |
| not dead | `(*(uint *)(... + 0x6f8) >> 7 & 1) == 0` |
| facing toward the target | dot product compared against `DAT_01762a30` |
| line of sight | ray call `FUN_00fd8e20` with `AppPhysics::RayHit` results |

The `Disposition` field itself is on the `AddNpc` wire record: opcode 2,
`lab/references/Sanctuary/src/Sanctuary.Packet/BasePlayerUpdatePacket/PlayerUpdatePacketAddNpc.cs`
(pinned Sanctuary revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`) documents `0` hostile,
`1` neutral, `2` ally, and the native selector independently confirms `0` is the hostile value.
Community source only: no community code was copied.

The wire messages a pursuit emits are the ones the registered encounter module already sends:
opcode 125 movement with a horizontal X/Z rotation, and opcode 35 sub 8 animation callbacks
(`notes/Design/Opening Playthrough.md`, `notes/Research/Stage7 Wolf Visual Identity.md`).

## What the preserved capture proves

`lab/runs/client-baseline-20261008T183551Z/analysis-live/native-stage8-watch-012989.json`
(SHA256 `eb96e220d255dc5ffc3da9e6130ccb90c49f425bd092e14717bffe88db8749af`) is the wire watch from
the second Gloam attempt. It is evidence about *our* server's messages as the original client
rendered them, so it constrains the reconstruction without being an original rule source:

| Observation | Value |
| --- | --- |
| recorded attacks in the watch window | 44 |
| closest / farthest observed attack distance | 2.0086 / 2.4000 world units |
| animation ids emitted by the watched actor | 1 stand, 3 run, 1001 attack |
| player knockouts | 2, each resetting the player to the recorded 1000 max health |
| actor spawn anchor vs last wire position | `[-2173.0,-31.5,348.0]` vs `[-2163.85,-49.66,304.96]` |

The capture therefore supports the reconstructed melee reach (every attack inside 2.5) and the
pursuit/stop animation handshake. It does **not** show a return to the spawn anchor, so the leash
return stays reconstructed. The checker fails with `LeashReturnEvidenceChanged` if a later capture
shows the actor back at its anchor, which forces this note and the module policy to be reviewed
instead of silently claiming the gap was closed.

## Community-recovered data and conflicts

`lab/references/Sanctuary` (AGPL-3.0, source-only research) exposes the same `Disposition` field on
`PlayerUpdatePacketAddNpc` and, in `Entities/Npc.cs`, an NPC `Speed` default of `6.25` with
`MovementType` 2 and per-NPC `InteractRange`. Sanctuary implements no server-side aggro at all: its
NPCs move only when a Lua/script command calls `MoveTo`. The pinned wiki corpus
(`lab/references/freerealms-wiki/wiki.sqlite`) carries taunt wording on Brawler abilities
("taunt all nearby opponents") but no aggro, leash or threat numbers, so no threat table is
implemented here.

Unresolved conflict, recorded rather than smoothed: Sanctuary's default NPC `Speed` is 6.25 while the
registered reconstruction walks pursuers at 3.0. The capture's frame counter is not a proven wall
clock, so it cannot arbitrate the two; both numbers stay labelled and neither is presented as
original.

## Reconstruction boundary

Everything numeric in the module is a labelled reconstruction that mirrors the registered
`encounter.rs` loop so integration is a behaviour-preserving move plus one deliberate addition:

1. a hostile actor acquires the **nearest** eligible player (deterministic, lowest guid on a tie);
2. non-finite positions, dead players, targets in another zone/instance tag and targets beyond
   40.0 units are rejected with typed reasons; exactly 40.0 is inside the band;
3. pursuit walks at 3.0 units per second, clamps one tick to 250 ms, ignores ticks closer than
   100 ms and stops at a 2.4 stop margin so f32 rounding cannot strand a pursuer outside the
   2.5 melee reach (2.5 is inside the band, 2.501 is not);
4. on range/death/zone/`TargetGone` the actor emits a typed `Disengaged` event, returns to its
   anchor at the same speed, snaps onto it and only then re-acquires.

Item 4 is new relative to production: `encounter.rs` currently freezes a pursuer outside 40 units
and never walks it home. The lead owns that integration decision and any native acceptance.

Movement, animation, attack and damage messages are deliberately absent from the module; the owning
subsystem keeps serialization, the 1 s enemy attack cadence and rewards.

## Explicit gaps

* The original server's aggro radius, speed, attack cadence and leash rule were not recovered; the
  original server binaries are not preserved.
* No preserved capture shows a return-to-anchor, so the leash half of the acceptance has server
  tests only.
* Threat/taunt ordering is unrecovered; the module has one target at a time and no threat table.
* Facing and line-of-sight stay native client selection; the server-side module consumes a
  caller-supplied candidate view instead of re-deriving them.
* Per-NPC spawn anchors and zone membership beyond the two registered enemy sets (world stage and
  dungeon) are not modelled; the caller supplies the zone tag.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_053
  RED  before implementation: error: couldn't find file `tests\..\src\restoration\piece_053.rs`
  GREEN: 12 passed; 0 failed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_053.py' --output <report>
  GREEN: exit 0, disposition 0/1/2, 44 observed attacks inside 2.5 (farthest 2.4000)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_053.py' -v
  GREEN: 11 passed (Ran 11 tests, OK)
```

Integration gate for the lead: register `pub mod piece_053;`, decide whether production adopts the
return-to-anchor policy, then re-run the encounter suite and a native pursuit/reset check.
