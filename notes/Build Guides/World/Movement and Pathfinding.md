---
type: build-guide
section: World
updated: 2026-10-12
---

# Movement and Pathfinding

Part of [[Build Guides]] › World. Related: [[Physics]], [[Actors and NPCs]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

150 accepted functions: 60 confirmed, 50 likely, 23 speculative, 17 unknown. Network-linked (`wire`): 5.

Subsystems: `game.movement` (53), `world` (45), `world.pathfinding` (25), `world.proximity` (16), `world.camera` (8), `world.railcamera` (3)

### `game.movement`

**ClientInterpolatedMovementController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc4410` | ClientInterpolatedMovementController::LookupTransformForKey… | confirmed | Initializes the four-dword output from a global vector, calls 00769710 and the two-word hash lookup 00984960, and, when found, copies four dwords returned by n… |
| `00cc4600` | — | unknown | Looks up the controller node using the two dwords at this+0x10 and +0x14; returns if absent, obtains and updates its four-word transform, stores param_4 at nod… |

**ClientMovementManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fdf40` | ClientMovementManager::ClientMovementManager | confirmed | fastcall (ECX=this, returns this). Calls 008fdb80 (the MovementManager constructor, see that record) on this, then stores ClientMovementManager::vftable at thi… |
| `008fdfa0` | ClientMovementManager::Dtor | confirmed | fastcall (ECX=this, EH frame). Stores ClientMovementManager::vftable at this+0, then calls 008fdc00 (MovementManager::Dtor, see that record) on this. Returns v… |
| `008fdff0` | ClientMovementManager::DtorAndConditionalFree | likely | thiscall with one byte argument (ret 4). Stores ClientMovementManager::vftable at this+0, calls 008fdc00 (MovementManager::Dtor) on this, then if bit 0 of the … |

**InitiateMountControllerHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008bb440` | InitiateMountControllerHandler::ResetController | confirmed | Resolves the stored ids and, when the associated controller state is active, calls two helpers with values 1 and 0; then resets the ids to global defaults. |
| `008bb3a0` | — | unknown | Resolves the two stored ids and checks controller state; if eligible, invokes FUN_00b04290 with a temporary id pair and stored field, then resets the stored id… |

**InterpolatedMovementController** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01077940` | InterpolatedMovementController::Destruct | confirmed | Destructor body under an EH frame: stores InterpolatedMovementController::vftable into this, then MovementController::vftable. Calls no other function and free… |
| `01077990` | InterpolatedMovementController::SetDwords4At20 | confirmed | Thiscall with one stack argument (pointer to four dwords): copies them to this+0x20..+0x2c, then clears the byte at this+0x38. ret 4. |
| `010779c0` | InterpolatedMovementController::SetFloat30And34 | confirmed | Thiscall with one float stack argument: stores it to this+0x30 and to this+0x34 (movss). ret 4. |
| `01077a70` | InterpolatedMovementController::InterpolatedMovementControl… | confirmed | Constructor under an EH frame with two stack arguments (ret 8): calls MovementController::MovementController (010779e0) on this with the same two arguments, th… |
| `01077af0` | InterpolatedMovementController::ResetVtablesFreeIfBit0 | confirmed | Thiscall with one stack byte argument under an EH frame: stores InterpolatedMovementController::vftable then MovementController::vftable into this; if bit 0 of… |
| `01077b60` | InterpolatedMovementController::AdvanceInterpolation | likely | Thiscall with one uint argument (elapsed ms; 0 means derive it from the tick pair at +0x40/+0x44 via 007027f0, clamped to 0x7fffffff). Calls virtual +0x24 to g… |

**MovementController** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01077910` | MovementController::DtorBody | confirmed | Fastcall. Stores the MovementController vtable (0x1882e8c) into the object's first dword and returns. No members are released. |
| `01077930` | MovementController::GetByte38 | confirmed | Returns the byte at this+0x38 (mov al,[ecx+0x38]; ret). No other side effects. |
| `010779e0` | MovementController::MovementController | confirmed | Thiscall constructor with two stack arguments (ret 8): stores MovementController::vftable; stores the arguments at this+0x10 and +0x14; copies 16 bytes of cons… |
| `01077a50` | MovementController::ResetVtableFreeIfBit0 | confirmed | Thiscall with one stack byte argument: stores MovementController::vftable into this; if bit 0 of the argument is set calls _free(this) through thunk 00d82463; … |

**MovementManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fdb80` | MovementManager::MovementManager | confirmed | fastcall (ECX=this), returns this. Stores MovementManager::vftable at this+0 and SoeUtil::HashListMap<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureChar… |
| `008fdc00` | MovementManager::Dtor | confirmed | fastcall (ECX=this), EH frame. Stores MovementManager::vftable at this+0 and the HashListMap<...,1024,-1> vtable at this+4, then while the head at this+0xc is … |

**ProxiedCharacterMovementController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfba00` | ProxiedCharacterMovementController::~ProxiedCharacterMoveme… | confirmed | Rewrites the object's first dword to ProxiedCharacterMovementController::vftable and returns; it releases no storage. |
| `00bfba80` | ProxiedCharacterMovementController::ScalarDeletingDestructor | confirmed | Restores the movement-controller vtable, tests param_2 bit 0, conditionally frees this, and returns the original pointer when it does not free. |

**ProxiedCharacterRailMovementController** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfbaa0` | ProxiedCharacterRailMovementController::AdvanceRail | confirmed | Adds the tick delta to progress at this+0x20 and clamps it to the rail length. Computes the rail transform, calls the controlled object's virtual+0x20, updates… |
| `00bfbd70` | ProxiedCharacterRailMovementController::~ProxiedCharacterRa… | confirmed | Restores the rail vtable, calls the cleanup helper and frees the owned pointer at this+0x24 when non-null, then restores the base movement-controller vtable. |
| `00bfbde0` | ProxiedCharacterRailMovementController::ScalarDeletingDestr… | confirmed | Calls the rail destructor body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |
| `00bfbca0` | — | unknown | Installs the base and rail-controller vtables, stores its constructor arguments and transform, clears progress, then initializes route state and runs one updat… |

**ProxiedCharacterRelativeRailMovementController** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfbe10` | ProxiedCharacterRelativeRailMovementController::AdvanceRela… | confirmed | Advances the two progress values at this+0x38 and this+0x3c, clamped to their route lengths. Interpolates the paired route position, applies the transform thro… |
| `00bfbfb0` | ProxiedCharacterRelativeRailMovementController::ProxiedChar… | confirmed | Installs the base and relative-rail vtables, stores its route references, additional constructor values, and a four-word vector, initializes progress to zero, … |
| `00bfc0e0` | ProxiedCharacterRelativeRailMovementController::~ProxiedCha… | confirmed | Restores the relative-rail vtable, cleans and frees the pointers at this+0x30 and this+0x34 when non-null, then restores the base movement-controller vtable. |
| `00bfc170` | ProxiedCharacterRelativeRailMovementController::ScalarDelet… | confirmed | Calls the relative-rail destructor body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |

**ProxiedCharacterSeekMovementController** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfc190` | ProxiedCharacterSeekMovementController::ResetTargetData | confirmed | Forms a two-word argument pair from the caller's stack values, adds the constant pointer 0x017efda8, and delegates through Target at this+0x1c to 0101c060. |
| `00bfc280` | ProxiedCharacterSeekMovementController::ProxiedCharacterSee… | confirmed | Installs the base and seek-controller vtables, initializes the embedded Target at this+0x1c through 0101c060, stores its arguments, and derives initial orienta… |
| `00bfc440` | ProxiedCharacterSeekMovementController::ScalarDeletingDestr… | confirmed | Calls the seek-controller cleanup body, frees this only when param_2 bit 0 is set, and otherwise returns the original pointer. |
| `00bfc460` | ProxiedCharacterSeekMovementController::Update | confirmed | Clamps the controller's speed at this+0x24 using acceleration at this+0x0c and limit at this+0x10. With a valid controlled object, computes normalized target-r… |
| `00bfc3e0` | ProxiedCharacterSeekMovementController::~ProxiedCharacterSe… | likely | Calls Target cleanup 0101b370 with ECX=this+0x1c, then restores the base ProxiedCharacterMovementController vtable. |

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b3180` | GetEntitySpeed | confirmed | Returns the float at this+0x3ec scaled by DAT_0175d738 when the byte at this+0x650 is set, otherwise scaled by DAT_0175b420; a leaf that reads two fields and o… |
| `0095dae0` | SetHorizontalDirection | confirmed | Masks the input vector to horizontal X/Z components, normalizes its four lanes, stores the resulting direction at this+0x610..0x61c, and optionally computes ya… |
| `00bf9430` | UpdateMovementState 🌐 | confirmed | Returns for a null actor or missing actor handle. Scales the tick delta, evaluates movement flags and path branches, applies transform updates, and stores disp… |
| `008c5a10` | PositionDeltaGate_Check | likely | Computes the four float differences between this+0xd0..0xdc and param_2[0..3], then the Euclidean norm of the four-component difference. If the norm exceeds th… |
| `009607c0` | UpdateActionAndRelatedState | likely | Sets bit 6 at this+0x143, reads a virtual transform, and selects an action value 0x3fac or 0x1609 from the float threshold branches; it also checks related mou… |
| `00963500` | SetPositionUpdateStateFlag | likely | For the owning character, when bit 6 at +0x142 is clear, either sends through FUN_0092e860 when param_2 is nonzero or skips the send; it then sets or clears th… |
| `009635f0` | ClearModeAndSendPositionUpdate | likely | For the owning character with mode bit 1 set, clears bit 1 when param_2 is true and calls FUN_0092ea40; then sets or clears bit 0 according to param_2. |
| `00963ad0` | ComputeMovementTowardTarget | likely | Clears the movement list and optional object, reads current transform, normalizes the target delta, computes and stores velocity-related fields, and may call F… |
| `00bfba20` | ResolveVector4OrDefault | likely | Uses the global four-word default when the object pointer at this is null; otherwise calls that object's vtable+0x10 with param_3. Copies four words to param_2… |
| `01084010` | CurvePointTangentAtProgress | likely | Cdecl with six stack args, plain ret. Writes the sampled point to param_1: the curve object at *param_4 is evaluated through its vtable +0xc at the clamped pro… |
| `01084250` | RelativeCurvePointFrameAtProgress | likely | Cdecl with nine stack args, plain ret. Evaluates two curve objects: *param_4 at progress param_5 and *param_6 at progress param_7, each through vtable +0xc, wi… |
| `01084cd0` | SeekStepSolveTimeAndAdvance | likely | Cdecl with eight stack args, returns 1 or 0 in eax. Builds relative position and velocity from the two pairs of 4-vectors (param_3 vs param_8, param_2 vs param… |
| `0095f700` | ComputeScaledSpeedFactor | speculative | Selects one of two global divisors from bit 7 of this+0x140, clamps the stored float at this+0x3ec to at least param_2, then returns param_2 squared times 0.5 … |
| `0095d930` | — | unknown | Passes scalar param_2 to FUN_00770e70, stores the four generated words at this+0x610..0x61c and copies them once to this+0x620..0x62c; when requested, it deriv… |
| `00963550` | — | unknown | For the owning character, updates bit 1 at byte +0x147 from param_2 and calls FUN_0092ea40 when that mode transition requires it; outside the owner path it onl… |
| `00963670` | — | unknown | For the owning character, sets or clears bit 7 at +0x148 and calls FUN_0092ebf0 on the transition; otherwise it only updates the stored bit. |
| `00963700` | — | unknown | For the owning character, sets or clears bit 6 at +0x148 and calls FUN_0092eda0 on the transition; otherwise it only updates the stored bit. |
| `009637d0` | — | unknown | When mask 0x10502e4 at +0x6f8 is clear, the owning character may call FUN_0092e670; it then updates bits 5, 2 and 3 at +0x142 according to param_2 and clears b… |
| `00a89bc0` | — | unknown | Per-frame update of the click-to-move arrow (3081 bytes): reads the target pair at [this+0x34]+0x260/+0x264 and, when a target exists, fetches a position throu… |

### `world`

**ClientAreaCompositeEffectProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cecbc0` | StoreAndDispatchVector4At50 | confirmed | Copies four input dwords to object offsets 0x50 through 0x5c, then recomputes and dispatches the composite-effect transform. |
| `00cecbf0` | StoreAndDispatchVector4At60 | confirmed | Copies four input dwords to object offsets 0x60 through 0x6c, then recomputes and dispatches the composite-effect transform. |
| `00cecc20` | SetScaleAboveThresholdAndDispatch | confirmed | Stores scale at object offset 0x44 and recomputes the transform only when the input exceeds DAT_01762a30. |
| `00cece40` | ClientAreaCompositeEffectProperty::Update | confirmed | When active and not already updated, checks typed resource readiness, resolves the effect location, queues the local composite effect and recomputes transforms… |

**ClientAreaDataLoader** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b09ec0` | ClearLoaderStateFields | speculative | Clears object byte at offset 0x24 and dword at offset 0x28. |

**ClientAreaDataShaped<ClientAreaDome>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b09db0` | GetAreaShapeData | likely | Returns this plus offset 0x1e0 for the dome-shaped client-area data field. |
| `00b09e10` | GetAreaShapeData | likely | Returns this plus offset 0x1e0 for the second dome data field. |
| `00b0a250` | Method_00b0a250 | likely | For dome-shaped data, chooses a debug color from the flag and draws the stored bounds unless disabled. |
| `00b09e40` | ReturnStaticShapeNameData | speculative | Returns the address of static data at 0181d274; its contents are not proven by this function body. |
| `00b0a100` | TestDomeBounds | speculative | Validates an area object, copies eight float values from a virtual getter, applies a dome bound test, and returns its result. |

**ClientAreaDataShaped<SoeUtil::Math::Box_t<float>_>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b09dd0` | GetAreaShapeData | likely | Returns this plus offset 0x1e0 for the box-shaped client-area data field. |
| `00b09e30` | GetAreaShapeName | likely | Returns the address of static data at 0181d270. |
| `00b0a1a0` | Method_00b0a1a0 | likely | For box-shaped data, copies stored bounds and static parameters, selects debug color, then draws the box. |
| `00b09e70` | ForwardAreaDataBoolResult | speculative | Calls the shared area-data validation helper and returns whether validation succeeded. |

**ClientAreaDataShaped<SoeUtil::Math::Sphere_t<float>_>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0a290` | ReadSphereCenterAndRadius | confirmed | Reads three keyed coordinate values and radius, converts them to floats, and stores center/radius at offsets 0x1e0?0x1f0. |
| `00b09d90` | GetAreaShapeData | likely | Returns this plus offset 0x1e0 for the shaped client-area data field. |
| `00b09e20` | GetAreaShapeName | likely | Returns the literal string sphere. |
| `00b0a160` | Method_00b0a160 | likely | For sphere-shaped data, chooses a debug color from the flag and draws the stored bounds unless disabled. |
| `00b09e50` | ForwardAreaDataBoolResult | speculative | Calls the shared area-data validation helper and returns whether validation succeeded. |

**ClientAreaFloor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b09e90` | WriteFloorShapeTag | likely | Passes the literal key/value pair shape=floor to the shared data writer. |

**ClientAreaMinimapProperty** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ced930` | ClientAreaMinimapProperty::Enter | confirmed | Calls the global client accessor, loads its field at offset 0x3c, obtains a nested context pointer, then tail-jumps to the minimap reset handler. |
| `00ced950` | ClientAreaMinimapProperty::Exit | confirmed | Calls the global client accessor, loads its field at offset 0x4c, obtains a nested context pointer, then tail-jumps to the minimap reset handler. |

**Deep::Scene** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00746210` | — | unknown | Allocates 0x250 bytes through 00d833a6; when the allocation succeeds it runs the Scene constructor 007375d0 on the new block with the argument and returns the … |

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090e8a0` | PlayClient::SetStartingZoneState | speculative | Compares the new zone value with client field +0x788, emits StartingZone transitions when entering or leaving value 3, stores the new value, and updates a bool… |

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa1b0` | HashListMap_MovementController_Construct | confirmed | Constructor of the signature-guid-to-movement-controller map with 1024 buckets: installs the SoeUtil::HashListMap<SoeUtil::StrongType<unsigned __int64,SoeUtilS… |

**WorldCompositeEffectEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fe0a0` | WorldCompositeEffectEvent::ApplyChildAndNotify | speculative | fastcall (ECX=this, no stack argument). Calls the virtual slot +4 of the object at this+0x44 (returns an int); if nonzero, calls 00769710 with this+0x94, the r… |

**WorldMeshEmitterEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eb490` | WorldMeshEmitterEvent::UpdateEmitter | likely | If emitter is empty but source exists, derives parameters, creates and stores emitter, notifies event, and sets completion flag. |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a7840` | SetZoneType | confirmed | Stores the zone-type word at this+400 (0x190) and maps it to a ZoneType name via FUN_00976560: 0 -> TileStatic, 1 -> TileSeamless, 2 -> RuntimeSeamless, 3 -> t… |
| `008d3460` | SetRenderFloraFromZoneData | confirmed | Looks up the render pipeline entry for the names 'Terrain' and 'RenderFlora' through FUN_006feda0 (passing the global 01802bc0 and a temporary IString) and FUN… |
| `008ed660` | SetWorldNameAndLoadRoadmap | confirmed | Thiscall with a world name. Assigns the name into the IString at this+0x180 (00402ce0). Calls 00976560 with the key "World", the value at this+0x184, and ECX =… |
| `008ed7b0` | SetZoneTypeFromName | confirmed | Thiscall with a zone-type string. Copies the string (00402ce0) into a local and logs it through 00976560 with the key "ZoneType". It then compares with 006fc5b… |
| `0093d180` | BeginPlayerZoning | confirmed | Resets loading and activity fields, calls zone loading with destination and transform values, updates player orientation, and raises begin-zoning and zoning ev… |
| `008e4330` | SphereTreeFirstOverlapCandidate | likely | Clears the result slot, expands four query components by the supplied radius, calls the sphere-tree overlap collector, and returns the stored candidate data at… |
| `009613d0` | SetZoneFlagsAndResetQueue | likely | Stores the supplied flags at this+0x6f8; when mask 0x10502e4 matches, clears the list, sets bit 2 at +0x143 and releases the optional object at +0x3fc; always … |
| `00985340` | CreateActorNodeIfMissing | likely | For the manager in thiscall ECX and the forwarded creation arguments, looks up the key through 00984960. On a miss it allocates a node, calls 0096fa70, and cal… |
| `00985550` | RemoveActorNodeAndMaybeUnregisterGuidPair | likely | Receives the manager in ECX and the node as a stack argument (ret 4). If the node is non-null, saves byte node+0x141, calls 00957f90 with the node in ECX and l… |
| `009855d0` | FindNodeByKeyPairAndDestroy | likely | Forms a two-dword key from its stack arguments, calls 00984960 with the manager in ECX, and if a node is returned calls 00985550 with the manager in ECX and th… |
| `00985600` | RebuildIndexesAroundParam2 | likely | If the pointer at [*(this+0x340)+0xc8] differs from param_2, clears that field. If param_2 is null, drains the 100-bucket table, node list, and GUID map and re… |
| `009856d0` | CleanupNodeIndexesAndOwnedObject | likely | Teardown of the node manager under SEH frame LAB_01599e9c: clears the selected-node pointer at [*(this+0x340)+0xc8], drains the 100-bucket table (00984470), de… |
| `00a3bca0` | SetMember24c8FromGlobalService | likely | Creates an object through a global service and stores it in the ref-counted member at +0x24c8: it copies a 16-byte global (DAT_01cb53d0 block) to the stack, ge… |
| `009584b0` | SetTransformAndNotify | speculative | Calls virtual slot 7, copies four transform words through 00beff10, then invokes 00995e10 when linked context field +0x5e8 is nonzero. |
| `00b09eb0` | SetReadyFlagWhenArgumentZero | speculative | Sets object byte at offset 0x20 when the fourth argument is zero. |
| `00cec490` | TransformAndDispatchCompositeEffect | speculative | Builds and combines 4x4 transforms from property rotation, scale, client-area transforms, and effect offsets, then dispatches the resulting transform to each r… |
| `008ff3d0` | — | unknown | Reads an id from object this+0xc8 via vcall slot 7 (+0x1c) and FUN_00790460, and may replace it through FUN_010ba6e0 and the +0x20/+0x1c/+0x2b0/+0xa0 chain. Wh… |
| `00984fe0` | — | unknown | Under an SEH frame, clears the head/tail/count fields and 100-bucket array at offsets +0..+0x10, then clears a second 100-bucket array at +0x1a0..+0x1a8 and st… |

### `world.camera`

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb2bc0` | LoadRailCameras | confirmed | Loader for RailCameras.txt: drains the existing table (008bf730 per node), builds the file path from the argument string with the SoeUtil::IString helpers (004… |
| `00793a20` | EnsureViewAndApplyTransform | likely | Thiscall (ECX = this, 6 stack args). Stores param_5..param_7 and 0 at this+0x390..0x39c. When the pointer at this+0x264 is null it allocates 0x10 bytes with 00… |
| `00bb28c0` | HashListClearAll_8 | likely | Erases the whole table: while the head pointer at this+8 is non-zero it calls 008bf730(this, head), which unlinks the node from the insertion list (+0x30 next … |
| `00bb2960` | ScanEntriesMeasureQuaternionDistance | likely | Walks the insertion-order list at this+8 (next at +0x30) and, for every entry whose dword at +4 equals *(param_2+0x194), resolves the two objects referenced by… |
| `00bb2b10` | HashListInsertTail | likely | Allocates an entry through the owner's virtual allocate slot (*(owner)+4), copies 12 dwords from param_3 into it, stores the key *param_2 at +0x38, appends it … |
| `00793330` | SetPositionPairAndUpdate | speculative | Thiscall (ECX = this) with two float4 vectors (param_2, param_3) and two floats (param_4, param_5). Reads a float through 00769710/00769d90; when it exceeds DA… |
| `007935a0` | SetTanHalfAngleFrustum | speculative | Thiscall (ECX = this, param_2 = angle float). Stores the angle at this+0x434. When this+0x260 is non-null and its +0x18 member is non-null it computes tan(angl… |
| `00bb2720` | — | unknown | Hash lookup with a 64-bucket table: starts at *(this+0x14+(key & 0x3f)*4), follows the chain at node+0x3c and returns the node whose key at +0x38 equals the ar… |

### `world.pathfinding`

**RoadMap** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010baeb0` | RoadMap::Ctor | confirmed | Fastcall, ECX=this. Stores AssetManagement::AssetDeliveryHandlerInterface::vftable at +0 and the global field at +0x18 of the object returned by 00769710 at +4… |
| `010bc310` | RoadMap::DtorBody | confirmed | Fastcall destructor body with this in ECX. Stores RoadMap::vftable (0x018867dc) into [this], calls 010bbc20 on this+8 to clear the node table, then calls the b… |
| `010bc3f0` | RoadMap::ScalarDeletingDtor | confirmed | Thiscall with one flag argument (ret 4). Calls 010bc310 (RoadMap::DtorBody) on this, frees this through the _free thunk when bit 0 of the flag is set, and retu… |
| `010bccd0` | RoadMap::LoadNodesFromBuffer | confirmed | Thiscall with this in ECX and three stack arguments (ret 0xc); the source object is the first stack argument (param_2); the body uses only that argument. Clear… |

**(no class)** (21)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009cb990` | ClearBreadcrumbTargetAndPathState | confirmed | Drains three lists, clears target-state fields, releases the target object, optionally calls a helper for +0xd20, and logs that the target was cleared. |
| `009ccd50` | InitPathStateFromMinimapOptions | confirmed | Initializes three Vector4 lists and path fields, reads several keyed settings, and queries MinimapOptions.PathEnabled when the settings owner is present. |
| `009cd010` | RequestKynapsePath 🌐 | confirmed | Stores requested endpoint, reads current endpoint, checks route and distance state, builds and sends a tunneled packet when allowed, logs the request or excess… |
| `009cd2f0` | HandleKynapsePathReply 🌐 | confirmed | Rejects unexpected replies; on first failure logs and retries fallback, while success records time/state, copies returned nodes into the route list and clears … |
| `009cd6c0` | SetBreadcrumbTarget 🌐 | confirmed | Copies a target vector, ignores nearby unchanged targets unless state is failed, optionally clears existing target, logs target and start positions, then issue… |
| `009cbcf0` | UpdateNodeChainsFromPosition | likely | Checks active-position and distance thresholds, queries 010bba90 for a candidate path point, updates linked Vector4 chains, and adjusts path progress state. |
| `009cd410` | UpdateBreadcrumbPathState 🌐 | likely | Refreshes keyed route settings and timing state, updates path geometry and visuals, clears stale targets, and requests a path when position or timeout threshol… |
| `010bae00` | RoadNodeCtor | likely | Fastcall, ECX=new node. Stores 0xffffffff at +0 (the id sentinel) and zeroes dwords +0x20 through +0x5c (the edge container, list head and bucket pointers, and… |
| `010bb7c0` | NodeListNearestId | likely | Thiscall, ECX=list header (head at +0xc), one float* stack argument = query point (4 floats). Walks the list via the link at node+0x5c, computes the squared di… |
| `010bba90` | PolylineClosestPoint | likely | Stack arguments: query point (float*), path object (int*), output point (float*), output start-node pointer (dword*, may be null, set to 0 first). Returns at o… |
| `010bbdc0` | NodeGraphAStarById | likely | Thiscall with the graph in ECX and three stack arguments (ret 0xc): start id ([ebp+8]), goal id ([ebp+0xc]), output list ([ebp+0x10]). Clears the output list w… |
| `010bc210` | NodeGraphRemoveNode | likely | Thiscall with one stack id (ret 4). Returns 0 when no node has that id (010bb150). Otherwise calls 010bb300 and 010bb630 on the node, frees the node with _free… |
| `010bc410` | NodeTableAddNode | likely | Thiscall with the graph in ECX, an id and a pointer to four dwords (ret 8). Returns 0 when 010bb150 already finds a node with that id. Otherwise allocates a 0x… |
| `010bc4f0` | NodeGraphAddDirectedEdge | likely | Thiscall with the graph in ECX and two stack ids (ret 8). Looks up both nodes with 010bb150. If both exist, the ids differ, and 010bb1a0 finds no edge from the… |
| `010bc610` | NodeGraphLoadFromFile | likely | Thiscall with the graph in ECX and one stack argument, the file name (ret 4). Clears the node table at graph+8 (ECX = graph+8) with 010bbc20, opens the file (F… |
| `010bc8b0` | NodeGraphLinkTwoWay | likely | Thiscall with the graph in ECX and two stack ids (ret 8). Looks up both nodes with 010bb150 and does nothing unless both exist. The cost is the Euclidean dista… |
| `010bca20` | NodeGraphAddProjectedNode | likely | Thiscall with the graph in ECX and one stack argument, a pointer to a point (ret 4). Calls 010bb850 with the point and three output pointers, which yield a 4-d… |
| `010bcb30` | NodeGraphAddPointNode | likely | Thiscall with the graph in ECX, a pointer to four dwords (position) and a neighbour id (ret 8). Allocates a 0x60 node (00d833a6, constructed by 010bae00), take… |
| `010bcc00` | NodeTableRequestLoad | likely | Thiscall with this in ECX and one stack argument, a C string (ret 4). Clears the node table at this+8 with 010bbc20, builds a SoeUtil::IString<char> from the a… |
| `010bcf60` | NodeGraphFindPathBetweenPoints | likely | Thiscall with the graph in ECX and three stack arguments (ret 0xc): start point pointer, goal point pointer, output list pointer. Snaps the start and goal poin… |
| `010bb850` | — | unknown | Thiscall, ECX=graph (node list head at +0xc, id table buckets at +0x18), stack arguments: query point (float*), output point (float*, 4 floats), output node id… |

### `world.proximity`

**ProximityTrackerTemplate<SoeUtil::Math::Sphere_t<float>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010772d0` | ProximityTrackerTemplateSphereCtor | confirmed | Thiscall constructor, same layout as the Vector4 tracker constructor: stores the Sphere tracker vtable, owner at +0x18, element map at +0x40 built by 01076f10,… |
| `01077380` | ProximityTrackerTemplateSphereDtorBody | confirmed | Destructor body for the Sphere tracker: stores the Sphere tracker vtable, calls 01021730 with the tracker as argument and ECX = owner (+0x18), then calls 01076… |
| `01077410` | ProximityTrackerTemplateSphereScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) for the Sphere tracker: calls 01077380, then _free when bit 0 is set. |

