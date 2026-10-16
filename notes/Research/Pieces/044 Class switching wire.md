# 044 Class switching wire

Status: ready for review, 2026-10-09 05:42 UTC. This piece recovers the class-switch
**request**, the **failure command**, the activate-feedback opcode and the preservation invariants, and
ships one strictly-typed module plus an evidence checker. It is not a playable claim: no native gate
was run and no class-switch frame exists in the preserved captures.

Owned paths: `prototype/osfr-tools/src/restoration/piece_044.rs`,
`prototype/osfr-tools/tests/piece_044.rs`, `scripts/pieces/piece_044.py`,
`tests/pieces/test_piece_044.py`, this note and `lab/reports/piece-044-20261009T044119Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Command family. The decompiled original server enum
`lab/decompiled/server/LibSOE/SOE/BasePackets.cs` pins `BaseCommandPacket = 26` and
`BaseClientUpdatePacket = 38`. Inside that family,
`LibSOE/SOE/BaseCommandPackets.cs` names `CommandPacketSetProfile = 13`,
`CommandPacketActivateProfileFailed = 26` and `CommandPacketSetProfileByItemDefinitionId = 36`; the
sibling `BaseClientUpdatePackets.cs` names `ClientUpdatePacketHitpoints = 1` and
`ClientUpdatePacketActivateProfile = 21`.

Client-side confirmation. The supplied client itself builds the same frames. In
`lab/decompiled/client/FreeRealms/pseudocode.c` the shared base constructor `FUN_008a2820` stores the
command family `0x1a` in `BasePacket` and the immediate in `BaseCommandPacket`, and
`FUN_008a28e0` builds `CommandPacketSetProfile` with `0xd`, `FUN_008a2b20` builds
`CommandPacketSetProfileByItemDefinitionId` with `0x24` and `FUN_008a2ca0` builds
`CommandPacketActivateProfileFailed` with `0x1a`. The client-update equivalent `FUN_0089d180` stores
`0x26` and takes the sub-opcode, which `ClientUpdatePacketActivateProfile` supplies as `0x15`. So a
command frame is `[26,0][sub,0][fields...]` and a client-update frame is `[38,0][sub,0][fields...]`.

Request payload. `FUN_008a2ca0` and its siblings sit beside `CommandPacketSetProfile`, whose base
class reads exactly one `int32`; the pinned community reader
`Sanctuary.Packet/BaseCommandPacket/CommandPacketSetProfile.cs` consumes one `int32` and then
requires `RemainingLength == 0`. The request is therefore `[26,0][13,0][int32 profile]`.

Failure command fields. `FUN_008a2ca0` initialises exactly two `int` fields beyond the base command
packet. Their meaning is **not** recovered.

## Community-recovered facts

`lab/references/FreeRealms-Legacy/src/Sanctuary.Packet.Common/Extensions/PacketReaderExtensions.cs`
names the same five sub-opcodes at the same values.
`Sanctuary.Gateway/Handlers/BaseCommandPacket/CommandPacketSetProfileHandler.cs` is the recovered
flow: look the requested profile up in the character's profiles, serialize the found profile into
`ClientUpdatePacketActivateProfile.Payload`, attach `GetAttachments()`, then send `Animation = 3001`
(`emo_outfit_all`) and `CompositeEffect = 4005` (`PFX_Job_Swirl`); an absent profile returns before
`ActiveProfileId` is touched. `ClientUpdatePacketHitpoints.cs` serializes exactly
`[38,0][1,0][current][max]`, and `ClientPcProfile.cs` keeps `Items` (gear) and an eight-slot
`Abilities` list per profile.

The read-only wiki records the feature as UI: the "Interface" page (revision 43135) describes "Quick
Job Change/Profile Info - Hitting the arrows will let you change your job without having to go to the
About Me menu". No wiki page describes a switch restriction, a cooldown or a failure reason.

## Reconstructed policy

Labelled as ours, not historical:

* the **refusal choice** itself - refuse a profile the caller does not own;
* the **failure payload shape** `[26,0][26,0][requested profile][0]`. The sub-opcode is original and
  the field count is original; the second field is emitted as the zero the recovered client
  constructor installs rather than as an invented reason code, because no reason vocabulary is
  recovered for this command;
* the **preservation guard** - sample health, gear and the ability-set size before and after a
  switch and fail the switch claim if any of them moved, or if the profile did not actually move.

## What this piece refuses to claim

* Which profiles a character may switch into is **caller-supplied ownership**, not a recovered rule.
  The module deliberately does not hard-code a class list, a rank gate or a level gate.
* The activate payload bytes, the attachment list and the animation are built by the existing
  `profiles`/`inventory` code. This module selects the profile and certifies what was preserved; it
  does not build the payload.
* Cooldown, in-combat and minigame restrictions on switching are unrecovered and are not invented.
* Nothing here grants money, items or experience, and no native behaviour is claimed from these
  packets.

## Exact missing evidence

1. **No class-switch frame in any preserved capture.** `lab/captures/**/*.bin` was scanned for
   `[26,0][13,0]`; the negative-control finding in the bundle records the frame count and zero
   matches. The recovered request shape currently rests on the enums, the client constructor and the
   community reader, not on an observed retail frame.
2. **No recovered failure-command payload semantics**, and no recovered string or reason id for a
   refused switch. The second field stays at the constructor default.
3. **No native handler decompilation** for the server side of `CommandPacketSetProfile`; the flow is
   community-recovered.

## Verification

* RED: `cargo test --test piece_044` failed to compile with
  `error: couldn't find file 'tests\..\src\restoration\piece_044.rs'` before the module existed
  (`lab/reports/piece-044-20261009T044119Z/red-cargo-test.log`).
* GREEN: `cargo test --test piece_044` -> 7 passed, 0 failed
  (`green-cargo-test.log`).
* Regression: `cargo test --test 'piece_0[0-4][0-9]'` -> every sibling target ok, including this one
  (`sibling-cargo-test.log`). A repo-wide `cargo test` currently stops on another worker's
  mid-write `tests/piece_051.rs`; that file is outside this item's paths and was not touched.
* Checker: `scripts/pieces/piece_044.py --reports-dir .../piece-044-20261009T044119Z` -> 55 findings,
  0 failures, 0 gaps (`checker-run.log`, `findings.json`, `provenance.json`).
* Checker tests: `python -m unittest discover -s tests/pieces -p "test_piece_044.py"` -> 17 passed
  (`python-unittest.log`), including synthetic negative controls for a drifted opcode, a renamed
  packet and a capture that already contains a switch frame.

## Integration and native gates still open

* The lead still has to register `pub mod piece_044;` where the restoration modules are declared;
  the owned test includes the file by `#[path]` until then.
* Wiring the failure command into the live switch paths is a shared-file change and belongs to the
  lead; both the opening-world and legacy gameplay paths currently answer a bad switch with a chat
  message instead.
* Native acceptance - actually switching class in the running client and reading back health, gear
  and the ability bar - is unrun and is the only gate that would support a playable claim.
