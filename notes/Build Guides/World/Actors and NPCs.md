---
type: build-guide
section: World
updated: 2026-10-12
---

# Actors and NPCs

Part of [[Build Guides]] › World. Related: [[Animation]], [[Movement and Pathfinding]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

198 accepted functions: 67 confirmed, 63 likely, 42 speculative, 26 unknown. Network-linked (`wire`): 0.

Subsystems: `world.actor` (154), `world.transform` (44)

### `world.actor`

**Actor** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0077bb10` | SetMaterialParameterVec4 | confirmed | Same structure as SetMaterialParameter (0077b9a0) with a four-float value: validates the material index against the part's material count and, when out of rang… |
| `0077c0e0` | AttachChildActor | confirmed | Thiscall with this = the parent actor and param_2 = the child actor. Each actor's base model is taken as the first non-null entry of its model array (+0x130, c… |
| `0077d0f0` | ActorSkeletonAnimationValidate | confirmed | Thiscall on an actor: builds a 'Bad Actor' diagnostic when the skeleton and animation assignments disagree. It checks the animation count at +0x3e4 and the ske… |
| `00788080` | Actor::~Actor | confirmed | Actor destructor body: stores Actor::vftable at +0 and at +0x1c0, releases owned members (reference-counted pointers with LOCK decrement, string buffers, list … |
| `00788990` | Actor::AdjustorDeletingDtor | confirmed | Thunk: sub ecx,0x1c0 (81 e9 c0 01 00 00) then jmp 0078b2c0 (e9 25 29 00 00). It is slot [0] of the secondary Actor vftable at +0x1c0 and adjusts this back to t… |
| `0078aa90` | Actor::Actor | confirmed | Constructor body: writes AssetRequester::vftable and then Actor::vftable at +0x1c0, Actor::vftable at +0, stores the owner argument at +0x1c4, initialises flag… |
| `0078b2c0` | Actor::DeletingDtor | confirmed | Scalar deleting destructor (Actor::vftable slot [0]): calls 00788080(this) (the destructor body), then _free(this) when bit 0 of the flag argument is set; retu… |
| `0078b5f0` | Actor::Update | confirmed | Per-frame update with a float delta argument. Calls the hook at +0x368 (vtable +0x30) and returns when that returns 0. Advances the +0x798 accumulator, eases t… |
| `0077b2f0` | ActorVirtual12_DetachFromParent | likely | Actor virtual slot [12] (vtable +0x30): calls 0075c060 on this, then calls 00779f90(this) (ActorDetachFromParent) when the parent pointer at this+0x364 is non-… |
| `0078dda0` | Actor::CreateSubobjectByKind | likely | Switch on the kind in param_3 (0..0xd, default grouped with 0xc). Each kind calls operator new (00d833a6) with a kind-specific size (0x54, 0x2c, 0x80, 0x524, 0… |
| `00773a60` | Actor::PointDistanceMinusRadius | speculative | Thiscall, one float4 pointer arg (ret 4): takes a radius from 007707f0 and, when bytes +0x22d, +0x22e, +0x232, +0x233 and +0x234 are all non-zero, sets byte +0… |
| `00774a90` | Actor::BoundsSphereQuery | speculative | Thiscall (ECX = object, one float stack arg). Returns 0 when the owner test fails (this+4 null and this+8 zero, or the owner from 00729610 / this+4+0x38 is zer… |
| `00787e70` | Actor::TimedNodeSweep | speculative | Returns unless byte this+0x240 is set. Reads the 64-bit ms tick (007027f0) and compares it with the stored tick at +0x768/+0x76c. When more than 60000 ms have … |
| `00788a70` | Actor::QueryKeyedFloat | speculative | Returns 0 unless the second stack argument equals -0x4ebba5b5. Then writes one float to the out pointer (first stack argument): DAT_0175b420 minus max(this+0x2… |
| `00774940` | — | unknown | Thiscall, one char arg (ret 4): updates the object at this+0x29c (clearing or setting its +0x23d state through 00fd8160 or 00fdb600 depending on bit 1 of byte … |
| `0077b9a0` | — | unknown | Reads the model part at this+0x130 indexed by modelType (param_2) and its material count at part+0x84->+0x10. When materialIndex (param_3) is negative or not b… |
| `0077d450` | — | unknown | Thiscall with param_2 = a reference-counted lighting data object: adds a reference to param_2, releases the previous object at this+0x6f4 (dropping its count a… |
| `0077f870` | — | unknown | Actor::vftable slot [3]; one argument (context pointer). Returns when byte this+0x22f is zero. Owner object is this+4 (or accessor 00729610 when that is null);… |
| `0078c1f0` | — | unknown | Switch on event code param_3 (0..13). Codes 0-4 and 0xd: checks the model definition at this+0xd0 and the visibility meshes at +0x40/+0x44 and logs the 'No mod… |

**ActorCompositeEffectReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0077ab60` | ActorCompositeEffectReference_ScalarDtorWrap | likely | Scalar deleting shape with an SEH frame: calls 007772b0 (the ActorCompositeEffectReference body) on this, then calls _free(this) only when bit 0 of the second … |

**ActorGroupRequestor** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007993d0` | ActorGroupRequestorDtorBody | confirmed | Fastcall (ecx = this). Stores ActorGroupRequestor::vftable at +0. If the pointer at +0x1c is non-NULL: when its count at +4 is 1 it calls the vtable slot at +0… |
| `007995d0` | ActorGroupRequestorScalarDtor | confirmed | Virtual slot [0] of ActorGroupRequestor: calls 007993d0 on this, frees the block through _free when bit 0 of its flag argument is set, returns this. |
| `007997a0` | ActorGroupRequestorOnAssetLoadFailed | confirmed | Virtual slot [2] of ActorGroupRequestor, thiscall with one stack argument (asset name). Formats 'Could not load asset %s for actor group requestor %s' with the… |
| `007994b0` | ActorGroupRequestorSetFlag28 | likely | Virtual slot [4] of ActorGroupRequestor (bytes c6 41 28 01 c3): stores 1 into the byte at this+0x28. |
| `007994c0` | ActorGroupRequestorGetFlag28 | likely | Virtual slot [5] of ActorGroupRequestor (bytes 8a 41 28 c3): returns the byte at this+0x28 in AL. |
| `0079a880` | ActorGroupRequestorAcceptAsset | likely | Virtual slot [1] of ActorGroupRequestor, thiscall with two stack arguments. When arg3 is 0, dynamic-casts arg2 from AssetManagement::Asset to ActorGroupDefinit… |
| `0079c210` | ActorGroupRequestorCreateAsset | likely | Virtual slot [3] of ActorGroupRequestor, two stack arguments. When arg2 is non-zero returns 0. Otherwise allocates 0x124 bytes through 00d833a6; when the alloc… |
| `00799700` | — | unknown | Thiscall with five stack arguments. Stores AssetManagement::AssetRequestInterface::AssetRequester::vftable and the owner pointer arg1 at +4, then ActorGroupReq… |

**ActorHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00798db0` | ActorHandler::Dtor | confirmed | Stores ActorHandler::vftable (017e952c) at this+0 and returns. Seven bytes (c7 01 imm32; c3), no other field touched. |
| `00798e80` | ActorHandler::ReleaseIString | likely | Virtual method of the ActorHandler family at vtable slot [6]. When its 4th stack argument is greater than 0, atomically decrements the refcount header at (2nd … |

**ActorManager** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0079c820` | ActorManagerRecordDrawnActor | confirmed | Virtual slot [5] of ActorManager, thiscall with one stack argument (actor). If the actor's +8 link is non-NULL and the link's +4 word is non-NULL it counts a '… |
| `0079d450` | ActorManagerDtorBody | confirmed | Fastcall (ecx = this). Stores ActorManager::vftable at +0, runs 0079cdf0 (drains the actor list into the pool), releases the list block at +0x1054 through 006f… |
| `0079da80` | ActorManager::~ActorManager | confirmed | Calls 0079d450 on this, then if bit 0 of the byte stack argument is set calls _free(this); returns this (ret 4, ECX=this). Scalar deleting destructor shape. |
| `0079c460` | ActorManagerGetFloat1060 | likely | Virtual slot [8] of ActorManager (bytes d9 81 60 10 00 00; c3): loads the float at this+0x1060 onto the x87 stack and returns it. |
| `0079c470` | ActorManagerGetByte1064 | likely | Virtual slot [7] of ActorManager (bytes 8a 81 64 10 00 00; c3): returns the byte at this+0x1064 in AL. |
| `0079c7c0` | ActorManagerFindById | likely | Virtual slot [4] of ActorManager, thiscall with one stack argument (32-bit id). Hashes the id with & 0x3ff into the 1024-bucket table at this+0x44, walks the c… |
| `0079c7f0` | ActorManagerFindByIdAlt | likely | Virtual slot [3] of ActorManager, byte-identical to 0079c7c0 (ActorManagerFindById): hashes the 32-bit id argument with & 0x3ff into the table at this+0x44, wa… |
| `0079c950` | ActorManagerSetFloat1060Clamped | likely | Virtual slot [9] of ActorManager, thiscall with one float argument. Raises the argument to the floor DAT_017edf3c when it is not greater than the floor, then s… |
| `0079cd90` | ActorManagerRemoveById | likely | Virtual slot [2] of ActorManager, thiscall with one stack argument (id). Finds the entry through this vtable's slot [4] (ActorManagerFindById). When found: unl… |
| `0079d5e0` | ActorManagerSetByte1064 | likely | Virtual slot [6] of ActorManager (bytes: store of the byte argument): writes the byte argument to this+0x1064 and returns nothing. |
| `0079d5f0` | ActorManagerCreateEntry | likely | Virtual slot [1] of ActorManager, thiscall with one stack argument (request block). Increments the id counter at +0x30 (skipping 0), allocates an entry through… |
| `0079d8c0` | — | unknown | Stores ActorManagerInterface::vftable then ActorManager::vftable at this; zeroes this+0x4..+0x40 (selected dwords); memsets 0x1000 bytes at this+0x44; stores D… |

**ActorManagerInterface** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0079c390` | ActorManagerInterfaceDtorBody | confirmed | Fastcall (ecx = this). Stores ActorManagerInterface::vftable (017edf4c) at +0 and returns. Seven bytes (c7 01 imm32; c3). |
| `0079c400` | ActorManagerInterfaceScalarDtor | confirmed | Thiscall (ecx = this, one flag argument). Stores ActorManagerInterface::vftable at +0 and frees this through _free when bit 0 of the flag is set; returns this. |
| `0079c3d0` | ActorManagerInterfaceConstFloat | speculative | Virtual slot [8] of ActorManagerInterface (bytes d9 05 imm32; c3): loads the float constant at 017ed7d4 onto the x87 stack and returns it. |

**ActorParticleReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0077ab00` | ActorParticleReference_ScalarDtorWrap | likely | Scalar deleting shape with an SEH frame: calls 00777210 (the ActorParticleReference body) on this, then calls _free(this) only when bit 0 of the second argumen… |

**ActorSockets** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c65c0` | ActorSockets::ScalarDeletingDtor | confirmed | Virtual slot [0] (30 bytes): calls 007c58d0 (the ActorSockets destructor body), then calls _free on the object when the deleting flag bit 0 is set, and returns… |
| `008b8420` | ActorSockets::ActorSockets | confirmed | Installs ActorSockets vtable, stores param_2 at fields +4 and +0x258, clears two 0x100-byte arrays and initializes the intervening counters and trailing fields. |
| `007c58d0` | ActorSockets::DtorBody | likely | Fastcall, ECX = this (365 bytes, SEH frame). Stores the ActorSockets vtable at +0. Calls 007c5690 with ECX = this+0x228 (8-bucket name table: frees its nodes),… |
| `007c65e0` | ActorSockets::LoadFromOwnBuffer | likely | Virtual slot [3] (229 bytes). Reads the object's byte buffer (data at +4, size at +8). Builds a temporary byte array (SoeUtil::Array<unsigned char,0,1> layout)… |
| `007c47b0` | ActorSockets::SetFlag25C | speculative | Virtual slot [4] (10 bytes): stores byte 1 at this+0x25c and returns. No other effect. |

**ActorSoundReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0077aaa0` | ActorSoundReference_ScalarDtorWrap | likely | Scalar deleting shape with an SEH frame: calls 00777170 (the ActorSoundReference body that stores its vtable and releases members) on this, then calls _free(th… |

**AppModelEntity** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ab8b0` | AppModelEntity::SetFlagBit1 | confirmed | Thiscall with one argument: if bit 1 of the ushort flag word at this+0x10 differs from the argument byte, it sets bit 1 to the argument, calls 00735a30 when th… |
| `007ab900` | AppModelEntity::SetFlagBit1On | confirmed | Secondary-base vtable entry (AppModelEntity vftable +0x12c, slot [1]): if bit 1 of the flag word at this+0x10 is clear it sets that bit, calls 00735a30 when th… |
| `007abad0` | AppModelEntity::DtorBody | confirmed | Destructor body: stores AppModelEntity::vftable at this[0], at this+0x12c and at this+0x130, then stores SoeUtil::RefCounted::vftable at this+0x130; calls 0079… |
| `007abb60` | AppModelEntity::AdjustThunk12cToScalarDtor | confirmed | Adjustor thunk (11 bytes): subtracts 0x12c from ECX and jumps to 007abf70 (the scalar deleting destructor of AppModelEntity). |
| `007abb70` | AppModelEntity::AdjustThunk130ToScalarDtor | confirmed | Adjustor thunk (11 bytes): subtracts 0x130 from ECX and jumps to 007abf70 (the scalar deleting destructor of AppModelEntity). |
| `007abf70` | AppModelEntity::ScalarDeletingDtor | confirmed | Scalar deleting destructor (AppModelEntity vftable slot [0]): calls 007abad0 (dtor body) on this, then frees this with _free when bit 0 of the argument byte is… |
| `007abb90` | AppModelEntity::AssignBlock140 | likely | Virtual slot [7] of AppModelEntity: copies 16 dwords (0x40 bytes) from the argument buffer into this+0x140, then if the pointer at this+0xc is non-null calls i… |
| `007abbd0` | AppModelEntity::ResetHandleState | likely | Virtual slot [12] of AppModelEntity: calls 0075c060 (unlabelled) with ECX=this, then if the ushort state at this+0x182 is below 2 it clears the dword at this+0… |
| `007abcd0` | AppModelEntity::MinOfSlot9AndAttachment | likely | Virtual slot [9] of AppModelEntity, float result: starts from the float at DAT_01be0240; if the pointer at this+8 and its +4 target are non-null it replaces th… |
| `007abf90` | AppModelEntity::VirtualSlot10_ReleaseIfLastRef | likely | Virtual slot [10] of AppModelEntity: calls 00765080 with the argument, then when the argument is 0 and the dword at this+0x134 is 1 it calls 007abc00 (ReleaseR… |
| `007abfc0` | AppModelEntity::VirtualSlot8_ReleaseIfFlagLastRef | likely | Virtual slot [8] of AppModelEntity, one char argument: when the argument is non-zero and the dword at this+0x134 is 1 it calls 007abc00 (ReleaseRefCount); othe… |
| `007abfe0` | AppModelEntity::Slot2_AdvanceTimedPhase | likely | Virtual slot [2] of AppModelEntity, float delta: returns when bit 0 of the ushort at this+0x180 is set; otherwise it adds the delta to the time at +0x184 and m… |
| `007ac200` | AppModelEntity::ReleaseRef | likely | Thiscall with one char argument: when the argument is non-zero it calls 007abc00 (ReleaseRefCount); then it atomically decrements the dword at this+0x134 and, … |
| `007ab850` | AppModelEntity::GetSubobjectAt140 | speculative | Virtual slot [20] (+0x50) of AppModelEntity: returns this+0x140 (bytes 8d 81 40 01 00 00 c3, lea eax,[ecx+0x140]; ret). |
| `007ab860` | AppModelEntity::Slot3CallIfFlag180Bit0Clear | speculative | Virtual slot [3] (+0xc) of AppModelEntity: if bit 0 of the byte at this+0x180 is clear it calls 00764f80 with ECX=this and returns; otherwise it returns withou… |
| `007ac680` | AppModelEntityDtorBodyCall | speculative | Stores ECX as the argument and calls 007abad0 (AppModelEntity destructor body), then returns without freeing the object. |
| `007abc00` | — | unknown | Once only (guarded by bit 0 of the ushort at this+0x180, set on entry): sets 0 at this+0x368 (0076b950), takes the global object from 00769710 and calls its vt… |
| `007ac330` | — | unknown | Thiscall constructor: calls 00766d80 (Deep::AttachableEntity ctor) and 0071b950, stores ActorHandler::vftable then AppModelEntity::vftable at +0x12c and the Ap… |

**PlayerSphereTrackable** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfe0f0` | PlayerSphereTrackable::Ctor | confirmed | Stores PlayerSphereTrackable::vftable at the object start, then calls 01085190 with the stack-cookie value. |
| `00bfe340` | PlayerSphereTrackable::CtorWithPayload | confirmed | Calls 01085690 with arguments (param_2,1,0), stores param_3 in field +0x40, installs PlayerSphereTrackable::vftable, and calls 01084e80. |
| `00bfe3b0` | PlayerSphereTrackable::ScalarDeletingDtor | confirmed | Restores PlayerSphereTrackable::vftable, calls 01085190, and frees this only when flag bit 0 is set; otherwise returns this. |
| `00bfe410` | — | unknown | If the pointer at this+0x40 is non-null, copies eight dwords from that object into its +0x1820 block and returns that block; otherwise returns this+0x50. |

**PlayerTrackable** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bfe210` | PlayerTrackable::CtorWithPayload | confirmed | Calls 01085690 with arguments (param_2,1,0), stores param_3 in field +0x40, installs PlayerTrackable::vftable, and calls 01084e80. |
| `00bfe280` | PlayerTrackable::ScalarDeletingDtor | confirmed | Restores PlayerTrackable::vftable, calls 01085190, and frees this only when flag bit 0 is set; otherwise returns this. |
| `00bfe090` | — | unknown | Stores PlayerTrackable::vftable at the object start, then calls 01085190 with the stack-cookie value. |
| `00bfe2e0` | — | unknown | If the pointer at this+0x40 is non-null, copies eight dwords from that object into its +0x1820 block and returns that block; otherwise returns this+0x50. |

**ProxiedObject** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009706d0` | ProxiedObject::SubobjectAt20 | confirmed | Returns the input pointer plus 0x20 bytes. |
| `009706e0` | ProxiedObject::ReturnFirstStackArgument | confirmed | Loads the dword at [ESP+4], returns it in EAX, and executes ret 8; ECX is untouched. |
| `009706f0` | ProxiedObject::GetField48 | confirmed | Reads and returns the dword at this+0x48. |
| `00970700` | ProxiedObject::SubobjectAt4C | confirmed | Returns the input pointer plus 0x4c bytes. |
| `00beff10` | ProxiedObject::SetTransform | confirmed | Copies four 32-bit values from the input transform into object offsets 0x20 through 0x2c. |
| `00beff50` | ProxiedObject::SetOverheadValue | confirmed | Stores the supplied value at offset 0x4c, then obtains overhead element set type 2 and sends the value through virtual slot +0x18. |
| `00bf00e0` | ProxiedObject::Ctor | confirmed | Initializes object fields, installs ProxiedObject vtable, initializes a shared string, sets a color default, and conditionally creates an overhead element set … |
| `00bf0240` | ProxiedObject::Dtor | confirmed | Restores the ProxiedObject vtable and decrements the owned shared string reference, invoking its release slot when the count expires. |
| `00bf0330` | ProxiedObject::ScalarDeletingDtor | confirmed | Calls the ProxiedObject destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `00bf07c0` | ProxiedObject::SpawnOverheadEffect | confirmed | If an overhead set exists, derives effect parameters from object and input fields, allocates a 0x140-byte effect, creates it through FUN_0096f540, then release… |
| `00bf0a30` | ProxiedObject::SpawnOverheadEffectVariant | confirmed | If an overhead set exists, computes effect parameters from input state, allocates a 0x140-byte object, creates it through FUN_0096f540, and releases the tempor… |
| `00bf0c10` | ProxiedObject::SpawnOverheadEffectWithParams | confirmed | If an overhead set exists, allocates a 0x140-byte effect object, prepares fixed parameter words, calls FUN_0096f540, and releases the resulting reference. |
| `00bf04f0` | ProxiedObject::UpdateOverheadChatBubble | likely | When an overhead set exists, computes timing and color parameters, allocates an OverHeadChatBubbleElement, creates it, then updates existing elements in revers… |
| `00bf01e0` | ProxiedObject::GetTransformData | speculative | Returns the address of the object's data at offset 0x38. |
| `00bf0d20` | ProxiedObject::SpawnTimedOverheadEffect | speculative | If an overhead set exists, computes a scaled duration, allocates a 0x140-byte effect object, calls FUN_0096f540, and releases the resulting reference. |

**ProximityTrackableMonitorTemplate<SoeUtil::Math::Sphere_t<float>_>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01085770` | ProximityTrackableMonitorTemplate_Sphere::RemoveTrackerByKey | confirmed | Thiscall, one stack arg (object whose dword key is at +0x38), ret 4. Searches bucket (key & 0x3f) of the table at this+0x54 for a node whose key at +0xc equals… |
| `01086020` | ProximityTrackableMonitorTemplate_Sphere::Ctor | confirmed | Thiscall with a stack owner pointer (param_2), a dword (param_3) and a byte flag (ret 0xc). Stores the Sphere trackable vftable and field defaults (+0x18 = 0xf… |
| `010860d0` | ProximityTrackableMonitorTemplate_Sphere::DtorBody | confirmed | Fastcall dtor body (EH variant), ECX = this. Stores the Sphere monitor vftable at +0x0, runs the Sphere HashList -2 dtor body 01085a80 on the embedded list at … |
| `01086160` | ProximityTrackableMonitorTemplate_Sphere::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 010860d0 (Sphere monitor dtor body) on this, then if bit 0 of the flag is set calls the free thunk 00d82463 on th… |
| `010862f0` | ProximityTrackableMonitorTemplate_Sphere::AddTrackerByKey | confirmed | Thiscall with one stack arg X, ret 4. Reads the dword key at X+0x38, adds 0x40 to ECX and calls 01086220 with the key pointer and the address of X, so the node… |

**ProximityTrackableMonitorTemplate<SoeUtil::Math::Vector<float,4>_>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01085710` | ProximityTrackableMonitorTemplate_Vec4::RemoveTrackerByKey | confirmed | Thiscall, one stack arg (object whose dword key is at +0x38), ret 4. Reads the key, looks in bucket (key & 0x3f) of the table at this+0x54 for a node whose key… |
| `01085f00` | ProximityTrackableMonitorTemplate_Vec4::Ctor | confirmed | Thiscall with a stack owner pointer (param_2), a dword (param_3) and a byte flag. Stores ProximityTrackableTemplate<Vector<float,4>>::vftable and the same fiel… |
| `01085fb0` | ProximityTrackableMonitorTemplate_Vec4::DtorBody | confirmed | Fastcall dtor body (EH variant), ECX = this. Stores the monitor vftable at +0x0, runs the Vector4 HashList -2 dtor body 010859e0 on the embedded list at this+0… |
| `01086140` | ProximityTrackableMonitorTemplate_Vec4::ScalarDeletingDtor | confirmed | Thiscall with one byte flag arg, ret 4. Calls 01085fb0 (Vector4 monitor dtor body) on this, then if bit 0 of the flag is set calls the free thunk 00d82463 on t… |
| `010862c0` | ProximityTrackableMonitorTemplate_Vec4::AddTrackerByKey | confirmed | Thiscall with one stack arg X, ret 4. Reads the dword key at X+0x38, adds 0x40 to ECX (the monitor's embedded 64-bucket table) and calls 01086180 with the key … |

**ProximityTrackableTemplate<SoeUtil::Math::Sphere_t<float>_>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01085190` | ProximityTrackableTemplate_Sphere::DtorBody | confirmed | Fastcall dtor body, ECX = this. Stores ProximityTrackableTemplate<Sphere_t<float>>::vftable at +0x0, then loads ECX from this+0x34 and calls 01023e00 on that m… |
| `010852b0` | ProximityTrackableTemplate_Sphere::ScalarDeletingDtor | confirmed | Thiscall with one flag stack arg, ret 4. Saves this in esi, loads ECX from this+0x34 and calls 01023e00 on that member, stores ProximityTrackableTemplate<Spher… |
| `01085690` | ProximityTrackableTemplate_Sphere::Ctor | confirmed | Thiscall with three stack args plus a byte flag (ret 0xc). Stores ProximityTrackableTemplate<Sphere_t<float>>::vftable at +0x0; zeroes +0x8, +0xc, +0x10, +0x14… |

**ProximityTrackableTemplate<SoeUtil::Math::Vector<float,4>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01085180` | ProximityTrackableTemplate_Vec4::DtorBody | confirmed | Fastcall dtor body, ECX = this. Stores ProximityTrackableTemplate<Vector<float,4>>::vftable at +0x0, then loads ECX from this+0x34 and calls 01023b60 on that m… |
| `01085280` | ProximityTrackableTemplate_Vec4::ScalarDeletingDtor | confirmed | Thiscall with one flag stack arg, ret 4. Saves this in esi, loads ECX from this+0x34 and calls 01023b60 on that member, stores ProximityTrackableTemplate<Vecto… |

**SoeUtil::RefObjectPool<AppModelEntity,0>::Node** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ac550` | RefObjectPoolNodeAppModelEntityCtor | confirmed | Constructor of a pooled AppModelEntity node: dereferences eight pointer arguments and passes the values to 007ac330 (AppModelEntity ctor), stores the Node vtab… |
| `007ac660` | PoolNode_AdjustThunk12c | confirmed | Adjustor thunk (11 bytes): subtracts 0x12c from ECX and jumps to 007ac860 (the pool node scalar deleting destructor). |
| `007ac670` | PoolNode_AdjustThunk130 | confirmed | Adjustor thunk (11 bytes): subtracts 0x130 from ECX and jumps to 007ac860 (the pool node scalar deleting destructor). |
| `007ac860` | PoolNode_ScalarDeletingDtor | confirmed | Scalar deleting destructor (Node vtable slot [0]): calls 007abad0 (AppModelEntity destructor body) on this, frees this with _free when bit 0 of the argument by… |
| `007ad2c0` | PoolNode_ReleaseAndDestroy | likely | Virtual slot [1] of the secondary Node vtable at +0x130: adjusts this by -0x130 to the primary object, calls the primary vtable slot [0] with 0 (destruct witho… |

**(no class)** (41)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00778d50` | ListDeleteObjectsAndDrain | likely | Fastcall (ECX = owner). Walks the list whose head is at owner+0x578 (links at +4); for each entry with a non-null object at +0 it calls that object's vtable sl… |
| `00779ea0` | SetFloatClampedMin_2e8 | likely | Stores at this+0x2e8 the larger of the float constant at 017ebbb8 and param_2 (a clamped-minimum store), then calls 00777bb0 with no arguments and returns. |
| `00779ed0` | GetModelStringOrStaticEmpty | likely | Initialises once (guard bit at DAT_01bdfdbc, with an atexit registration of 01746b30) a static SoeUtil::IString<char> at DAT_01bdfdac; then returns the string … |
| `00779f90` | ActorDetachFromParent | likely | When param_2 is non-null: removes param_2 from the child list at param_1+0x608 (node removal through 00771d80 when the node's first dword equals param_2), clea… |
| `0077a130` | NamedNodeTransformLookup | likely | Resolves a named sub-object of a model context: returns -1 only when the name is empty or no context exists and both output pointers are null. Otherwise it cal… |
| `0077cf80` | ResolveChildAttachmentIndex | likely | Thiscall with this = a parent actor and param_2 = a child entry. When the child's name at param_2+0x3c is non-empty and the manager at 00769710 reports +0xf4 n… |
| `00780ee0` | UpdateWorldBounds_00780ee0 | likely | Fastcall (ECX=this): sets the byte at this+0x22d to 1, copies eight floats from (*(this+0x130)+0x10)+0x10 into a local block, scales them by this+0x2e8 (times … |
| `007ad040` | AppModelEntityPoolNew | likely | Allocates one element through 007ace80 (guarded pool allocation); when it returns non-zero it constructs an AppModelEntity pool node in it through 007ac550 wit… |
| `007c5a40` | Dword4Setter | likely | Virtual slot [1] (10 bytes): stores the second stack dword at this+4 and returns. Slot [1] of ActorSockets (017efdf4) and of other RTTI classes listed for that… |
| `007c5a50` | Dword4Getter | likely | Virtual slot [2] (4 bytes): returns the dword at this+4 (fastcall, ECX = this). |
| `007c5a60` | LoadSocketElements | likely | Fastcall, ECX = holder (301 bytes). Looks up the child element 'Sockets' (00d50af0); when found, iterates its 'Socket' children with 00d50b60. For each child i… |
| `007c5ba0` | LoadBoneSocketNodes | likely | Fastcall-style with one element argument and one holder argument (931 bytes). Iterates the child elements named BoneSocket of the given element. For each: read… |
| `007c5f50` | LoadSkeletonGroups | likely | Fastcall, ECX = holder (234 bytes). Iterates child elements named Skeleton (00d50af0, then 00d50b60). For each: reads its name attribute (00d517f0 with DAT_017… |
| `007c6040` | LoadModelGroups | likely | Fastcall, ECX = holder (255 bytes). Same loop as 007c5f50 for child elements named Model: each group is built with 007c5830 from its name attribute (00d517f0 w… |
| `007c6140` | LoadActorSocketsDocument | likely | Plain function, one stack argument (the parse buffer or holder; 168 bytes). Calls 00d51ba0 and 00d549d0(arg,0,0) to set up the document, gets the current root … |
| `007c61f0` | FindSocketByName | likely | Thiscall, ECX = holder (4824 bytes): args: socket name, skeleton (or model) name string, a context for the skeleton/model name, and an error-log object (may be… |
| `008b5ec0` | GetPairFromSlot3c8 | likely | Copies the dwords at this+0x3c8 and this+0x3cc into param_2[0] and param_2[1]; a leaf accessor that reads nothing else. |
| `0096d370` | UpdateActorMotionStep | likely | Thiscall(this=P, float, float). When P[0x71] is zero and the position/direction vector equals the constants at 0x190..0x193 (or the flag at P+0x143 bit 2 is cl… |
| `00beff30` | ProxiedObject::SetChildFlag | likely | If object offset 0x48 is nonzero, writes the supplied byte to byte offset 0x24 of that referenced object. |
| `00773ce0` | ListRemoveNodeByKey | speculative | Thiscall, one stack arg (key; ret 4): walks the list headed at this+0x710 (node next +4, prev +8); the key of a node is the dword returned by vtable slot +0x18… |
| `00773db0` | ListClearByKeys | speculative | Fastcall (ECX = this): walks the list at this+0x710 (next +4); for each node computes its key from (node object)+0x2c vtable slot +0x18 and calls 00773ce0 with… |
| `007794f0` | Wrapper_00777170 | speculative | Frame-protected thiscall wrapper: forwards ECX (this) unchanged to 00777170 inside an SEH frame, restores the frame and returns. |
| `00779540` | Wrapper_00777210 | speculative | Frame-protected thiscall wrapper: forwards ECX (this) unchanged to 00777210 inside an SEH frame, restores the frame and returns. |
| `00779590` | Wrapper_007772b0 | speculative | Frame-protected thiscall wrapper: forwards ECX (this) unchanged to 007772b0 inside an SEH frame, restores the frame and returns. |
| `0077b400` | TwoStringPredicateRefresh_1c4 | speculative | Returns without action unless the pointer at this+0x1c4 is non-null. Otherwise it scalar-deletes the objects at this+0x278 and this+0x204 (virtual slot [0] wit… |
| `0077bcc0` | ModelPartBindingRebind_64Bucket | speculative | Indexes the 12-byte entry at (this+0x130 part)+0x84+0xc by param_3, then looks up key param_4 in the 64-bucket table at entry+0xc+0xf0 (chain nodes compare +0x… |
| `0077be70` | SetMaterialVec4ChainAffine | speculative | Returns 0 unless this+0x2b0 (model) is set and 007b7000(param_3) yields an entry. For each entry in the chain (advanced by 007b7090), it maps the four floats a… |
| `0079c620` | NodePairMoveLinks | speculative | Thiscall (ecx = destination holder, one stack argument = source node). Unlinks the source from the chain formed by its +0x1d0 and +0x1d4 links: when +0x1d0 is … |
| `0079c690` | HashChainUnlinkById1024 | speculative | Thiscall (ecx = table holder, one stack argument = entry). Computes the bucket at table+0x10 + (entry id at +0x1cc & 0x3ff) * 4, walks the chain through the ne… |
| `0079ca50` | ListPopHeadAndUnlink | speculative | Fastcall (ecx = holder with head at +4 and count at +0xc). When the count is positive, takes the head entry, removes it from its id bucket through 0079c690, mo… |
| `0079cdf0` | EntryListDrainToPool | speculative | Fastcall (ecx = holder with list head at +0x38, tail at +0x3c, count at +0x40). Loops while the count is positive: takes the head entry, unlinks it from its id… |
| `007c3240` | AdoptSourceRefAndResolveName | speculative | Thiscall, one stack argument P. Stores P at this+0xa0 and increments P's count at +8, calling P's vtable[+0x24] when the count was zero. If P's string at +0x2c… |
| `007c6800` | ConditionalNodeListFlush | speculative | Thiscall, one object argument (68 bytes). When the dword at arg+0x36c is nonzero, calls 007787f0 with this+0x314, then calls 00786e40 on each node of the list … |
| `01084e80` | CallMember34Init_01024ea0 | speculative | Fastcall, ECX = this, no stack args. Pushes ECX, loads ECX from this+0x34 (the embedded member, not this) and calls 01024ea0 on that member; returns with plain… |
| `0077b710` | — | unknown | With a non-empty name at param_3 and a slot index 0..4 (mapped to the same request code), sends the request through the handler at this+0x1c4 vtable slot [3] w… |
| `0077bfd0` | — | unknown | When this+0x4b8 is non-null: fills a temporary SoeUtil::List<int,-1> through 007b74e0(param_2, param_3, list), then for each listed index in range of the objec… |
| `007abd30` | — | unknown | Calls 007abc00 (ReleaseRefCount) and then stores 0 at param_1+0x5c (byte offset 0x170 as the pseudocode is written against the int pointer). |
| `008b3320` | — | unknown | Stores param_2 into this+0x1818, then copies 8 dwords starting at this+0x20 into this+0x1820 while loading this+0x20..+0x2c into a local block first; the effec… |
| `0096b520` | — | unknown | Thiscall(param_1=P, char flag, param_3, param_4 float fade time). Sets bit 0x08 of byte P+0x144 and bit 0x80 of P[0x1be]; calls 0095e530; sets bit 0x04 of byte… |
| `00bfe170` | — | unknown | Copies eight dwords assembled from offsets 0x20 through 0x2c and 0x1818 into the eight-dword block beginning at this+0x1820. |
| `00bfe1c0` | — | unknown | Copies eight dwords assembled from offsets 0x20 through 0x2c and 0x1818 into the eight-dword block beginning at this+0x1820. |

### `world.transform`

**Actor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00788a30` | Actor::CopyMatrix16AndNotify | speculative | Same body as 007889a0: copies 16 dwords from the argument into this+0x160, calls the vtable slot +0x18 of the object at this+0xc with the argument when non-nul… |

**Deep::ModelEntity** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ac4d0` | ModelEntity::ReturnPtrAt1a0 | confirmed | Virtual slot [21] of Deep::ModelEntity (Actor::vftable shares it): returns the dword at this+0x1a0. |
| `010ac4e0` | ModelEntity::ReturnAddrOf160 | confirmed | Virtual slot [20] of Deep::ModelEntity (Actor::vftable shares it): returns the address this+0x160 of the embedded 16-float block. |
| `010acf70` | ModelEntity::SetModelDefinition | confirmed | Sets model slot param_3 of the entity (pointer array at this+0x130, count at +0x134): grows the array through 010accd0 when needed, deletes the previous model … |
| `010ad190` | ModelEntity::~ModelEntity | confirmed | Destructor body of Deep::ModelEntity: stores the vtable, deletes each Model in the array at this+0x130 through its vtable slot [0] (flag 1), releases the refer… |
| `010ad2a0` | ModelEntity::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape for Deep::ModelEntity (vtable slot [0]): calls the destructor body 010ad190 on ECX, frees this with _free when flag bit 0 is s… |
| `010ac3d0` | ModelEntity::VirtualSlot19 | likely | Virtual slot [19] of Deep::ModelEntity (Actor::vftable also holds it): when the sub-object at this+0x1a0 is set it calls 010b41a0 with param_2 and the value at… |
| `007889a0` | ModelEntity::CopyMatrix16AndNotify | speculative | Copies 16 dwords (64 bytes) from the argument into this+0x160. If this+0xc is non-null, calls its vtable slot +0x18 with the argument. If this+4 is non-null, c… |
| `007889e0` | ModelEntity::CallThenNotifyMatrix | speculative | Calls 00764c50(argument) with this in ECX, then calls the vtable slot +0x18 of the object at this+0xc with &this[0x160] when that object is non-null, then call… |
| `010ac420` | ModelEntity::VirtualSlot2 | speculative | Virtual slot [2] of Deep::ModelEntity: when the sub-object at this+0x1a0 exists and the flag at +0x1b0 is clear, runs 010b43a0 on the matrix at this+0x160, the… |
| `010ac9e0` | ModelEntity::VirtualSlot3 | speculative | Virtual slot [3] of Deep::ModelEntity: when the error text at this+0x1ac is set it calls 010ac7f0 on param_2; otherwise, when the model at index this+0x1a4 exi… |
| `010ad090` | — | unknown | Constructor body of Deep::ModelEntity: stores the ModelEntity vtable, constructs the 8-inline Model pointer array at this+0x12c (010acd30), copies 16 dwords fr… |

**SoundListenerEntity** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b2fa0` | SoundListenerEntity::UpdateTransformsFromSource | likely | Copies the four dwords at param_2+0x30..+0x3c into this+0x20..+0x2c, then calls 007bd8f0 with the source block at param_2+0x30 and 007bd930 with the source blo… |

**(no class)** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010aca80` | DrawSpecialWithMaterial | confirmed | Draw path that takes a material (param_2) and a context (param_3): a null material sets this+0x1ac to 'Attempted DrawSpecial with NULL material'; with that err… |
| `007652e0` | Mat4ComposeEntryAt | likely | Thiscall (ECX = record, out pointer and an index argument): computes a 16-float output as the 4x4 product of the matrix at record+0x20 (rows at +0x20, +0x30, +… |
| `00777480` | WorldMatrixFromParentChain | likely | Thiscall (ECX = node, stack arg = out pointer to 16 floats). Copies the 4x4 at node+0x160..0x19c to out, then walks the parent chain through the pointer at +0x… |
| `00777610` | TransformSetTranslation | likely | Thiscall (ECX = node, stack arg = float4 vector pointer). Compares the stored vector at +0x190..+0x19c with the argument; when any lane differs it stores the f… |
| `00777bb0` | TransformRefreshDirty | likely | Thiscall (ECX = node). Sets the dirty byte at +0x22a to 1, then calls 007776b0 with ECX = node and the orientation pointer this+0x2f0 as its stack argument (th… |
| `00778880` | NodeMatrixOrChildMatrix | likely | Thiscall (ECX = node, stack args: out pointer to 16 floats, index). When the pointer at +0x370 and the pointer at +0x1a0 are non-null and 0 <= index < the coun… |
| `00778ff0` | LookupNamedRecordAndMatrix | likely | Thiscall (ECX = node, stack args: name C-string pointer, out record (8 dwords), out matrix (16 floats)). When the model pointer at +0x130 and the pointer at +0… |
| `008b31c0` | GetPairFromSlot1870 | likely | Reads the pointer at this+0x1870, substitutes the pair of globals at DAT_01be4850/DAT_01be4854 when it is null, and copies that pair into param_2[0] and param_… |
| `008b31e0` | DiffersFromDefaultPair | likely | Returns 0 when the dwords at this+0x17a0 and this+0x17a4 both equal the globals DAT_01be4850 and DAT_01be4854, and 1 otherwise. |
| `008b32e0` | GetPairFromSlot18b0 | likely | Copies the dwords at this+0x18b0 and this+0x18b4 into param_2[0] and param_2[1]; a leaf accessor that reads nothing else. |
| `008ba7a0` | OffsetResolvedVectorAndApply | likely | Resolves an object from two ids, reads its four-component vector through virtual +0x1c, adds a global value to the second component, and passes the result to F… |
| `00961b70` | GetSeatTransform | likely | Looks up the object selected by this[0x1cd] and this[0x141]; for a nonnegative seat index it computes the character yaw and composes a matrix from seat data, o… |
| `00961c40` | GetSeatTransformSecondary | likely | Looks up the selected object; for an in-range nonnegative index it reads the second seat transform at offsets +0x60/+0x6c and composes a matrix, otherwise copi… |
| `00962200` | SetModelFloatWhenChanged | likely | Gets a virtual object; when it exists and FUN_0076bcb0 differs from param_2, calls FUN_00779ea0 with param_2; a second guarded path can make the same call. |
| `00764960` | OwnedRecordPayloadPtr | speculative | Thiscall with ECX = owner (param_1) and param_2 = object: reads record = [param_2+8]; if record is null or record+4 is not the owner it returns the static addr… |
| `00764990` | OwnedRecordPayloadSet16 | speculative | Thiscall with ECX = owner (param_1), param_2 = object and param_3 = 16-dword source: if the record at [param_2+8] has its owner at +4 equal to param_1, it copi… |
| `00766040` | ListWalkComposeMatrix | speculative | Fastcall (ECX = object with a node list at +0x18, next +0x60): for each node whose attached object (+0x50) has a parent link [obj+4] that is null or passes the… |
| `007660b0` | SlotBindOwnerMatrix | speculative | Thiscall (ECX = owner object, param_2 = slot holder whose record pointer is at +8, param_3 = 16-float matrix, param_4 = 12-bit value, param_5 = flag byte): if … |
| `00766160` | SlotBindOwnerIndexMatrix | speculative | Thiscall (ECX = owner object, param_2 = slot holder whose record pointer is at +8, param_3 = 12-bit index, param_4 = 16-float matrix, param_5 = 12-bit value, p… |
| `00770410` | CopyVec8FromThis340OrDefault | speculative | Thiscall, one stack arg (out pointer to 8 dwords; ret 4): if the first entry of the array at this+0x130 is non-null, copies 8 dwords from this+0x340 to the out… |
| `00778820` | ChildListReleaseA4 | speculative | Fastcall (ECX = owner). When the list at owner+0x374 is non-empty, walks its links (+0xc) and for each entry: if the pointer at entry+0xa4 is non-null it calls… |
| `010ac490` | ModelEntity_PrepMatrixThenCall00766040 | speculative | When the sub-object at this+0x1a0 exists and the flag at +0x1b0 is clear it runs 010b43a0 on the matrix at this+0x160; then it calls 00766040(param_2) uncondit… |
| `010ac5b0` | ReplaceRefObjectAt84 | speculative | Replaces the reference-counted object at this+0x84: releases the old one (refcount at +4; at zero it calls the object's vtable slot [1]), then allocates 0x88 b… |
| `010ac7f0` | ResolveOwnerNodeAndNotify | speculative | Resolves an owner node: uses the owner at this+4 (its +0x38 field) when set, otherwise walks through 00729610 when this+8 is set, otherwise null; when that own… |
| `010ad300` | EhCleanup_00764e10 | speculative | Exception-cleanup funclet: calls 00764e10 on ECX (this) with the SEH frame set up and restored around the call. |
| `007658a0` | — | unknown | Thiscall (ECX = parent object, param_2 = 16-float output, param_3 = child object): reads the child's record at [child+8]; when that record's owner at [record+4… |
| `007776b0` | — | unknown | Thiscall (ECX = node, stack arg (flags word) and param_3 = float4 pointer). If the vector at +0x2f0..+0x2fc differs from the argument or the byte at +0x22a is … |
| `00777930` | — | unknown | Thiscall (ECX = node, int flags in EDX/register, param_3 = float4 translation pointer, param_4 = float4 orientation pointer). When the orientation at +0x2f0..+… |
| `009606e0` | — | unknown | Calls FUN_00776970 to derive four words from the supplied value and this+0x3d4; stores them at +0x610..+0x61c, initializes +0x620..+0x62c once, and optionally … |
| `00963040` | — | unknown | Changes bit 4 at byte +0x143 only when it differs from param_2; on clearing the bit, obtains the model, reads words +0x2f4..+0x2fc, computes yaw from fields 0x… |
| `00c5c4c0` | — | unknown | Fastcall, ECX = this (dword indices). If the low byte of [0x52a] is set, builds a 4x4 float matrix from FUN_00739fc0 and FUN_00770e70 applied to the object at … |

## Source shards

[[S0162|S0162]] (7), [[S0164|S0164]] (5), [[S0165|S0165]] (10), [[S0166|S0166]] (22), [[S0167|S0167]] (2), [[S0169 00787e70-0078b5f0|S0169]] (10), [[S0170|S0170]] (2), [[S0172 00798490-0079d890|S0172]] (27), [[S0173|S0173]] (2), [[S0175|S0175]] (25), [[S0179|S0179]] (14), [[S0222 008b1e00-008b7730|S0222]] (6), [[S0223 008b7790-008bcf40|S0223]] (2), [[S0256 009606e0-00965340|S0256]] (5), [[S0258 00969a10-0096de80|S0258]] (2), [[S0259 0096e490-00972d30|S0259]] (4), [[S0378 00becaa0-00bf1530|S0378]] (12), [[S0382 00bfcfe0-00c056a0|S0382]] (10), [[S0396|S0396]] (1), [[S0581|S0581]] (16), [[S0588|S0588]] (14)
