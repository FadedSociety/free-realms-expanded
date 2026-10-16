# System: minigame.fishing

37 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd1210` | `ProxiedFishingSchool::ProxiedFishingSchool` | ProxiedFishingSchool | Client-side proxy for a fishing school: holds its model and effect lists and links to its owner. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd16d0` | `ProxiedFishingSchool::ScalarDeletingDtor` | ProxiedFishingSchool | Scalar deleting destructor of ProxiedFishingSchool (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd1760` | `ProxiedFishingUnderwaterFish::ProxiedFishingUnderwaterFish` | ProxiedFishingUnderwaterFish | Client-side proxy for an underwater fish: initial state word 1, owner link at +0x170, default motion block. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd1950` | `ProxiedFishingUnderwaterFish::~ProxiedFishingUnderwaterFish` | ProxiedFishingUnderwaterFish | Destructor body of ProxiedFishingUnderwaterFish. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd19f0` | `ProxiedFishingUnderwaterFish::ScalarDeletingDtor` | ProxiedFishingUnderwaterFish | Scalar deleting destructor of ProxiedFishingUnderwaterFish (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3cf0` | `ProxiedFishingBobber::ProxiedFishingBobber` | ProxiedFishingBobber | Client-side proxy for a fishing bobber: links to its owner and holds its target and motion fields. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3de0` | `ProxiedFishingBobber::ReleaseMemberThenFreeIfFlag` | ProxiedFishingBobber | Virtual slot 0 of ProxiedFishingBobber: releases its owner-side member and frees the proxy when requested. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4420` | `FishingDataSource::FieldName` | FishingDataSource | Column-name lookup for the fishing data source. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd44e0` | `FishingScoringDataSource::FieldName` | FishingScoringDataSource | Column-name lookup for the fishing scoring data source. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4620` | `FishingDataSource::GetCellValue` | FishingDataSource | Cell accessor for the fishing data source table. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd47b0` | `FishingScoringDataSource::GetCellValue` | FishingScoringDataSource | Cell accessor for the fishing scoring data source table. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4930` | `FishingDataSource::~FishingDataSource` | FishingDataSource | Destructor body of FishingDataSource. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd49a0` | `FishingScoringDataSource::~FishingScoringDataSource` | FishingScoringDataSource | Destructor body of FishingScoringDataSource. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4a10` | `FishingDataSource::FishingDataSource` | FishingDataSource | Constructor of the FishTypes fishing data source; keeps arg2 at +0x254. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4a90` | `FishingDataSource::ScalarDeletingDtor` | FishingDataSource | Scalar deleting destructor of FishingDataSource (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4ab0` | `FishingScoringDataSource::FishingScoringDataSource` | FishingScoringDataSource | Constructor of the FishScored fishing scoring data source; keeps arg2 at +0x254. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4b30` | `FishingScoringDataSource::ScalarDeletingDtor` | FishingScoringDataSource | Scalar deleting destructor of FishingScoringDataSource (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4b50` | `FishingLureDataSource::FishingLureDataSource` | FishingLureDataSource | Constructor of the FishingLures data source. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4c90` | `FishingLureDataSource::AdjustorThunk_Sub254` | FishingLureDataSource | Entry through the secondary FishingLureDataSource vtable (01839674, slot 1) that reaches the primary scalar deleting destructor. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4ca0` | `FishingLureDataSource::ScalarDeletingDtor` | FishingLureDataSource | Scalar deleting destructor of FishingLureDataSource (vtable slot 0). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |

## 🟡 speculative (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd12a0` | `ProbeNodeListRebuild ?` |  | Walks the node list headed at arg2+4 (next link at +0xc). For each node it calls 00769710, 00726e30, 0079d310 and 00402ce0 and virtual slot 1 of the object at this+0x3c; when that returns non-null it obtains a node from… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd1720` | `ReadByte_0xd9 ?` |  | Returns the byte at ECX+0xd9 (fastcall; mov al,[ecx+0xd9]; ret). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd1a10` | `StateSwitchFrameUpdate ?` |  | Per-frame update that switches on the state dword at +0x20 (cases 1 to 8). It advances motion vectors at +0x130..+0x14c and timers at +0xc4, +0xc8, +0xcc, +0xd0, +0x154 and +0x160, sets an animation index at +0x1c throu… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3a70` | `ApplyArgsToStateAndCue ? ⚠rename` |  | Stores arg2 (byte) at +0x10 and arg3 at +0x14. Looks up an object through virtual slot 1 of the object at (+0x170)+4; when non-null, calls its slot 7 with a copied vector, sets +0x18 from its dword +0x268, maps arg7 (1-… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3c90` | `FieldAddr110 ?` |  | Returns ECX+0x110 (fastcall; lea eax,[ecx+0x110]; ret). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3ca0` | `ZeroFloatAt154 ?` |  | Stores 0.0f at ECX+0x154 (xorps xmm0; movss [ecx+0x154],xmm0; ret). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3cd0` | `FieldAddr140 ?` |  | Returns ECX+0x140 (fastcall; lea eax,[ecx+0x140]; ret). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3ce0` | `FieldAddr130 ?` |  | Returns ECX+0x130 (fastcall; lea eax,[ecx+0x130]; ret). | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3e20` | `ProbeHitAndDistance ?` |  | Gets the object at +0x20 and calls its virtual slot 4 with handle +0xd8, computes a target point from its +0x180..+0x18c scaled by +0xe0 with offsets, calls 00fd4290 with the point at +0x140..+0x14c and that target, sto… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd42a0` | `InitParamsAndSync ?` |  | Stores arg5 at +0x28, arg6 at +0xdc, arg7 at +0xe4 and arg8 at +0xe0, and the dword at arg4+0x268 at +0xd8. Looks up an object through virtual slot 1 of the object at +0x20 (key from arg2 and the owner's +0x260); when f… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4480` | `ForEachNestedRowCall ?` |  | Reads the object pointer at this+0x30. In a do-while over the outer count from its virtual slot 2, it reads the inner count from its slot 0 and calls 010cb9a0(outer, inner) for each inner index below that count. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd45e0` | `FishingDataSource::Value29C ?` | FishingDataSource | Virtual slot 2 of FishingDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x29c of the object it points to. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4600` | `FishingScoringDataSource::Value220 ?` | FishingScoringDataSource | Virtual slot 2 of FishingScoringDataSource: returns 0 when the pointer at this+0x224 is null; otherwise returns the dword at +0x220 of the object it points to. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4bd0` | `Wrapper_00cb1b10 ?` |  | Thiscall with no stack arguments: installs an EH frame, saves ECX in the frame, calls 00cb1b10 with ECX as the receiver, removes the frame and returns. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cd1550` | `— ⚠reject` | ProxiedFishingSchool | Destructor body of ProxiedFishingSchool. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd3f80` | `— ⚠reject` |  | Converts arg2 to a step; when the byte at +0xd4 is set, moves the target at +0xf0..+0xfc toward the point at +0x140..+0x14c by a normalized step (speed +0xdc, clamped by the distance at +0x100) and refreshes the block a… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd4c20` | `— ⚠reject` | FishingLureDataSource | Virtual slot 15 of FishingLureDataSource. Returns 0 unless 00b0d810 returns non-null twice; then returns 1 when 00bae6a0 is non-zero, the record from the first call has dword +0x6c equal to 1 and dword +0x8c in (0x10be,… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |

