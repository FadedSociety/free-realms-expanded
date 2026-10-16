# 092 Mounts and vehicles

Queue item 092. Acceptance: recover mount/dismount/speed/ownership requests; valid state
transitions and native visuals.

Status 2026-10-09 05:41 UTC: the mount base packet surface (opcode 168 and its 13 sub-opcodes), the
client request bodies for spawn/dismount, the captured character's 34 ownership rows, the mount
speed/glide stat block (opcode 38 sub-opcode 7) and the two server response field orders are
recovered and re-derived by a checker (63/63 checks). A bounded session contract gates spawning on
ownership. **No native or gameplay acceptance was attempted**: the actor packets that make a mount
visible are deliberately not re-encoded here, and no mount was spawned in the client.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_092.rs` | Recovered opcode/sub-opcode/stat-id constants, the 34 captured ownership rows, request decoder, response + stat-block encoders, `MountSession`, `mount_stats`/`dismount_stats` |
| `prototype/osfr-tools/tests/piece_092.rs` | 11 tests; includes the module by `#[path]` until the lead registers it |
| `scripts/pieces/piece_092.py` | Read-only checker over the enum, pseudocode, capture, item tables, `Mounts.json`, OSFR `MountSystem` and Sanctuary packet sources |
| `tests/pieces/test_piece_092.py` | 20 stdlib tests: hermetic tamper cases, parsers and real-corpus pins |
| `../lab/reports/piece-092-20261009T0541Z/` | `piece-092-check.json`, `piece-092-provenance.json`, checker console, RED/GREEN logs |

## Recovered original facts

1. **Packet family.** `MountBasePacket` stores outer opcode `0xa8` (168) and a one-byte sub-opcode:
   the native constructor writes `param_1[1] = 0xa8;` immediately before the `MountBasePacket`
   vtable. The recovered enum (`LibSOE/SOE/MountBasePackets.cs`) is 1 `PacketMountRequest`,
   2 `PacketMountResponse`, 3 `PacketDismountRequest`, 4 `PacketDismountResponse`,
   5 `PacketMountList`, 6 `PacketMountSpawn`, 8 `PacketMountSpawnByItemDefinitionID`,
   9 `PacketSetAutomount`, 10 `PacketMountStartTrial`, 11 `PacketMountEndTrial`,
   13 `PacketMountTrialFailed`, 14 `PacketMountUpgradeRequest`, 15 `PacketMountUpgradeResponse`.
   Every one of those numbers is confirmed independently in the native client: each packet class
   constructor calls `FUN_0089f9c0(<sub-opcode>)` and then installs its own vtable (13 pairs
   re-checked by the checker).
2. **Client request surface.** The client Lua drives it: `mountSelected` calls `Mount.Spawn(b)`
   with the mount-list id, `dismountButtonSelected` calls `Mount.Dismount()`, and the dock gate is
   `if MiniGameStatusHandler:GetCurrentMiniGameType() == -1 and not Ui.IsInOverworldCombat()`.
   The mount browser also has an upgrade path (`self.mountToUpgrade = b`). The native
   `PacketMountSpawnByItemDefinitionId` path logs `SpawnMountByItemDefinitionId: item %d` and the
   Lua bridge logs `Lua Spawn and automount: id %d`.
3. **Request bodies recovered** (bytes after the tunneled inner opcode 168):
   sub-opcode 6 = `i32` mount number; sub-opcode 8 = `i32` item definition id + one boolean;
   sub-opcode 3 = empty. The supplied OSFR server reads the sub-opcode byte and then an `i32` for
   the spawn path only, and the pinned Sanctuary deserializers require exactly those lengths
   (`PacketMountSpawn`, `PacketMountSpawnByItemDefinitionId`, `PacketDismountRequest`). Sub-opcodes
   1, 2, 4, 5, 9, 10, 11, 13, 14, 15 remain body-unrecovered.
4. **Ownership rows.** The captured self packet carries 34 `Mounts` rows in capture order, each
   with mount number, name id, icon id, guid, members-only, tint id, tint alias, fly-training,
   able-to-fly, model id and item definition id. Census: 1 members-only row (1017), 3 fly-training
   rows (1019, 1017, 1001) and 4 able-to-fly rows (1030, 1025, 1024, 1002). Numbers run 1000-1033.
   **Four items are not in either pinned item table**: 81256176 (1033), 163632473 (1032),
   35463452 (1031) and 6346231 (1023); the other 30 resolve. Rows 1002 and 1017 share item 11038,
   so the item-to-mount lookup is not unique in the capture.
5. **Fire Hawk cross-link.** Captured row 1030 (`MountName` 442674, `MountModelID` 4590,
   `MountItemDefinitionID` 78397, able-to-fly) matches `Mounts.json` row 13459 "The Fire Hawk"
   (`NameId` 442674, `ModelId` 4590, `ImageSetId` 8556), and item 78397 binds to that row through
   `Param1 = 13459`. Three independent records therefore agree on the same mount.