**ProximityTrackerTemplate<SoeUtil::Math::Vector<float,4>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010771b0` | ProximityTrackerTemplateVec4Ctor | confirmed | Thiscall constructor (three stack arguments and a byte flag). Stores the tracker vtable, zeroes +8, +0xc, +0x10, +0x14, the registration id at +0x38 and the fl… |
| `01077260` | ProximityTrackerTemplateVec4DtorBody | confirmed | Destructor body: stores the tracker vtable, calls 01022850 with the tracker as argument and ECX = owner (+0x18), then calls 01076c90 on the embedded element ma… |
| `010773f0` | ProximityTrackerTemplateVec4ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls 01077260, then _free on the object when bit 0 of the argument is set. |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01076090` | ConditionalOwnerCall01024dd0 | speculative | Fastcall. When the dword at this+0x38 is zero, calls 01024dd0 with ECX set to the owner pointer at this+0x18 and this as its stack argument. Otherwise returns … |
| `010760a0` | Wrapper_01021730 | speculative | Fastcall forwarder. Pushes this, loads ECX from this+0x18 (owner) and calls 01021730. Returns nothing. |
| `010768d0` | DrainHeadAt48ThenUnlinkMap40 | speculative | Fastcall on an object whose element list head is at +0x48. For each node: calls the stored element's vtable +0x10 with this, then calls this's vtable +8 with t… |
| `01076920` | RemoveElementByKeyFromTrackerMap_Vec4 | speculative | Thiscall on a tracker with one stack argument (an element pointer). Finds the node whose key equals element+0x2c in bucket (key & 0x3f) of the embedded map at … |
| `010769a0` | DrainHeadAt48ThenUnlinkMap40Sphere | speculative | Sphere-element variant of 010768d0: same element loop (element vtable +0x10 with this, this vtable +8 with the element), then unlinks the remaining nodes of th… |
| `010769f0` | RemoveElementByKeyFromTrackerMap_Sphere | speculative | Sphere-element variant of 01076920: same key lookup in the embedded map at this+0x40, element vtable +0x10 and tracker vtable +8 calls, then unlinks the node w… |
| `01077570` | RefreshElementsFromOwnerList_Vec4 | speculative | Thiscall on a tracker (ECX), two stack arguments (ret 8). Adds the tracker's count (vtable +0x18, floored at 1) into the 64-bit value passed to 01022eb0. Walks… |
| `01077740` | RefreshElementsFromOwnerList_Sphere | speculative | Sphere-element variant of 01077570: same count-and-accumulate step through 010234b0, same owner-list walk, but the element link is at +0x34 instead of +0x24. I… |
| `01077860` | SyncOneElementWithPredicates_Sphere | speculative | Sphere-element variant of 01077690: single-element sync on the embedded Sphere map at this+0x40, inserting with 010774d0 when absent and both predicates pass, … |
| `01077690` | — | unknown | Thiscall on a tracker with two stack arguments (ret 8); the pseudocode uses the first as the element pointer. Looks up the element key in its bucket of the emb… |

