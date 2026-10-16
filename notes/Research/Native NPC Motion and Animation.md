# Native NPC motion and animation

2026-10-08 UTC. Investigated the owner's report of stationary enemies, absent attacks and long-range hits in run073831. This note establishes packet/asset facts for the root agent's combat fixes. It does **not** claim a successful visible motion/attack retest or an original AI implementation. Hit reach, damage rules and reconstructed AI remain the root agent's responsibility.

## Community first and independent oracle

Compiled unmodified packet writer/reader sources directly from pinned [Sanctuary](https://github.com/Open-Source-Free-Realms/Sanctuary) revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e` into the isolated private `lab/tools/native-motion-oracle` project. AGPL-3.0 sources and generated game-specific binaries remain outside Git. Thirteen fixtures in `lab/reports/native-motion-oracle` are verified by an independent Python cursor/expected field check. The compiled writer proves these offsets in the empty334-byte AddNpc template:

| Field | Template offset | Original value |
| --- | --- | --- |
| Speed,f32 | 128 | 0.0 |
| Walk/Run/Stand animation IDs,i32 | 137/141/145 | -1/-1/-1 |
| IsInteractable,bool | 224 | true |
| RiderGuid,u64 | 225 | 0 |
| MovementType,i32 | 233 | **2**, already present |

Offsets shift after variable string/list fields. Patch the verified empty template before inserting texture aliases and names, or account for their lengths. The template is already physics MovementType2; do not describe the failure as an observed MovementType0. Captured run072258 opening actors also retain type2.

```powershell
& '../lab/tools/dotnet/dotnet.exe' run --project '../lab/tools/native-motion-oracle/NativeMotionOracle.csproj' -- '%USERPROFILE%/Documents/Projects/CartoonMMO/lab/reports/native-motion-oracle'
& '../lab/tools/wiki-env/Scripts/python.exe' '../lab/reports/verify-native-motion.py'
```

## Native movement

Opcode125 is standalone: `u16 opcode,u64 GUID,3f position,3f rotation,u8 State,u8 Unknown` =36bytes. Native reader `008e5940` (original1030083) consumes exactly those fields, initializes positionW=1/rotationW=0, and has **no timestamp** field. Existing community NPC movement uses State1/Unknown0.

Handler `0090de90` (1065441) resolves the GUID, excludes self/attached actors, then branches on entity+0x188c. Type2 calls `00963840`; type1 takes the platform/controller route; type0 has no position-apply branch. Ghidra mislabels integer2/1 as tiny floats in this function. AddNpc dispatch assigns this destination from its movement field (original1092670); native per-frame physics also checks type2 (1145194).

Statebit0 set passes0.0 as the speed parameter; clear calls `008b3180`. That getter returns entity.Speed(+0x3ec) times1.0, or times0.25 when entity+0x650 is set. Both constants were read directly from preserved PE bytes. `00963840` queues target positions and normalizes nonzero rotation; it deliberately skips an all-zero rotation. The original writer's zero heading cannot orient an attacker toward the player. A finite horizontal direction and explicit animation packets are supported by native code.

Zero spawn speed and unset locomotion overrides are concrete differences from Sanctuary's moving NPCs. Further physics branches decide how queued positions display; this audit does not assert an infinite interpolation mechanism or that State0 alone explains the stationary model. A visible native retest is required after the combined changes.

### Facing direction proven against the native setter

The rotation field is a **direction vector**, not a quaternion with its W component omitted. Native `0095dae0` first masks its input with `[1,0,1,0]`, requiring a nonzero horizontal X/Z length (1126963–1126967). Static initializer `0173c1c0` establishes this mask. It then normalizes XYZ with `[1,1,1,0]` from initializer `0173c140`, stores the result at entity+0x610/+0x614/+0x618/+0x61c, and explicitly derives yaw using `atan2(X,Z)` before the model rotation setter `007776b0` (1127006–1127011). Capstone disassembly of the preserved executable independently confirms the floating-point loads and `fpatan` instruction.

A yaw quaternion truncated to XYZ, `[0,sin(yaw/2),0]`, has zero horizontal length and is ignored by this setter. The community PathFollower's Quaternion type therefore cannot establish the meaning of the three wire floats. A finite horizontal heading such as `[-1,0,0]` correctly faces negative X. Captured native player movement also uses changing horizontal unit directions with Y=0.

`00963840` enqueues position and normalized direction through `0095e890`. Per-frame `0096d370` consumes that queue and uses entity.Speed(+0x3ec) and `008b3180` in movement calculations. The queue node's +0x20 field is the separate vertical parameter supplied to `00963840`, **not** the speed pointer; the speed pointer also controls entity+0x143 bit4. The Statebit0 choice and zero spawn Speed are distinct inputs. These static branches establish the relevant application path, but do not prove exactly which combination caused the user's stationary result. Root owns the combined Speed3, explicit locomotion IDs, State1 and finite-heading change and its visible retest.

Additional fixtures copied into `lab/references/rust-opening/encounter-oracle` preserve GUID5715144128442986753: `movement-v2.bin` sends position `[291.25,28,289]`, direction `[-1,0,0]`, State1/Unknown0; `animation-attack01.bin` sends ID1001/Flags0; `animation-run.bin` and `animation-stand.bin` send IDs3/1 with Flags1. The separate `movement-v2-manifest.json` records lengths and SHA-256 hashes without replacing the earlier oracle files.

## Native animation

Opcode35/subtype8 is `u16 opcode,u16 subtype,u64 GUID,i32 animationID,i32 Unknown,u8 Flags` =21bytes. Native reader `00908a50` and subtype8 dispatch (1093435) corroborate the community writer. Flagsbit0 set assigns base/idle animation at entity+0x51c. Clear immediately calls `0096c780`→`007829a0` animation lookup. Flagsbit1 is forwarded as a separate boolean; keep it zero for the neutral immediate attack case.

The supplied `Resources/CodeAnimationMappings.txt` maps data IDs1=`loc_stand`,3=`loc_run`,1000=`com_h2h_attack`,1001=`com_h2h_attack_01`. Pinned FR ADR Toolkit revision `5acfdf8d6006fa6f43d1d3605f076a6be7393e72` actually parsed preserved `wolf_generic.adr` and `gloamling_m_default.adr`. Both contain stand/walk/run and hand-to-hand attack01 slots bound to corresponding GR2 clips that exist in the extracted catalog. Wolf attack02 also reuses its attack01 clip. Native resources/ADR bindings establish usable animation candidates; they do not prove rendered playback or every boss/model's support.

Private reproducible evidence: `lab/reports/native-motion-verification.json`, `native-motion-functions.c/.json`, `native-motion-disassembly.txt`, `verify-native-motion.py`, and the C# fixture manifest. The helper verifies field mutation offsets, all thirteen direct-writer fixtures, preserved native speed constants and rotation masks, pinned ADR reader results and animation clip catalog matches. No shared Rust, harness, saved progress or game input was changed by this audit.
