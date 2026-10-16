# 091 Pets and companions

Status: bounded recovery returned for lead review, 2026-10-09 05:07 UTC. All commands were
run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`. Raw game files and
decompiled output stay private under `../lab`; this note carries provenance, recovered
facts, the reconstruction boundary and the remaining gates.

Queue acceptance was "Recover ownership/spawn/follow/interaction definitions; native
behavior and preserved pet state." What is delivered: the recovered pet packet family and
the client constructors, the client-side request bodies for summon/recall, summon-by-item,
follow-location and in-water, the preserved per-pet profile record with a 28-record
recorded sample, the recovered client interaction surface, and a bounded reconstructed
companion session. No native run is claimed.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_091.rs` | Subtype table, request codecs, the `PetRecord` codec and the reconstructed `CompanionSession`. Not registered in `src/lib.rs` (lead-owned). |
| `prototype/osfr-tools/tests/piece_091.rs` | 15 focused tests, `#[path]`-including the module so it compiles and runs before registration. |
| `scripts/pieces/piece_091.py` | Read-only extractor/checker over the preserved corpus; writes the private lab bundle. |
| `tests/pieces/test_piece_091.py` | 16 stdlib tests: synthetic rejection paths plus corpus pins and a byte round trip over all 28 recorded pet records. |
| `../lab/reports/piece-091-20261009T050738Z/pet-companions-20261009T050738Z/` | Private evidence: `catalogue.json`, `findings.json`, `provenance.json` (source paths + SHA-256). |

## Recovered originals

### Packet family

The pet family is base opcode **53 (0x35)** with 35 named subtypes. Three independent
sources agree:

* Client constructor `FUN_00b50670` writes `param_1[1] = 0x35` and `param_1[2] = subtype`
  (`lab/decompiled/client/FreeRealms/pseudocode.c`, the shared base for every pet packet).
* Sanctuary `PacketReaderExtensions.cs` line 499: `53 => reader.Read<byte>() switch { 1 =>
  "PetDecisionPacket", ... 38 => "PetPreviewRequestPacket" }`.
* OpenSourceFreeRealms `LibSOE/Core/PacketCodes.cs` `PetBasePackets` enum (35 members).

Every named subtype has a constructor in the client export, derived by the checker (not
assumed). Representative rows (`subtype  ctor  class`):

| Subtype | Ctor | Class | Subtype | Ctor | Class |
| --- | --- | --- | --- | --- | --- |
| 1 | `FUN_00b50730` | PetDecisionPacket | 18 | `FUN_00b51610` | PetUtilityGroomPacket |
| 3 | `FUN_00b507e0` | PetStatusPacket | 19 | `FUN_00b51730` | PetUtilityFeedPacket |
| 4 | `FUN_00b50890` | PetSummonRecallPacket | 23 | `FUN_00b53090` | PetExpireTimePacket |
| 5 | `FUN_00b52f90` | PetListPacket | 24 | `FUN_00b519c0` | PetUiModePacket |
| 9 | `FUN_00b53020` | PetActivePacket | 26 | `FUN_00b51b20` | PetCanGoLocationPacket |
| 10 | `FUN_00b544d0` | PetSetNamePacket | 28 | `FUN_00b51bd0` | PetInWaterPacket |
| 11 | `FUN_00b50e00` | PetEquipPacket | 29 | `FUN_00b51c80` | PetTempLockoutError |
| 17 | `FUN_00c06b40` | PetDeletePacket | 32 | `FUN_00b50940` | PetSummonByItemIdPacket |
| 22 | `FUN_00ccdb70` | PetTactileNotify | 33 | `FUN_00b50b30` | PetPlayWithToyPacket |

The full 35-row table is in `catalogue.json` under
`packet_family.client_constructors`. Subtypes 0, 27, 30 and 37 exist in no source and are
not invented.

### Client request bodies

The recovered field layouts, with the client function that writes each field. Sizes are in
bytes, in wire order:

| Subtype | Fields | Sender(s) |
| --- | --- | --- |
| 4 `PetSummonRecallPacket` | `u32 pet_id`, `u8 action` (1 = summon, 0 = recall) | `FUN_00b60080` (action 1), `FUN_00b601a0` (action 0), `FUN_00c07420` (action 0, ClientGateway template) |
| 4 (forced dismiss) | `u8 action = 0` alone | `FUN_00b60610` - the sender assigns only the action byte and never the id field |
| 26 `PetCanGoLocationPacket` | four 32-bit floats + `u8` flag | `FUN_00b5fcf0` (client follow path), `FUN_00b5ce60` (shared pet handler) |
| 28 `PetInWaterPacket` | `u8` flag | `FUN_00b60350` (1), `FUN_00b60420` (0) |
| 32 `PetSummonByItemIdPacket` | `u32 pet_id`, `u32 item_id` | `FUN_00b60110` |