### `world.railcamera`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0107fb50` | ZeroInitRecord0x30 | speculative | Zero-initialises a record: clears the dwords at +0x0,+0x4,+0x8,+0xc,+0x14,+0x18,+0x1c,+0x28,+0x2c, the bytes at +0x10 and +0x20, and stores the default dword _… |
| `008ee710` | — | unknown | Thiscall with a byte argument. Stores it at this+0xc18. When this+0x6d8 is nonzero it formats it with "%d" and logs it through 006fe530("Controls", "RailCamera… |
| `0107fb90` | — | unknown | Thiscall parser: consumes caret-delimited (0x5e) tokens from the row at param_2 into the record fields: integers via 0072d900 at +0x0, +0x4, +0x8, +0xc, +0x18 … |

## Source shards

[[S0156|S0156]] (1), [[S0171|S0171]] (3), [[S0220 008a3ce0-008a8500|S0220]] (1), [[S0221 008a86b0-008b1d80|S0221]] (1), [[S0222 008b1e00-008b7730|S0222]] (1), [[S0223 008b7790-008bcf40|S0223]] (2), [[S0225 008c52a0-008ca980|S0225]] (1), [[S0227 008d0920-008d5c60|S0227]] (1), [[S0230 008deaaa-008e47b0|S0230]] (1), [[S0232 008e82e0-008ec1d0|S0232]] (1), [[S0233 008ec2a0-008f0cc0|S0233]] (3), [[S0236|S0236]] (6), [[S0237 008ff3d0-00904e70|S0237]] (1), [[S0239 0090a890-009105b0|S0239]] (1), [[S0248 009392c0-0093d740|S0248]] (1), [[S0254 00954ec0-0095a710|S0254]] (1), [[S0255 0095a840-00960520|S0255]] (3), [[S0256 009606e0-00965340|S0256]] (9), [[S0263 00982f10-009897a0|S0263]] (6), [[S0275 009c8e90-009cdee0|S0275]] (7), [[S0295 00a39b00-00a3f350|S0295]] (1), [[S0309 00a87ba0-00a8cdc0|S0309]] (1), [[S0333 00b052f0-00b0a290|S0333]] (17), [[S0366 00bb11b0-00bb7710|S0366]] (5), [[S0381 00bf9430-00bfcf30|S0381]] (17), [[S0415 00cc1a00-00cc6bc0|S0415]] (2), [[S0423 00ceb9b0-00cf13e0|S0423]] (7), [[S0578|S0578]] (17), [[S0579|S0579]] (9), [[S0580 0107d0c0-010829c0|S0580]] (2), [[S0581|S0581]] (3), [[S0591|S0591]] (5), [[S0592|S0592]] (13)