6. **Speed/glide stat block.** Character stats travel as opcode 38 sub-opcode 7: actor guid, `i32`
   entry count, then per entry `i32` stat id, `i32` value kind (0 = `i32`, 1 = `f32`) and the typed
   value. The mount block uses `MaxMovementSpeed` (2), `GlideDefaultForwardSpeed` (59),
   `GlideMinForwardSpeed` (60), `GlideMaxForwardSpeed` (61), `GlideFallTime` (62),
   `GlideFallSpeed` (63), `GlideEnabled` (64, integer), `GlideAccel` (67) and `JumpHeight` (68);
   the ids resolve against the recovered `CharacterStatId` order. Recovered values: Fire Hawk
   (name id 442674) 12.5 / 12 / 6 / 22 / 0.75 / 3 / 1 / 4 plus jump 5; able-to-fly 12.5 / 8 / 2 /
   18 / 0.75 / 4 / 1 / 4 with no jump entry; default mount 12.5 only; mount number 1023 a single
   `MaxMovementSpeed` 0.0; dismount restores `MaxMovementSpeed` 8.0, `GlideEnabled` 0 and
   `JumpHeight` 0.0. The Fire Hawk numbers also appear verbatim in `Mounts.json` row 13459.
7. **Server responses.** `168/2` writes rider guid, mount guid, seat, queue position, unknown,
   composite effect and name vertical offset; `168/4` writes rider guid and composite effect. The
   supplied OSFR server and the pinned Sanctuary sources agree field for field, and both stamp
   composite effect 46. The visible mount itself arrives as an actor packet (`35/2`
   `PlayerUpdatePacketAddNpc`) and the dismount pairs with `35/3`; those two belong to the actor
   pipeline and are not re-encoded in this piece.

## Community evidence used (pinned, not original binaries)

`references/OpenSourceFreeRealms` (the supplied OSFR server tree) and
`references/Sanctuary` both carry mount handlers; `references/FreeRealms-Legacy` carries a
byte-identical `Mounts.json`. The wiki corpus holds pages "Mount" (revision 8101) and
"Misty Mountain" (49139); neither is load-bearing for the claims above and no page text is used.

## Reconstructed policy (labelled, not historical behaviour)

1. `MountSession` refuses a spawn for a mount number the captured rows do not own, replaces the
   current mount when a second spawn arrives, and reports a dismount with no mount as a no-op.
   Both pinned servers dismount before spawning; what the original server sent for a duplicate
   spawn is not recovered.
2. `item_bound_mount` resolves the duplicate item 11038 with the last captured row (1002),
   mirroring Sanctuary's `LastOrDefault`. The original tie-break rule is not recovered.
3. `mount_stats` follows the OSFR branch order (Fire Hawk name id, then able-to-fly, then mount
   1023, then the 12.5 default). The mount-1023 zero-speed literal has no recovered meaning and is
   carried as-is rather than explained away.

## Explicit gaps

* `PacketMountRequest` (1), `SetAutomount` (9), the trial packets (10, 11, 13) and the upgrade
  packets (14, 15) have no recovered bodies; the trial/upgrade gameplay has no owner here.
* `PacketMountList` (5) is server-to-client; the only field order available is the community
  `PacketMountInfo` (`Id, NameId, ImageSetId, Guid, MembersOnly, TintId, TintAlias, IsUpgradable,
  IsUpgraded`), which the supplied capture never exercises. The client Lua data-source columns
  (`Id, ImageSetId, Name, IsCurrentlySpawned, IsCurrentlyMounted, Tint, IsMemberOnly,
  IsUpgradable, IsUpgraded`) are recovered but not yet mapped to a sent packet.
* Interaction-mounting (`PacketMountRequest` against a mountable NPC) and the mount geometry the
  actor packet carries are unrecovered.
* Four captured rows name item definitions that resolve in no pinned table; the ownership gate
  therefore keys on the captured mount number, not on an item join.
* Vehicle physics belong to the kart/derby families (queue items 004 and 041), not to this piece;
  nothing here claims mount movement, collision or animation.

## Verification

| Command (project directory) | Result |
| --- | --- |
| `.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --test piece_092` with the module absent | RED: `error: couldn't find file tests\..\src\restoration\piece_092.rs` |
| `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest tests.pieces.test_piece_092 -v` with the checker absent | RED: `FileNotFoundError ... scripts\pieces\piece_092.py` |
| `.\scripts\cargo.ps1 test --manifest-path .\prototype\osfr-tools\Cargo.toml --lib --test piece_092` | GREEN: lib 24 passed, `piece_092` 11 passed, 0 failed, 0 warnings |
| `& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest tests.pieces.test_piece_092 -v` | GREEN: 20 passed (includes the real-corpus run) |
| `& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_092.py --out ..\lab\reports\piece-092-20261009T0541Z` | GREEN: 63/63 checks pass, 0 problems |

## Native and integration gates (lead-owned)

1. Dispatch tunneled inner opcode 168 to the decoder and answer `168/2` + `168/4` from a server
   state machine; ownership must come from the character store, never from the request alone.
2. Add the `38/7` stat block on spawn and on dismount, and keep the actor pipeline's `35/2`/`35/3`
   spawn/remove packets as the visible-mount step.
3. Native acceptance: mount, dismount and re-mount in the client with a captured mount row, confirm
   the model appears and the glide/speed values change, then repeat after a restart. This piece
   claims none of that.

## Skills applied

Ponytail (reuse: no new dependency, the module re-uses the recovered stat-id space documented by
item 047 and adds no actor encoder), Caveman (concise reporting; notes stay complete),
reverse-engineering (decompiled client constructors, Lua and community servers as the oracle) and
game-recon (identified the SOE/ForgeLight packet families and the pinned community readers).
game-research-websearch and asset-pipeline were not applied: the pinned community readers answered
the questions, and no asset output is produced here. game-automation is the lead's gate.