The subtype 26 body shape is corroborated by the packet's serializer `FUN_00b5a530`, which
writes the 16-byte block at object offset `0x10` and the byte at `0x20`.

Subtypes 7/8/14 (perform trick, gesture result, favourite set) are already pinned by
[[011 Pet training activities]] and are deliberately not duplicated here.

### Preserved pet state

`lab/decompiled/server/Gateway/Gateway.Login/ClientPcData.cs` carries the recovered managed
record serializer `UnknownPetStruct.Serialize` (line 353 class, line 397 method; plus
`PetTrickInfo`, `ItemGuid`,
`ProfileItemClassData` and `WriteString`, which is a 32-bit UTF-8 byte length followed by
the bytes). The checker re-derives the field order from that method rather than trusting a
copy, and it matches the pinned order exactly:

`Unknown, Unknown2, Unknown3, Food, Groom, Exercise, Happiness, Unknown8,`
`PetTricks.Count, PetTricks[], ItemGuids.Count, ItemGuids[], ProfileItems.Count,`
`ProfileItems[], PetName, Unknown10, Unknown11, TextureAlias, IconId, Unknown14,`
`Unknown15, Unknown16, Unknown17[4], Unknown18[8]`

Record sizes: 26 bytes of scalars, `4 + 37n` for `n` trick rows, `4 + 4g` guids,
`4 + 12p` utility items, two length-prefixed strings, 11 bytes of trailing flags and ints,
then the two int arrays.

The recorded sample `working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json`
holds **28 pet records**, all with unique ids and all round-tripping byte-exactly through
the checker's encode/decode pair (8,225 bytes total). The first record is **Captain Zeus**
(482 bytes): id 2, `Unknown3` 652, Food 53.0, Groom 82.37982, Happiness 2.909777, seven
trick rows, five item guids, five utility items, texture alias `lightmark-chocolate`, icon
3211, `Unknown17` `[17,16,15,14]`, `Unknown18` `[0,688,0,0,352,354,129,0]`. Ten of the 28
records carry no trick list at all and several have an empty name.

The community `Sanctuary/.../ClientPcData.cs` writes the same profile with `ActivePetId`
and `ActivePetGuid` after a `// TODO Pets` placeholder, so the active-pet pair belongs to
the profile record too; the recorded dump contains the pets list but no active-pet row.

### Client interaction surface

`lab/decompiled/client-ui/ScriptsBase.lua` carries the whole controller. Recovered engine
calls: `Summon, Dismiss, GetActivePetId, IsBusy, RaiseNeed, StartFeed, StartGroom,
EquipUtility, EquipItem, PlayWithToy, PerformTrick, SetFavoriteTrick, SetGestureData,
SetName, SetNameCustom, ModeSwitch, WasGood, WasBad, CameraAtPet, CameraRestore,
StopGestureGame, TutorialEnd`. Recovered handlers: `OnForcedDismiss, OnSummonFailed,
LockUtilities, UnlockUtilities, GetPetTableByID, SetProgress, ShowProgress, IsVanityPet,
PetDataUpdated, CancelPetTrick` and the rest of the controller table.

Behaviour that matters for ownership and spawn:

* `BrowserOnUserEvent("SelectPet", b)` calls `Pet.Summon(b)` with the browser row's pet id;
  the dock path additionally gates members-only pets (`c == "1"`) before summoning.
* `Pet.GetActivePetId()` returns 0 when nothing is spawned (`if 0 < Pet.GetActivePetId()`).
* `PetController:CloseController()` calls `Pet.Dismiss()`; a new summon closes the previous
  controller first.
* `PetsHandler:OnForcedDismiss()` and `OnSummonFailed()` are the client's server-driven
  paths, and `FUN_00b60610` is the client's forced-dismiss sender.
* `PetExpireTime(ID, time, sadPetIconId)` drives the trial countdown; `PetsHandler:IsVanityPet`
  distinguishes vanity pets, and `Pet.IsBusy` / `LockUtilities` gate the utility buttons.
* The native utility gate is `PetsHandler:UnlockUtilities`, reachable from
  `FUN_00b5f720` / `FUN_00b5f8c0`, which require the pet not to be busy and a flag bit at
  `+0x143` to be clear.

## Inferred (not named by any recovered source)

* The record's leading `Unknown` int is treated as the pet id used by the pet browser and
  `Pet.GetActivePetId`. It is the only per-pet unique id in the sample (28 distinct values);
  the serializer never names the field.
* `Unknown3` looks like a species/family id because the sample only uses 652 and 653.
* `Exercise` is paired with the client's `play` progress column; the sample's untrained rows
  pin Exercise at 100.

## Reconstructed (server policy, not history)

Implemented in `piece_091.rs` and labelled there:

* One active pet per character; summoning another pet replaces it.
* A spawn requires an owned pet (roster membership) and a non-zero actor guid.
* Recall applies only to the pet that is currently active; forced dismiss is the
  server-initiated despawn.
* Follow updates (`CanGoLocation`, `InWater`) and utility requests need the active actor
  guid; utilities are refused while the pet is busy.

Deliberately absent: follow distance thresholds (the client compares against
`DAT_01b5d878`, `DAT_0176b1b0`, `DAT_017ed93c`, `DAT_017649d4`, none resolvable from the
export), need decay, utility or trial timers, trick effects, purchase and reward values.

## Gaps and remaining gates

* **No decoded pet datagram exists.** The piece 011 wire probe scanned the preserved
  captures and found zero pet-family frames; this checker re-reports that finding instead
  of re-running Scapy. Every body above comes from the client's own writers, so the
  datagram-level encoding (SOE framing, field endianness on the wire) stays unverified.
* 31 of the 35 subtypes still have no pinned body; three of them (7, 8, 14) are covered by
  piece 011.
* Field semantics for 11 record fields remain `Unknown`; they are kept as gaps in the
  catalogue rather than named.
* Native behaviour, pet models, follow motion and the pet UI are unverified. The lead owns
  the native gate; this piece did not touch the shared client window, so the strongest
  status it supports is `code/test-candidate`.
* Integration: the module is unregistered. The lead either adds `pub mod piece_091;` to
  `src/restoration`/`src/lib.rs` and drops the `#[path]` include in the test, or keeps it
  as an evidence module.

## Verification

Initial RED (naive baseline scaffold, before the implementation):

```powershell
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_091
# test result: FAILED. 1 passed; 14 failed
```

GREEN, Rust:

```powershell
scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_091
# test result: ok. 15 passed; 0 failed
```

RED then GREEN, Python checker (the first runs exposed four real corpus problems: the
enum's last member has no trailing comma, three pet constructors assign a local before the
vftable store, the first `Serialize` in the recovered file is an unrelated empty class, and
the serializer's trailing `for` loops write one array element per trip):

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_091.py"
# first run: FAILED (failures=6, errors=4); final run: Ran 16 tests ... OK
```

Extractor/evidence bundle:

```powershell
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_091.py' --stamp 20261009T050738Z --reports-dir '..\lab\reports\piece-091-20261009T050738Z'
# {"base_opcode": 53, "subtypes": 35, "client_constructors": 35, "pinned_layouts": 4,
#  "recorded_records": 28,
#  "findings": ["payload-layout-gap", "field-semantics-gap", "community-payload-gap", "wire-gap"]}
```

Source hashes recorded in `provenance.json`: pseudocode
`f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3`, client Lua
`c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`, recovered server
serializer `5aa6b566ac418eaa68ec4b546f34eb09f84c9fd894b047a02ac7e7007dc59998`, recorded
profile sample `e471ab7fd98532328fada3e759bdcb073f2baf626a4650f7334a540718358777`.

## Skills applied

`game-recon` for the engine fingerprint (native packet classes plus a Lua controller rather
than a Flash-only path), `reverse-engineering` for the Ghidra field/size recovery and the
recovered managed serializer, `game-research-websearch`/community-first practice (the two
pinned readers, the recorded profile sample, the earlier Scapy-based wire probe) before any
custom parsing, plus RTK (normal commands first), Ponytail (reused the existing corpus and
the piece 011 wire finding instead of new readers or another capture scan) and Caveman
(concise reporting). `asset-pipeline` was not needed, and `game-automation` was not used:
the shared native window is lead-controlled.

Cross-links: [[../../MMO Work Queue]], [[011 Pet training activities]],
[[043 Pet trainer and card progression]], [[../Free Realms Recovery]].
