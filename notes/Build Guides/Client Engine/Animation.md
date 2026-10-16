---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Animation

Part of [[Build Guides]] › Client Engine. Related: [[Actors and NPCs]], [[Rendering]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

1026 accepted functions: 228 confirmed, 307 likely, 352 speculative, 139 unknown. Network-linked (`wire`): 0.

Subsystems: `anim.controller` (237), `lib.granny.curve` (121), `lib.granny.container` (67), `anim.definition` (62), `lib.granny.math` (58), `lib.granny.util` (55), `lib.granny.localpose` (40), `lib.granny.alloc` (38), `lib.granny.file` (31), `anim.cinematic` (25), `lib.granny.dag` (23), `lib.granny.bink` (20), `lib.granny.binding` (19), `lib.granny.mesh` (17), `lib.granny.data_type` (15), `anim.skeleton` (13), `lib.granny.data_type.io` (13), `lib.granny.sampler` (12), `anim` (11), `lib.granny.model` (11), `lib.granny.worldpose` (11), `lib.granny.data_type.fixup` (10), `lib.granny.variant` (10), `lib.granny.format` (10), `lib.granny.texture` (10), `lib.granny.controlled_animation` (8), `lib.granny.dag2.trackmask` (7), `lib.granny.iterator` (7), `lib.granny.dag2.subgraph` (6), `lib.granny.dispatch` (6), `lib.granny.mirror` (5), `lib.granny.dag2.sequence` (4), `lib.granny.dag.pose_cache` (4), `lib.granny.dag2.instance` (4), `lib.granny.lookat` (4), `lib.granny.dag2.timeshift` (3), `lib.granny.dag2.poseanimsource` (3), `lib.granny.dag2.locked_blend` (3), `lib.granny.dag2.callback` (3), `lib.granny.dag2.blend` (3), `lib.granny.skeleton` (2), `lib.granny.dag2.selection` (2), `lib.granny.dag2.scaleoffset` (2), `lib.granny.dag2.scalarsource` (2), `lib.granny.dag2.nway_blend` (2), `lib.granny.dag2.additive_blend` (2), `lib.granny.aggregate` (2), `lib.granny.compression` (2), `lib.granny.callback` (1)

### `anim`

**PlayerAnimationEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0096f6e0` | PlayerAnimationEvent::Dispatch | confirmed | Examines bits in param_3, conditionally fetches shared game context and updates animation-related fields through virtual calls, invokes FUN_0096c780 with store… |

**SoeUtil::Hash<AnimationSlot*,2048,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a3000` | HashFreeNode2048 | confirmed | Virtual slot [2] shared by Hash<AnimationSlot*,2048,-1> and HashMap<int,AnimationSlot*,2048,-1>: frees the node pointer argument through 00401c00 with alignmen… |
| `007a4910` | AnimSlotHashInit2048 | confirmed | Constructor (ECX=this, no stack arguments): stores SoeUtil::HashMap<int,AnimationSlot*,2048,-1>::vftable at this, zeroes this+0x8, memsets 0x2000 bytes at this… |
| `007a4990` | AnimSlotHashScalarDtor | confirmed | Scalar deleting destructor reached through Hash vtable slot [0]: stores the HashMap<int,AnimationSlot*,2048,-1> vtable at this, calls 007a1650 (drain) on this,… |
| `007a2fc0` | HashAllocNode2048 | likely | Virtual slot [1] shared by Hash<AnimationSlot*,2048,-1> and HashMap<int,AnimationSlot*,2048,-1>: calls FUN_006fb9c0, then FUN_006fb860; if that returns an obje… |

**SoeUtil::HashMap<int,AnimationSlot*,2048,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a1620` | AnimationSlotHashMapCtor | confirmed | Constructor (ECX=this, no stack arguments): stores SoeUtil::HashMap<int,AnimationSlot*,2048,-1>::vftable at this, zeroes this+0x4 and this+0x8, memsets 0x2000 … |
| `007a2f80` | AnimSlotHashMapDtorBody | confirmed | Destructor body (ECX=this, 11 bytes): stores SoeUtil::HashMap<int,AnimationSlot*,2048,-1>::vftable at this and tail-jumps to 007a1650 (2048-bucket drain). |
| `007a32f0` | AnimSlotHashMapScalarDtor | confirmed | Scalar deleting destructor (ECX=this, byte stack argument): stores the HashMap<int,AnimationSlot*,2048,-1> vtable at this, calls 007a1650 (2048-bucket drain) o… |
| `007a4500` | AnimSlotHashMapDtorBodyFramed | confirmed | Framed copy of the 2048-bucket HashMap destructor body: stores the HashMap<int,AnimationSlot*,2048,-1> vtable at this, calls 007a1650 (drain) on this and retur… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a4550` | BuildAnimationTypesXmlDoc | confirmed | __fastcall: ECX = object; its AnimationType list head is at this+0x4024 (next at record+0xc; record fields: +0x10 id, +0x18 name string, +0x28 boneMaskSuffix s… |
| `0095d330` | ClearAnimationMixerEntries | likely | Walks entries from param_2+4 through each node's +0x10 link, calls a virtual operation for each stored value, and logs it to AnimMixer.log. |

### `anim.cinematic`

**ControllerCinematic** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6f2e0` | ControllerCinematic::BeginTimedInterpolation | likely | Sets the state at +0xe8 to 1, copies the 4-float vector param_2 into +0xf0..+0xfc, calls 0101b230 and 0101b250(param_2), stores param_3 as the duration at +0x1… |
| `00a6f3c0` | ControllerCinematic::BeginStateOneWithRefs | likely | Sets the state at +0xe8 to 1, passes param_2 and param_3 to 0101c990 (reference handoff for two objects), and stores param_4 at +0x110. |
| `00a6f520` | ControllerCinematic::ScalarDeletingDtor | likely | Scalar deleting destructor in the secondary vtable at 01815b9c, slot 5: calls 00a6f1c0 (the cinematic destructor body), then frees this through _free when bit … |
| `00a6fa90` | ControllerCinematic::StartPlaybackState2 | likely | Checks the handle (manager slot 0x10 with DAT_01be701c) and that 007866b0 accepts param_3. It then stores param_3 at +0x120 and param_4 at +0x11c, clears the t… |
| `00a6fc20` | ControllerCinematic::StartPlaybackState3 | likely | Same checks as 00a6fa90. It stores param_3 at +0x120 and param_5 at +0x11c, clears the top two bits of the flag at +0xec, passes param_4 to 0101c990, calls 007… |
| `00a6fdb0` | ControllerCinematic::SetFlag40IfState23 | likely | When flag 0x40 of +0xec is clear and the state at +0xe8 is 2 or 3, gets the handle through the manager and, when valid, calls 00784c80 on +0x120 and calls 007b… |
| `00a6fe20` | ControllerCinematic::ClearFlag40IfState23 | likely | When flag 0x40 of +0xec is set and the state at +0xe8 is 2 or 3, gets the handle through the manager and, when valid, calls 00784c80 on +0x120 and 007bac90 wit… |
| `00a70790` | ControllerCinematic::SetCinematicModeFlag | likely | If param_2 differs from bit 0x20 of the flag byte at +0xec: when param_2 is nonzero it fires GameEvents:OnCinematicModeOn through 009797a0 and sets bit 0x20; w… |
| `00a6f270` | ControllerCinematic::BindManagerObjectAndNotify | speculative | Virtual in the secondary vtable at 01815b9c, slot 7. Calls 0094e210, 007935a0 (with _DAT_01be7018) and 0094f3c0; clears a byte in the sub-object at (this+0x74)… |
| `00a6f400` | ControllerCinematic::ForwardField2cToSlot1 | speculative | Virtual in the secondary vtable at 01815b9c, slot 48 (+0xc0), 31 bytes: when the dword at +0x2c is nonzero and the object at +0x11c is non-null, it replaces th… |
| `00a6fa10` | ControllerCinematic::ReleaseDetachedActor | speculative | Virtual in the secondary vtable at 01815b9c, slot 32 (+0x80), 115 bytes: when the state at +0xe8 is 2 or 3, bit 0x80 of +0xec is set and the object at +0x11c i… |
| `00a6f1c0` | — | unknown | Destructor body: stores the ControllerCinematic vtable at this, releases two members through 0101b370 (the first call carries the frame cookie), then calls the… |
| `00a6f960` | — | unknown | Virtual in the secondary vtable at 01815b9c, slot 8. Releases the object at +0xe4 through 0094cf60 and 00958580 when present, and clears it. When flag bit 0x20… |
| `00a6fe90` | — | unknown | Virtual in the secondary vtable at 01815b9c, slot 49 (+0xc4), 2052 bytes. Per-frame update dispatched on the state at +0xe8. State 1 advances the elapsed time … |
| `00a706a0` | — | unknown | Constructor: calls the base constructor 0094f6c0, stores param_5 at +0xe0, installs the ControllerCinematic vtable at offset 0, zeroes +0xe4, +0xe8 and the byt… |

**_anon_34A1D9ED::ControllerCinematicActorHandler** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6f850` | _anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseNam… | confirmed | Releases the IString<char> at +4 of the handler: stores the SoeUtil::IString<char> vtable there and, when the length at +0x10 is positive, drops a reference on… |
| `00a6f8d0` | _anon_34A1D9ED::ControllerCinematicActorHandler::Controller… | confirmed | Constructor: installs the ActorHandler vtable and then the ControllerCinematicActorHandler vtable at offset 0, embeds an IString<char> at +4 with empty buffer … |
| `00a6f940` | _anon_34A1D9ED::ControllerCinematicActorHandler::ScalarDele… | confirmed | Scalar deleting destructor: calls 00a6f850 (text member release), then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6f540` | _anon_34A1D9ED::ControllerCinematicActorHandler::ApplyTimeA… | speculative | Virtual slot 1 of the cinematic actor handler. Clamps the float returned by 007707f0 to at least the constant DAT_01815b84 and stores it at +0x98 of the object… |
| `00a6f770` | _anon_34A1D9ED::ControllerCinematicActorHandler::ReleaseGlo… | speculative | Virtual slot 2 (+8) of the cinematic actor handler: releases the global handle DAT_01be701c through the manager's +0xf0 object slot 2 (+8), then sets the globa… |
| `00a6f7b0` | _anon_34A1D9ED::ControllerCinematicActorHandler::ClearGloba… | speculative | Virtual slot 13 (+0x34), 24 bytes: when the global DAT_01be701c equals param_1 it sets the global to 0; otherwise does nothing. |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d009c0` | GetTimelineEntryField4ByIndex | confirmed | Reads the requested index, checks it against the array count, resolves the indexed entry when in range, and returns that entry's dword at offset four or zero. |
| `00a6f740` | ReleaseGlobalCinematicHandle | likely | If the global DAT_01be701c is non-zero, gets the manager object through 00769710 and calls virtual slot 2 (+8) of its +0xf0 object with the handle. Does not cl… |
| `00a6f7d0` | CinematicHandle_QueryActorDeltaPositive | speculative | Asks the manager (00769710, then its +0xf0 object slot 4 with the global handle DAT_01be701c) whether the handle is valid. If so, calls 00784c80 on the field a… |
| `00a707f0` | CinematicBoneModel_LookupAndBind | speculative | Hashes the literal 'Cinematic.BoneName' with a byte-wise one-at-a-time style mix (final *9 ^ >>11 and *0x8001 step) and looks it up in the 2048-bucket table at… |

### `anim.controller`

**Actor** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007803f0` | Actor::SetSkeleton | confirmed | Thiscall with one argument (skeleton definition p2), returns 1 or 0. Releases the reference held at this+0x370 (count at +4, vtable slot [1] when it reaches ze… |
| `00789010` | Actor::PlayAnimRecord | confirmed | Returns early unless this+0x2c0 is zero, this+0x374 is nonzero and byte +0x228 is zero. Resolves the animation slot and transition rule (0076ff00, 00769710, 00… |
| `00785280` | Actor::RegisterAnimSlot | likely | Thiscall (ECX = O) with stack arguments (a pointer to 12 dwords, id, a field, and IString arguments). Does nothing unless 00784c80(id, 0) finds a node. When it… |
| `007866d0` | Actor::ProcessPendingAnimationLoads | likely | Fastcall (ECX = Actor). Walks the pending-load list whose head is at +0x3f8 (next at node+0x158). For each node it looks the node name (+0x28) up in the table … |
| `00786e40` | Actor::AddAnimationSlot | likely | Thiscall (ECX = Actor; stack argument = record R). Reads the animation id dword at R+0x48+0x10 and looks it up in the Actor table at +0x3d8/+0x3e8 (bucket by i… |
| `0078b2e0` | Actor::PlayAnimRecordAndChildren | likely | Calls 00789010(record, 0). When the result is positive it stores it at this+0x37c and the record id at +0x380, then for each child in the +0x608 list whose +0x… |
| `0078b520` | Actor::LookupAndPlayRecord | likely | Calls 007742d0(arg1). If the result is non-null and its +0x10 value is not -1, builds a 0x30-byte record (key from +0x10, byte flag from arg2, two fields -1, t… |
| `00787860` | Actor::IsAnimActive | speculative | Thiscall (ECX = actor A; param_2 = animation id; param_3 id filter; param_4 state value). Returns a byte: walks A's active list (A+0x374, next at +0xc). For ea… |
| `00787490` | — | unknown | Thiscall (ECX = actor A; param_2 = track owner T; param_3 int; param_4 log flag; param_5 char loop flag; param_6 float ease time, negative means use the entry'… |

**ActorAnimationControl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007704b0` | ActorAnimationControl::ActorAnimationControl | confirmed | Thiscall, two stack args (ret 8 expected): calls 007ecae0 with its two arguments, stores ActorAnimationControl::vftable at this+0, computes the case-folded has… |
| `007705d0` | ActorAnimationControl::~ActorAnimationControl | confirmed | Thiscall with one byte argument (scalar deleting destructor shape): sets up an SEH frame, calls the base destructor body 007ecb50 with ECX = this, and when bit… |
| `00778de0` | ActorAnimationControl::Slot1ApplyUniformScale | speculative | Thiscall virtual slot [1] of ActorAnimationControl (ECX = object, two stack args). Calls 007f0010 with the two stack arguments. When the index at +0x18 is non-… |

**ActorLookControl** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c7350` | ActorLookControl::ScalarDeletingDtor | confirmed | Virtual slot [0] (90 bytes): calls 010aff30 on the object (base destructor, receiver from the caller's ECX), then calls _free on it when the deleting flag bit … |
| `007c7050` | ActorLookControl::Ctor | likely | Thiscall with four stack arguments (113 bytes). Calls 010afda0 with the four arguments (base constructor), then stores the ActorLookControl vtable at +0. |
| `007c70e0` | ActorLookControl::SetFlag90 | speculative | Virtual slot [3] (10 bytes): stores byte 1 at this+0x90 and returns. Slot [3] is shared by ActorLookControl, ActorLookControlTurret and ActorLookControlHumanoi… |
| `007c7470` | ActorLookControl::DotAboveThreshold | speculative | Virtual slot [4] (78 bytes); the signature shows two float-array pointer arguments and no this. Computes the four-term dot product of the two 4-float vectors a… |

**ActorLookControlHumanoid** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c7410` | ActorLookControlHumanoid::ScalarDeletingDtor | confirmed | Virtual slot [0] (90 bytes): same body as 007c7350: calls 010aff30, then frees the object when flag bit 0 is set, and returns the pointer. |
| `007c7240` | ActorLookControlHumanoid::Ctor | likely | Thiscall with four stack arguments (113 bytes). Calls the ActorLookControl constructor 007c7050 with the four arguments, then stores the ActorLookControlHumano… |

**ActorLookControlTurret** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c73b0` | ActorLookControlTurret::ScalarDeletingDtor | confirmed | Virtual slot [0] (90 bytes): same body as 007c7350: calls 010aff30, then frees the object when flag bit 0 is set, and returns the pointer. |
| `007c7150` | ActorLookControlTurret::Ctor | likely | Thiscall with four stack arguments (113 bytes). Calls the ActorLookControl constructor 007c7050 with the four arguments, then stores the ActorLookControlTurret… |

**AnimVisibilityController** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e72e0` | AnimVisibilityController::DtorBody | confirmed | Fastcall (ECX = object). Stores AnimVisibilityController::vftable at the object and at +8, clears a local flag, calls 007e7260 on the +8 subobject when the own… |
| `007e7360` | AnimVisibilityController::ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the destructor body 007e72e0, then if bit 0 of the flag is set calls _free on this, and returns this. |
| `007e73d0` | AnimVisibilityController::AnimVisibilityController | confirmed | Thiscall (ECX = object) with two stack args (owner pointer, byte flag). Calls 007e7110 to construct the VisibilityController base, stores AnimData::CallbackInt… |
| `007e6f10` | AnimVisibilityController::ClearField4 | speculative | Virtual method with no args (thiscall): stores 0 to the dword at this+4 and returns. It has no free and no destructor call, so it is an ordinary virtual, not a… |
| `007e7450` | AnimVisibilityController::GetFloatD0 | speculative | Fastcall virtual (ECX = object), no args, returns a float. If the flag byte at +0x10 is nonzero it returns the constant at 0175b420. Otherwise it takes the own… |
| `007e74d0` | AnimVisibilityController::LookupEntryRatio | speculative | Thiscall virtual with one stack arg (key). When the owner at +0xc is non-null it hashes the key with 00705b00 (case-folded), searches the bucket chain in the o… |

**AnimationGroup** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a95d0` | AnimationGroupDtorBody | confirmed | Destructor body: stores AnimationGroup::vftable at this, truncates the slot array at this+0x1c through 007a5ab0, releases the IString<char> group name at this+… |
| `007a97b0` | AnimationGroupScalarDtor | confirmed | Scalar deleting destructor (vtable slot [0] of AnimationGroup): runs the destructor body 007a95d0 on this, then calls _free(this) when bit 0 of the stack flag … |
| `007a9390` | — | unknown | Constructor (ECX = this, three stack arguments): stores AnimationGroup::vftable at this, zeroes the header dwords, installs the slot-array vtable at +0x14 with… |

**AnimationRelationshipManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a6ca0` | — | unknown | Copies the path at this+0x49f4, appends AnimationTransitionRules.xml, reads AnimationConstants.wieldTypeTransitionTimeSec into the float at this+0x4e4c, then f… |
| `007a7590` | — | unknown | Looks up the 2048-bucket records for id param_2 and id param_3 (the second through the same table, both by id), copies each record's name at +0x20 through 0040… |

**AnimationTrigger** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00786350` | AnimationTriggerCtor | confirmed | Stores AnimationTrigger::vftable at offset 0, calls 00785380 with ECX = this+4 (constructs the embedded SoeUtil::List<AnimationTriggerEvent,-2> at +4), then ca… |
| `007863b0` | AnimationTriggerDtor | confirmed | Stores AnimationTrigger::vftable at [ecx], then with ECX = this+4 calls 00785e50 (ListDtor of the embedded -2 AnimationTriggerEvent list). It does not free thi… |
| `00786400` | AnimationTriggerDeletingDtor | confirmed | Virtual slot 0 of AnimationTrigger::vftable. Stores that vtable, calls 00785e50 with ECX = this+4 (embedded list teardown), then calls _free(this) when bit 0 o… |
| `007864c0` | AnimTrackInit | likely | Thiscall (ECX = new 100-byte track object T; stack argument = owner O). Calls 00786350 (AnimationTriggerCtor) on T, zeroes T+0x20, +0x24, +0x28, +0x30, +0x34, … |

**Controller** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0094f6c0` | Controller::Controller | confirmed | Thiscall ctor (ECX = this, three stack args forwarded from the derived ctors as param_2..4; ret 0xc). Stores Controller::vftable at [this], installs the embedd… |
| `0094f980` | Controller::SetByte88 | confirmed | Thiscall virtual (Controller::vftable slot 36, also ControllerRoot slot 36): stores the byte argument at this+0x88. |
| `0094f990` | Controller::GetByte88 | confirmed | Thiscall virtual (Controller::vftable slot 37, also ControllerRoot slot 37): returns the byte at this+0x88. |
| `0094f9d0` | Controller::GetByteD4 | confirmed | Virtual (Controller::vftable slot 43): returns the byte at this+0xd4. |
| `0094f9f0` | Controller::SetByteD4 | confirmed | Thiscall virtual (Controller::vftable slot 45): stores the byte argument at this+0xd4. |
| `0094fa50` | Controller::ClearLinkedStateEntries | confirmed | Virtual (Controller::vftable slot 9, also ControllerRoot slot 9): walks the linked list whose head is at this+0xc8 (next link at +4 of each node); for each nod… |
| `0094fa80` | Controller::ClearLinkedStateEntries2 | confirmed | Virtual (Controller::vftable slot 10): byte-identical body to 0094fa50 (same list walk at this+0xc8 and the same zeroing of +0x20, +0x24, +0x2c). A second vtab… |
| `0094fab0` | Controller::DtorBody | confirmed | Dtor body (ECX = this, EH state set): stores Controller::vftable at [this], sets fields +0x80/+0x84 to the global defaults, calls the object at +0x78 through i… |
| `009500f0` | Controller::SetPairAndDistributeToEntries | confirmed | Thiscall virtual (Controller::vftable slot 22; two stack args plus ret): stores arg2 (param_3) at this+0xb0 after calling 0094f3c0 when it changes. When the li… |
| `00950500` | Controller::SetProjectionFromTangentAngle | confirmed | Virtual (Controller::vftable slot 41, one float arg): computes fptan of the argument times DAT_017ed84c and passes the result with 1.0 (0x3f800000) to 0094f480. |
| `00950d10` | Controller::ScalarDeletingDtor | confirmed | Thiscall virtual (Controller::vftable slot 7, one flag byte arg, ret 4): calls 0094fab0 (Controller::DtorBody) on this, frees this with _free (00d82463) when b… |
| `00951000` | Controller::FormatMouseState | confirmed | Fastcall virtual (Controller::vftable slot 42): on first use initialises the static IString at 0x01be4fa8 and registers its exit cleanup, then formats the four… |
| `0094c850` | Controller::MulFloat87cByFloatA8 | speculative | Virtual (vtable slot 16): returns ST0 = float [[this+0x74]+0x87c] * float [this+0xa8] (fld dword [eax+0x87c]; fmul dword [ecx+0xa8]; ret). |
| `0094c860` | Controller::StoreFloat87c | speculative | Virtual (vtable slot 17): stores its float stack argument into the dword at [[this+0x74]+0x87c] (movss; ret 4). |
| `0094d160` | Controller::SetTargetDeltaAndQueue | speculative | Virtual (vtable slot 20): clears byte this+0xad; looks up the node for the key pair at this+0x80/0x84 (00984960); when found, subtracts the 4-float vector from… |
| `0094dfe0` | Controller::SetKeyFromSubobject | speculative | Virtual (vtable slot 8, thiscall, one argument): when the argument is null, sets the key pair this+0x80/0x84 to DAT_01be4ff8/DAT_01be4ffc; otherwise calls vtab… |
| `009502d0` | — | unknown | Thiscall virtual (Controller::vftable slot 21, five stack args): looks up the key-pair node at this+0x80/0x84; computes the four-component distance between two… |
| `00951700` | — | unknown | Thiscall virtual (Controller::vftable slot 3, one dword arg): does nothing unless the global count DAT_01b4215c is positive; otherwise it runs a settings looku… |
| `00951860` | — | unknown | Thiscall virtual (Controller::vftable slot 2, one arg): looks up the setting object named HUD:InterceptMouseWheel through 009797a0; for the found object it cal… |

**ControllerRoot** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00950530` | ControllerRoot::ControllerRoot | confirmed | Thiscall ctor (ECX = this; ret 0x14; its first three stack args are forwarded to the Controller base as param_2..4). Calls 0094f6c0, installs ControllerRoot::v… |
| `00950770` | ControllerRoot::SetByteF5 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 55; the same body is shared by the ControllerRoot-derived classes): stores the byte argument at this+0xf5. |
| `00950780` | ControllerRoot::SetByteF6 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 56): stores the byte argument at this+0xf6. |
| `00950790` | ControllerRoot::SetByteF7 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 57): stores the byte argument at this+0xf7. |
| `009507a0` | ControllerRoot::SetWord164 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 23): stores the dword argument at this+0x164. |
| `009507c0` | ControllerRoot::SetWord168 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 26): stores the dword argument at this+0x168. |
| `009507e0` | ControllerRoot::SetWord16C | confirmed | Thiscall virtual (ControllerRoot::vftable slot 28): stores the dword argument at this+0x16c. |
| `00950800` | ControllerRoot::SetFourWordsAt110 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 58), one pointer arg (ret 4): copies four dwords from the argument array to this+0x110, +0x114, +0x118 and +0x11… |
| `00950830` | ControllerRoot::SetFourWordsAt100 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 59), one pointer arg (ret 4): copies four dwords from the argument array to this+0x100 through +0x10c. |
| `00950860` | ControllerRoot::SetWord180 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 60): stores the dword argument at this+0x180. |
| `00950880` | ControllerRoot::SetByteF9 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 61): stores the byte argument at this+0xf9. |
| `00950890` | ControllerRoot::GetByteF5 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 38): returns the byte at this+0xf5. |
| `009508a0` | ControllerRoot::GetByteF6 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 39): returns the byte at this+0xf6. |
| `009508b0` | ControllerRoot::GetByteF7 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 40): returns the byte at this+0xf7. |
| `009508c0` | ControllerRoot::SetWord158 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 62): stores the dword argument at this+0x158. |
| `009508e0` | ControllerRoot::SetByteF4 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 63): stores the byte argument at this+0xf4. |
| `009508f0` | ControllerRoot::GetByteF4 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 43): returns the byte at this+0xf4. |
| `00950900` | ControllerRoot::GetByte190 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 65): returns the byte at this+0x190 (decimal 400). |
| `00950910` | ControllerRoot::CopyFourWordsAt120 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 32), one pointer arg: copies the dwords at this+0x120, +0x124, +0x128 and +0x12c (decimal 300) into the argument… |
| `00950940` | ControllerRoot::CopyFourWordsAt110 | confirmed | Thiscall virtual (ControllerRoot::vftable slot 33), one pointer arg: copies the four dwords at this+0x110 through +0x11c into the argument array and returns 1. |
| `00950970` | ControllerRoot::GetByte154 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 4): returns the byte at this+0x154. |
| `00950980` | ControllerRoot::GetWord74 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 70): returns the dword at this+0x74. |
| `00950990` | ControllerRoot::GetFloat164 | confirmed | Fastcall virtual (ControllerRoot::vftable slot 24): returns the float at this+0x164 as the x87 return value. |
| `009509a0` | ControllerRoot::DtorBody | confirmed | Dtor body (ECX = this, EH state set): stores ControllerRoot::vftable at [this]. When byte +0xf4 is clear, it looks up key-pair nodes (00984960, 0095ad40, 0101b… |
| `009510b0` | ControllerRoot::ScalarDeletingDtor | confirmed | Thiscall virtual (ControllerRoot::vftable slot 7, one flag byte arg, ret 4): calls 009509a0 on this, frees this with _free (00d82463) when bit 0 of the flag is… |
| `0094caa0` | ControllerRoot::SetByte190 | speculative | Virtual (vtable slot 64, thiscall, one byte argument): when the byte at this+0x190 differs from the argument it stores the argument there, calls 00769710, 008a… |
| `0094e400` | ControllerRoot::SetKeyFromSubobjectGuarded | speculative | Virtual (vtable slot 8, thiscall, one argument): looks up the node for the key at this+0xe8/0xec through 00769710 and 00984960; when the node exists and this+0… |
| `0094e4d0` | ControllerRoot::StepAxisFloats | speculative | Virtual (vtable slot 66, fastcall, per-frame): when bit 0x20 of this+0x2c is set, integrates this[0x5b] and this[0x5a] by this[0x12] and this[0x13] scaled by c… |
| `0094ebd0` | ControllerRoot::ClampMoveToLimit | speculative | Virtual (vtable slot 68, thiscall, two float pointer arguments): when byte this+0xf8 is set and both arguments are non-null, looks up the node for this+0x80/0x… |
| `0094ed90` | ControllerRoot::SetAxis59FromScaled | speculative | Virtual (vtable slot 25, thiscall, one float argument): when vtable slot 38 (+0x98) returns zero, sets this[0x59] to vtable slot 24 (+0x60) result minus the ar… |
| `0094c9d0` | — | unknown | Virtual (vtable slot 51, thiscall, one dword argument): calls this object's own vtable slot 67 (+0x10c) with the sub-object pointer this+0x48, then its slot 68… |
| `0094d3b0` | — | unknown | Virtual (vtable slot 53): calls 0101b8d0 and 0101c990 and returns early when the pair from 0101b8d0 equals the default pair DAT_01be4ff8/DAT_01be4ffc. Otherwis… |
| `0094e5e0` | — | unknown | Virtual (vtable slot 67, thiscall, 1508 bytes): snapshots the sub-object at this+0x38 (008efe00), calls vtable slot 66 (+0x108) of this, builds a normalised di… |

**ControllerSingleRotation** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a6ee70` | ControllerSingleRotation::~ControllerSingleRotation | confirmed | Destructor body: stores the ControllerSingleRotation vtable at this, then calls the base destructor 0094fab0. It does not free this. |
| `00a6ef10` | ControllerSingleRotation::ScalarDeletingDtor | confirmed | Scalar deleting destructor body: stores the ControllerSingleRotation vtable, calls the base destructor 0094fab0, then frees this through _free when bit 0 of th… |
| `00a6eda0` | ControllerSingleRotation::ControllerSingleRotation | likely | Constructor: calls the base constructor 0094f6c0, zeroes the float at +0xe0 (rotation angle), stores param_6 at +0xe4 (radius) and param_7 at +0xe8 (height off… |
| `00a6eec0` | ControllerSingleRotation::ConsumeFullTurn | likely | Virtual slot 35 (+0x8c), 34 bytes: when the float at +0xe0 (angle) is greater than the constant DAT_017eb268 (bits 0x40c90fdb, which is 2*pi as a float) it sto… |
| `00a6eef0` | ControllerSingleRotation::ForwardFloatToSharedSlot70 | speculative | Virtual slot 28 (+0x70), 29 bytes: gets an object from 0094c6f0 and, when it is non-null, calls that object's virtual slot 28 (+0x70) with the float argument. … |
| `00a6ef70` | — | unknown | Virtual slot 51 (+0xcc). Gets the centre object from 0094cf60 and reads its position through the virtual at +0x1c. Advances the angle at +0xe0 by (2*pi / perio… |

**Deep::KneeControl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ad6b0` | KneeControl::KneeControl | confirmed | Constructor for the knee control: runs the SkeletonControl base constructor body (vtable, owner at +0xc, list insertion through 010b3f20), stores the KneeContr… |
| `010ade30` | KneeControl::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape for Deep::KneeControl (vtable slot [0]): calls the SkeletonControl destructor body 010ad850 on ECX, frees the object with _fre… |

**Deep::LookControl** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010afda0` | LookControl::LookControl | confirmed | Base constructor of Deep::LookControl: stores the Deep::SkeletonControl vtable, the owner at +0xc and a flag at +0x10, calls 010b3f20 on the object, stores the… |
| `010aff30` | LookControl::~LookControlBody | confirmed | Destructor body of Deep::LookControl: stores the Deep::LookControl vtable, destroys the joint map through 010afc60, calls the Deep::Entity destructor 00764e10,… |
| `010affc0` | LookControl::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of Deep::LookControl: calls 010aff30 on this and calls _free on this when bit 0 of the argument is set; returns th… |
| `010adf40` | LookControl::Slot1Update | likely | Virtual slot [1] of Deep::LookControl. Returns early unless the byte at this+0x90 is set; the solve branch runs when the byte at this+0x10 is clear or the floa… |

**Deep::LookControl::Target** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010adbd0` | LookControl::Target::Target | confirmed | Constructor body: calls 007648d0 (the Deep::Entity base constructor) and then stores the LookControl::Target vtable. |

**Deep::SkeletonControl** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ad5f0` | SkeletonControl::SkeletonControl | confirmed | Constructor body: stores the SkeletonControl vtable, zeroes +4 and +8, stores param_2 at +0xc (owner), clears the byte at +0x10, and links the control into the… |
| `010ad620` | SkeletonControl::~SkeletonControl | confirmed | Destructor body (no EH frame): stores the SkeletonControl vtable and unlinks the control through 010b3fa0 when the owner pointer at +0xc is set. |
| `010ad850` | SkeletonControl::~SkeletonControlEh | confirmed | Destructor body of SkeletonControl with an EH frame (same work as the no-frame copy 010ad620): stores the SkeletonControl vtable and unlinks through 010b3fa0 w… |
| `010adba0` | SkeletonControl::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape for Deep::SkeletonControl (vtable slot [0]): stores the vtable, unlinks through 010b3fa0 when the owner at +0xc is set, frees … |

**ListAnimationReferenceDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078d790` | ListAnimationReferenceDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor (ret 4): calls the destructor body 0078a980 with this in ECX, then calls _free(this) only when the stack flag argument has bit 0 set… |

**SoeUtil::Array<AnimationGroup::AnimationGroupSlot,0,1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a2ef0` | AnimGroupSlotArrayFreeBuffer | confirmed | Virtual slot [2]: calls 00401c00 with the buffer argument and alignment 4, then returns. No stack cleanup beyond ret. |
| `007a2f10` | AnimGroupSlotArrayExchangeFields | confirmed | Virtual slot [3] (ECX=this, arg1 = other array pointer, arg2 = flag byte; ret 8 not verified). Flag non-zero: calls vtable slot [3] of the other object. Flag z… |
| `007a2e50` | — | unknown | Virtual slot [1] (ECX=array, args: requested count, pointer receiving the new count, flag byte; ret 0x10 not verified). Flag 0: grows to count*5/4 when the cur… |

**VisibilityController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e6fa0` | VisibilityController::DtorBody | confirmed | Thiscall (ECX = object). Stores VisibilityController::vftable and then SoeUtil::RefCounted::vftable into the object's first dword. Releases no members. Called … |
| `007e6ff0` | VisibilityController::ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Stores VisibilityController::vftable and then SoeUtil::RefCounted::vftable into the object (the destructor body is inlined here;… |
| `007e7110` | VisibilityController::VisibilityController | confirmed | Constructor (ECX = object, no args). Sets the cookie frame, calls 0071b950 (RefCountedImplicit construction, installs the RefCounted vtable and initialises the… |

**(no class)** (126)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007806a0` | AnimTrackListingLines | confirmed | Thiscall (ECX=this, second argument unused, third is a filter pointer). Walks the track list with 007a27d0 and 007a27e0 (advance) and, for each track keyed by … |
| `00788ad0` | AnimTrackLogText | confirmed | Formats one diagnostic line per animation track: walks the track list at this+0x374 (links at +0xc), finds the matching hash entry through the table at this+0x… |
| `0078a980` | ListAnimationReferenceDefinition::DerivedDtorBody | confirmed | Tears down an object whose primary base is SoeUtil::List<AnimationReferenceDefinition,-1>: releases an IString at +0x4c (buffer released when its count at +0x5… |
| `007aad60` | LoadCodeAnimationMappings | confirmed | Copies the path prefix at this+0x5ac into a local IString (006f9e60), appends the file name 'CodeAnimationMappings.txt' (402770), reads the file with 00fbf4a0 … |
| `007ab650` | LoadWieldTypeAnimationMappings | confirmed | Copies the path prefix at this+0x34 into a local IString (006f9e60), appends 'WieldTypeAnimationMappings.txt' (00402770), reads the file with 00fbf4a0 and, whe… |
| `00965890` | ToggleAllAnimationsOnPlayer | confirmed | Thiscall with a player argument. Calls the virtual at this vtable +0x24 as a guard; when it returns non-zero it releases the string at this+0xf1c and resets th… |
| `009662c0` | ReportAnimationPlaybackState | confirmed | Fastcall (ECX = this). Calls the vtable +0x24 guard on this and returns when it is zero. Otherwise it calls 0078d520, builds a temporary SoeUtil::IString<char>… |
| `00782fe0` | NormalizeIntoLookEntries_00782fe0 | likely | Thiscall (ECX=this; p2 = pointer to four floats, p3 = optional name pointer). When this+0x3b4 is zero, this+0x390 is set and the owner (this+0x2b0) is null or … |
| `00783150` | UpdateLookControls_00783150 | likely | Fastcall (ECX=this). Same gate as 00782fe0 (calls 00780a30 first when this+0x3b4 is zero, this+0x390 is set and the owner's +0x138 byte is clear). Walks the lo… |
| `00786620` | AnimTrackCreate | likely | Thiscall (ECX = owner O). Allocates 100 bytes with 00d833a6. When the allocation succeeds it calls 007864c0 with ECX = the new object and the owner as the stac… |
| `007866b0` | HasNodeForId | likely | Stack argument = id; ECX = owner O. Calls 00784c80(id, flag 0) with ECX = O and returns 1 when the result is non-null, else 0. |
| `00786c30` | EnsureAnimSlotRegistered | likely | Stack argument = id; ECX = owner O. Looks the id up with 00784c80(id, 0); when a node N is found and it lacks a non-null +0x14 object with a non-null +0x40 fie… |
| `00786cd0` | HasReadyAnimNode | likely | Stack argument = id; ECX = owner O. Returns 1 when 00784c80(id, 0) finds a node N whose +0x14 object is non-null, whose +0x14 object has a non-null +0x40 field… |
| `00787090` | TouchAnimTrackTime | likely | Fastcall (ECX = owner O). Reads the millisecond tick from 007027f0 into a 64-bit value, creates the track through 00786620 when O+0x14 is zero, then stores the… |
| `00787c50` | SetTrackDword50ById | likely | Thiscall (ECX = owner O; param_2 = id; param_3 dword). Finds the table node with key id; when it exists, its track (node+0x14) is non-null with a non-null +0x4… |
| `00787ca0` | AnimDataDuration | likely | Thiscall (ECX = owner O; param_2 = id; param_3 flag: divide by scaled rate; param_4 flag: log). Finds the node for id. When found and its AnimData duration (+4… |
| `0078a460` | ApplyAnimRecord | likely | Copies the 12-dword record at param_2 into a local and calls 0072fe10 with the record's name, the model name at this+0x2b0 +0x494, or DAT_0175b400. Gets a coun… |
| `0078b3b0` | ForEachChildApplyAnimRecord | likely | Copies the 12-dword record at param_2 into a local, calls 0072fe10 for a name (model name or DAT_0175b400), and checks the 0077c5e0 predicate. It gets a count … |
| `0078d3d0` | SetCurrentAnimAndPlay | likely | Releases the refcounted name buffer at this+0x2c8 (release through vtable +8 when count is positive), points +0x2c8 at the empty string DAT_01bdfda9, zeroes +0… |
| `007aaa50` | HashMapInt360WithIStringDtorBody | likely | Destructor body: sets the IString<char> vtable at this+0x5ac; if the capacity at +0x5b8 is above 0 it decrements the refcount at the buffer (+0x5b0, minus 4) a… |
| `007aabc0` | ParseCaretIntPairLines | likely | Parses the text of the IString at param_1 (buffer +4, length +8): skips the first line with 006fbd00, then for each line reads it with 006fc270, splits it on c… |
| `007aae80` | CodeAnimationMappingsCtor | likely | Constructor body: calls 007aa960, initialises the IString<char> at this+0x5ac to the empty static string (length 0, capacity 0), assigns the C string 'Resource… |
| `007aaf00` | CodeAnimationMappingsSetPrefix | likely | Thiscall with one char* argument: compares the prefix buffer at this+0x5b0 with the argument character by character; when they differ it assigns the argument t… |
| `007ab4d0` | ParseIntStrIntIntLinesToMap10 | likely | Clears the map at ECX (007aaf80), then parses the text in the IString at param_1 (buffer +4, length +8) line by line: skips lines whose first character is '#',… |
| `007ab770` | WieldTypeAnimationMappingsCtor | likely | Constructor body: calls 007ab280 (10-bucket map constructor) on this, initialises the IString<char> at this+0x34 to the empty static string (length 0, capacity… |
| `007ab7f0` | WieldTypeAnimationMappingsSetPrefix | likely | Thiscall with one char* argument: compares the prefix buffer at this+0x38 with the argument character by character; when they differ it assigns the argument to… |
| `007ba9b0` | AnimationSlot_GetField10 | likely | Getter: returns [ [this+4] + 0x48 ]->+0x10, or 0 when that pointer is null (17 bytes: `8b4104 8b4848 85c9 7403 8b4110 c3`). |
| `007baba0` | Timeline_ReleaseTarget | likely | Releases the refcounted object referenced through this+0x14: takes the object at [this+0x14]+0x44, decrements its refcount at +4 under LOCK and, when it reache… |
| `007bad10` | AnimationSlot_GetField20 | likely | Getter: returns [ [this+4] + 0x48 ]->+0x20, or 0 when that pointer is null (17 bytes). |
| `007badc0` | Timeline_ResetAndDeactivateNodes | likely | Resets the timeline cursor at +0x18 to the DAT_01762a34 constant and walks the node list at +8 (next at node+0x1c): for every node whose [node+0] pointer is no… |
| `007bb6e0` | Timeline_Reset | likely | If this+0x14 is non-null: calls 007badc0 (rewind+deactivate all nodes), then when [[this+0x14]+0x44] exists and its +0x94 float is negative calls 00779d20(0), … |
| `007bb7f0` | AnimationDefinition_BuildTrackEvents | likely | Walks three source lists (heads at [ [this+4] +4 ], +0x14 and +0x24, each chain stepped by +0x60) and for each element allocates and constructs an animation ev… |
| `007bbc20` | AnimTrack_BeginPlayback | likely | If bit 0x10 of the flags at this+8 is clear: sets that bit, creates the track via 00786620 when this+0x14 is null, and for every record of the track list at tr… |
| `007bbc60` | AnimTrack_EndPlayback | likely | If bit 0x10 of the flags at this+8 is set: creates the track if needed, calls vtable +0x24 on the runtime object of every record in the track list at track+0x2… |
| `007bbca0` | AnimTrack_SetClip | likely | Swaps the track's clip: creates the track if needed (returning null when param_2 is null), releases the old clip at track+0x3c (running the global object's vta… |
| `007bbdb0` | AnimTrack_AdvanceAndAutoReset | likely | Per-frame step: calls 007bbb20 (advance), then, when the active clip's remaining time (float +0x9c minus float +0xa0) is <= 0.0, resets the track - 007badc0, o… |
| `007bbe50` | AnimTrackOwner_DtorBody | likely | Destructor body of the animation control block (ECX = this, called from EH unwind funclets): when this+0x14 (the track) is non-null it calls vtable slot 0 on e… |
| `007e3570` | AnimationEffectsRead_AnimData | likely | One stack arg (the reader). Loops over tags: tag 1 opens an effect entry and counts it as pending while no animation name is resolved. Once a name is known, a … |
| `007e4710` | AnimationEffectGroupRead_A | likely | One stack arg (the reader). Loops over tags of an animation group: an effect tag (1) before any name is counted as pending, and after a name it is passed to 00… |
| `007e4b20` | AnimationEffectGroupRead_B | likely | One stack arg (the reader). Same tag logic as 007e4710 (effects before a name are counted as pending, a name resolves the group through 007a46c0, effects are p… |
| `007e4f50` | AnimationEffectGroupRead_C | likely | Byte-identical body to 007e4b20 apart from relocated call and jump displacements and the EH label. Same tag logic and the same two error strings. |
| `007e6190` | AnimationSlotRecordRead | likely | One stack arg (the reader). Loops over tags of an animation record: reads the name and file name strings with 007dfb20 and 007eb2d0, resolves the name through … |
| `007e7380` | DwordListRemoveThenAppend | likely | Thiscall (ECX = owner) with one stack arg (dword). Removes the first entry equal to the arg from the owner list (through 007e7260), creates the owner's list re… |
| `0094fc90` | KeyDescriptorTableLookup | likely | Cdecl with one dword arg: calls 0094d0f0(arg, &local) and gets a node (local receives one dword); the service is taken from 00769710. When the node is non-null… |
| `00950d40` | KeyPairReleaseIfMatch | likely | Cdecl with two args (group key, action value): looks up the group node (0094d0f0), returns 0 when the node or the second arg is null, gets the action index fro… |
| `0096c780` | PlayOrQueueActorAnimation | likely | Thiscall(this=param_1, param_2 animation id, param_3 duration, param_4 flag char, param_5, param_6, param_7 flag). When the flag param_4 is nonzero it stores p… |
| `0096cd80` | DumpAnimationsToMixerLog | likely | Thiscall(this=param_1, char flag). When vtable+0x24 gives an object, it formats a begin line with its name and a string from [object+0x2b0]+0x280, builds a lis… |
| `0096cfe0` | ListAnimationsDiagnostic | likely | Fastcall(ECX=this). Formats a begin line '-- List Animations' with the actor name, calls [this[0x1ce]+8]+0xc, and when the component at this[0x1cd] returns non… |
| `0076fcb0` | AnimRecordPlayableCheck | speculative | Thiscall, one stack arg (ret 4): returns 1 early when the record for the argument (007a2770) has byte +0x67 set; otherwise walks the chain from (this+0x374)+0x… |
| `0076fe00` | ChainMaxD0NodeHookNotify | speculative | Fastcall (ECX = this): walks the chain at (this+0x374)+0xc (next +0xc); for each node looks up node+0xcc in the hash table at this+0x3e8 and calls 007bbdb0() w… |
| `0076ff00` | FindFirstActiveChainNodeKey | speculative | Thiscall, one stack arg (out int*, ret 4): writes 0 to *out, then calls 00769710(0), 007671f0() and returns 007aa6f0(0) as the default result; walks the chain … |
| `0076ffe0` | SelectSlotKeyOrMaxF3c | speculative | Thiscall, four stack args (key, out dword, out float, out float; ret 0x10): walks the chain at (this+0x374)+0x10 (next +8). If key is -1 it selects the node wi… |
| `00770090` | ActiveSlotKeyInIndexedList | speculative | Thiscall, one stack arg (byte argument; ret 4 expected): when this+0x374 has a head node (+0x10) and the first matching hash entry's +0x14 object passes the sa… |
| `00770220` | ActiveSlotRecordByte64 | speculative | Fastcall (ECX = this): if this+0x374 has a head node (+0x10) whose float +0x94 is below DAT_01762a30, maps its key (+0xcc) through 007a7d10 and returns byte +0… |
| `00770270` | BroadcastKeyFloatToTree | speculative | Thiscall, two stack args (key dword, float; ret 8): if the hash table at this+0x3e8 (mask this+0x3d8) holds an entry whose +0x20 equals key, calls 007baf60(flo… |
| `00770630` | RebuildAnimationControlAt378 | speculative | Fastcall (ECX = this): when this+0x378 is null and this+0x2b0 is null or its byte +0x138 is zero, it reads dword +0x1a0, destroys any object at this+0x374 thro… |
| `00770700` | ReleaseAnimationControlAt378 | speculative | Fastcall (ECX = this): if this+0x374 is non-null calls 007ecba0 and clears it; clears this+0x374 again; if this+0x378 is non-null calls its vtable slot [0] wit… |
| `00773430` | TransformIndexedVec4 | speculative | Thiscall, one stack arg (output pointer, index in the second arg; ret 8): when this+0x370 and this+0x1a0 are non-null and 0 <= index < dword((this+0x370)+0xc),… |
| `007737a0` | TimedProgressAdvance | speculative | Thiscall, one float stack arg (delta; ret 4): when bit 0 of this+4 is set and duration this+0xc is positive, advances progress this+8 by delta/duration and cla… |
| `007742d0` | NodeSelectTypeFourOrZero | speculative | Thiscall, one stack arg (key; ret 4): walks the list at this+0x3dc (next +0x18); skipping nodes whose +0x10 equals the argument, takes the first node whose obj… |
| `00786690` | GetOrCreateAnimTrack | speculative | Fastcall (ECX = owner O). If O+0x14 is zero it calls 00786620(O). Returns the dword at O+0x14. |
| `00786d00` | IsNamedAnimReady | speculative | Thiscall (ECX = owner O; stack argument = name). Assigns the owner name at O+0x2b0 (+0x494, or the empty string) into a temporary IString with 00402ce0, resolv… |
| `00786e00` | IsAnimNodePending | speculative | Stack argument = id; ECX = owner O. Returns 1 when 00784c80(id, 0) finds a node N, N is not ready (its +0x14 object or the +0x40 field of that object is null, … |
| `00786fb0` | NodeDeltaAboveThreshold | speculative | Stack argument = id; ECX = owner O. Calls 00784c80(id, 0); returns 1 when the node exists, its +0x14 object is non-null, that object's +0x44 field X is non-nul… |
| `00787060` | IntrusiveListDrain8 | speculative | Fastcall (ECX = list header whose head is at +8). While the head is non-zero it calls 00786010 with ECX = the header and the head as argument. Returns nothing. |
| `00787100` | AnimTrackGetDword44 | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the dword at track+0x44. |
| `00787120` | AnimTrackGetDword60Shr8 | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the dword at track+0x60 shifted right by 8 (unsigned). |
| `00787140` | AnimTrackGetFloat58 | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the float at track+0x58 (x87 return). |
| `00787160` | AnimTrackSetDword58 | speculative | Thiscall (ECX = owner O; one stack argument). Creates the track through 00786620 when O+0x14 is zero, then stores the argument dword at track+0x58. Returns not… |
| `00787180` | AnimTrackGetByte60 | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the signed byte at track+0x60 as an int. |
| `007871a0` | AnimTrackSetByte60 | speculative | Thiscall (ECX = owner O; one stack byte argument). Creates the track through 00786620 when O+0x14 is zero, then stores the byte argument at track+0x60. Returns… |
| `007871d0` | AnimTrackGetFloat50 | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the float at track+0x50 (x87 return). |
| `007871f0` | AnimTrackSetDword50 | speculative | Thiscall (ECX = owner O; one stack argument). Creates the track through 00786620 when O+0x14 is zero, then stores the argument dword at track+0x50. Returns not… |
| `00787240` | AnimTrackSetFloat4cFromInt64 | speculative | Thiscall (ECX = owner O; stack argument = pointer to a 64-bit integer). Creates the track through 00786620 when O+0x14 is zero, then stores (float) of the int6… |
| `00787260` | AnimTrackGetFloat5c | speculative | Fastcall (ECX = owner O). Creates the track through 00786620 when O+0x14 is zero, then returns the float at track+0x5c (x87 return). |
| `00787a00` | AnimScaledTrackFloat50 | speculative | Thiscall (ECX = actor A; param_2 = animation id). Calls 00769710, 007671f0 and 007aa710 with 1 and, when that returns zero, again with 0x1e. Computes _DAT_0175… |
| `00787aa0` | EaseOutNegativeRateEntries | speculative | Thiscall (ECX = actor A; param_3 byte). When A+0x374 has a first element E0 whose id node has a ready track (non-null +0x40 and node+8 bit 3), it reads the 007… |
| `00787d70` | NamedSubEntryValue | speculative | Returns a float. Looks the id (param_1) up with 00784c80; returns 0 when absent. Otherwise it hashes the name param_2 with 00705b00 and walks the node's sub-ta… |
| `0078a740` | NodeValueOverIdDuration | speculative | Returns a float. It returns 0 when 007829a0(id) is below 1 or no node matches. Otherwise it finds the node keyed by that id in the hash table at this+0x3e8 (ma… |
| `0078b390` | Wrapper_0078a460 | speculative | Forwards its two stack arguments to 0078a460 as (arg1, 0, arg2), leaving ECX unchanged so the caller's receiver passes through. Returns with ret 8. |
| `0078b5a0` | SehCallDtorBody_0078a980 | speculative | Installs an SEH frame and calls 0078a980 with this in ECX and no stack argument. The DAT_01b839d8 ^ (uint)local_10 value in the pseudocode is the /GS stack coo… |
| `0078d520` | AdvanceAnimRecordOrReset | speculative | Requires this+0x2c0 nonnull and an empty node list (+0x374 with +0xc zero). Walks the chain of records via 007a27e0 (field +0xc of the current record) and look… |
| `007ba900` | TimelineNode_ActivateIfIdle | speculative | Guard-then-activate on a timed node: returns immediately unless vtable slot +8 returns non-zero; then returns if bit 15 of the 16-bit flags at +0xc is set (or … |
| `007bac30` | Timeline_SetValueIfActive | speculative | When this+0x14 and [[this+0x14]+0x44] are non-null: calls vtable slot +0x14 of the object at [[this+0x14]+0x44]+0x18 (no arguments), passes the float argument … |
| `007bad50` | TimelineNodeList_DestroyAll | speculative | Fastcall list teardown (head +0, tail +4, count +8): unlinks every node (next at node+8, prev at node+4), clears the links, decrements the count and destroys e… |
| `007baf60` | Timeline_ResetTargetIfFinished | speculative | If this+0x14 and [[this+0x14]+0x44] are non-null and the float at [[this+0x14]+0x44]+0x94 is negative (less than the 0.0 constant DAT_01762a30), calls 00779d20… |
| `007c7100` | LookControlBaseCleanupA | speculative | Cleanup routine (66 bytes) reached from six EH unwind funclets: calls 010aff30 on a stack local with the stack cookie value as its argument (Ghidra artifact; t… |
| `007c71f0` | LookControlBaseCleanupB | speculative | Cleanup routine (66 bytes, no callers in this shard): calls 010aff30 on a stack local with the stack cookie value as its argument (Ghidra artifact). |
| `007c75f0` | ClearNodeFlagsAndReleaseRef | speculative | Fastcall, ECX = holder (64 bytes). Walks the node list headed at holder+0x14 (next link at node+0x18) and stores zero in the byte at node+0x14 of every node. T… |
| `007c7630` | FindNodeByFoldedNameHash | speculative | Thiscall, one int argument (53 bytes). Walks the list headed at this+0x14 (next link at node+0x18), hashes each node's string at node+4 with 00705b00 and retur… |
| `007c7700` | NotifyUnflaggedNodesOnRefSet | speculative | Thiscall, one pointer argument (89 bytes). When the count at holder+0x1c is positive, walks the node list headed at holder+0x14 to the first node whose byte at… |
| `007e7260` | DwordListRemoveFirst | speculative | Thiscall (ECX = owner) with one stack arg (dword). Reads the list record at owner+0x14. If its array (begin +0x30, size +0x34) holds the value, it scans for th… |
| `0094cf60` | Wrapper_0094cf60 | speculative | Forwarder: loads the object at [this+0x74]+0xc0 into ECX, pushes the two-dword key (this+0x80, this+0x84) by value, calls 00984960 (hash list find, stdcall 8) … |
| `0094d2e0` | ResetKeyDefaultsAndDispatch | speculative | Fastcall: clears byte this+0x89; sets this[0x28]/[0x29] from DAT_01be5040/DAT_01be5044 and this[0x2e]/[0x2f] from DAT_01be4ff8/DAT_01be4ffc; calls vtable slot … |
| `0094d650` | FindNodeForKeyWithFallback | speculative | Fastcall: looks up the node for the key pair at this+0x80/0x84 in the hash list at [[this+0x74]+0xc0] (00984960); reads the pair returned by 0101b8d0 for this+… |
| `0094e110` | KeyStateListLatchFlags | speculative | Fastcall: for each KeyState pointer in the list at this+0xc8 (node +0 = object, +4 = next), when the object byte +0 is nonzero, sets object+0xd to 1 if object+… |
| `0094e180` | KeyStateListClearFlags | speculative | Fastcall: for each KeyState pointer in the list at this+0xc8, when the object byte +1 is nonzero clears its byte +0xc; then clears bits 0x8, 0x20 and 0x10 of t… |
| `0094e210` | ListClearEntryBytesAndFields | speculative | Thiscall: walks the list at this+0xc8 (node +0 = object pointer, +4 = next) and clears object bytes +0, +1, +0xc and +0xd; then zeroes this+0x20, +0x24 and +0x… |
| `0094e240` | KeyStatePruneList | speculative | Fastcall: walks the list at this+0xc8 (head) and, for each node whose object has +0x18 not equal to 2 or +0xc equal to 0, clears object byte +0x24, unlinks the… |
| `0096d1b0` | PlayAnimationRequestWithMixerClear | speculative | Thiscall with nine stack parameters. When param_2 is nonzero it builds an IString list, uses it only when the byte at this+0x145 bit 7 is set and the owner id … |
| `010ad7a0` | InitThreeVec4Defaults | speculative | Initialises three 4-float records at this+0x30, +0x40 and +0x50 to (0, 0, 0, w) where w is the shared constant DAT_0175b420. |
| `010aef00` | TreeFindKey50OrChainCreate | speculative | Searches the tree at this+0x9c for key param_2 (node+0x50; left +0x58 for smaller keys, right +0x5c for larger). If absent it walks a chain of 12-byte records … |
| `010aef60` | JointNodeSetVectorsAndWeights | speculative | Finds the node for key param_2 through 010aef00; if present stores param_3 at node+0x30, stores min(\|param_4\|, DAT_01885f20) at +0x34 and min(\|param_5\|, DA… |
| `00773810` | — | unknown | Fastcall (ECX = this): when byte +0x236 is set and byte +0x1b0 is clear, picks the reference object (the +0x38 field of this+4, or of 00729610() when this+4 is… |
| `00774330` | — | unknown | Thiscall, one float4 output pointer (ret 4): when this+0x394 is non-negative it takes the vector from 00773430 with that index, otherwise with index this+0x39c… |
| `00780a30` | — | unknown | Fastcall (ECX=this). Returns early unless this+0x370 is non-null and this+0x1a0 is non-null with byte +0x1b0 clear and 0077a130 finds the definition's first na… |
| `007829a0` | — | unknown | Thiscall (ECX=actor-like object; p2 = animation id dword, p3 = optional name pointer; the stack cookie frame is installed). Looks up the definition for p2 in t… |
| `007870d0` | — | unknown | Thiscall (ECX = owner O; stack argument = 8-byte output buffer). Creates the track through 00786620 when O+0x14 is zero, calls 00d83cc6 (no arguments, 8-byte r… |
| `00787210` | — | unknown | Thiscall (ECX = owner O; stack argument = 8-byte output buffer). Same shape as 007870d0: creates the track through 00786620 when O+0x14 is zero, calls 00d83cc6… |
| `00787280` | — | unknown | Fastcall (ECX = owner O). Gets an object from 00773370; when its byte +0x2c is non-zero and its +0x3c object has a non-zero +4 field, it calls 0076ec90, then 0… |
| `00787380` | — | unknown | Thiscall (ECX = owner O; stack argument = name). Assigns the owner name (+0x2b0 object +0x494, or the empty string) into a temporary IString with 00402ce0, res… |
| `00787e40` | — | unknown | Stack arguments (name, subname). Resolves the name to an id with 007829a0(name, 0); returns 0 for an id below 1, otherwise returns 00787d70(id, subname). |
| `0078d2e0` | — | unknown | If this+0x374 is nonzero, walks the list headed at +0x374 (next pointer at node+0xc), probing a bucket of the table at this+0x3d8 (empty loop, no effect) and c… |
| `007a28c0` | — | unknown | thiscall: ECX = animation table (+0x4150 bucket heads by id&0xff, node key at +0x8, next at +0x4), arg1 = animation id, arg2 = candidate id, arg3 = visit count… |
| `007a8d70` | — | unknown | Resolves the two ids param_2 and param_3 to keys through 007a73f0 (uVar3, uVar4) and to type values through the 256-bucket table at this+0x4564 and the 2048-bu… |
| `007bac90` | — | unknown | When this+0x14 and [[this+0x14]+0x44] are non-null, calls vtable slot +0x14 of the object at [[this+0x14]+0x44]+0x18 and then calls 007ecc90 in both branches (… |
| `007bae10` | — | unknown | Advances a timeline to the float argument: walks the node list at +8 (next at node+0x1c) and for each node whose start float at +4 lies in [cursor(+0x18), t) c… |
| `007bb730` | — | unknown | With this = the animation owner: creates the track through 00786620 when this+0x14 is null, links the AnimationEvent record (arg1) into the track's record list… |
| `007bba70` | — | unknown | With this = the animation owner: creates the track via 00786620 when this+0x14 is null, resets it (007badc0, optional 00779d20(0) when the current clip's +0x94… |
| `007bbb20` | — | unknown | Per-frame advance of the track at this+0x14 (created if null): for every record in the track list at track+0x20 (chain via +8) it takes the runtime object from… |
| `007c72d0` | — | unknown | Cleanup routine (66 bytes, no callers in this shard): calls 010aff30 on a stack local with the stack cookie value as its argument (Ghidra artifact). |
| `0094cf90` | — | unknown | Thiscall with one pointer argument (ret 4, returns 1): copies bit 0x8 of the argument byte at +1 to this+0x44 bit 0x8, bit 0x20 to bit 0x10, and bit 0x10 to bi… |
| `0094e2b0` | — | unknown | Thiscall with three pointer arguments: when the key pair at this+0x80/0x84 resolves to a node (00984960), sets byte this+0x89, stores the tick from 007027f0 at… |
| `0094f3c0` | — | unknown | Fastcall (ECX = this): when the pointer at this+0xb0 is set, looks up the node for the key pair at this+0x80/0x84 (00984960). If that node exists, the byte at … |
| `0094fb80` | — | unknown | Thiscall (ECX = this, arg1 = input record): copies the low three flag bits (values 1, 2, 4) of the record byte at +1 into bits 0..2 of the mask dword at this+0… |
| `0094fe10` | — | unknown | Thiscall with three stack args (param_2 frame value, param_3 unused, param_4 byte flag): calls 0094e110; if byte +0xac is set calls 009aaf60; if byte +0x89 is … |

### `anim.definition`

**ActorCompositeEffectReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb4f0` | ActorCompositeEffectReference_Ctor | confirmed | Constructor: runs the ActorEffectReference base constructor 007bb030 from the two dword arguments, stores ActorCompositeEffectReference::vftable at +0 and zero… |

**ActorEffectReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb030` | ActorEffectReference_Ctor | confirmed | Constructor (this in ECX): stores ActorEffectReference::vftable at +0, initialises two empty SoeUtil::IString<char> members at +4 and +0x14, assigns the second… |

**ActorParticleReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb300` | ActorParticleReference_Ctor | confirmed | Constructor: runs the ActorEffectReference base constructor 007bb030 from the two dword arguments, stores ActorParticleReference::vftable at +0 and zeroes +0x3… |

**ActorSoundReference** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb110` | ActorSoundReference_Ctor | confirmed | Constructor: runs the ActorEffectReference base constructor 007bb030 from the two dword arguments, stores ActorSoundReference::vftable at +0 and zeroes +0x30 a… |

**AnimationCompositeEffectReference** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb560` | AnimationCompositeEffectReference_Ctor | confirmed | Constructor: runs 007bb4f0 (ActorCompositeEffectReference part at +0), builds the AnimationEvent sub-object at +0x38 (AnimationEvent vftable, zeroed +0x3c, +0x… |
| `007bb600` | AnimationCompositeEffectReference_Adjustor38_vt3 | confirmed | 4 bytes: `8d41c8 c3` = lea eax,[ecx-0x38]; ret - returns the primary object pointer for the secondary vtable slot 3. |
| `007bb630` | AnimationCompositeEffectReference_Adjustor38_vt2 | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e9b8ffffff` jmp 007bb5f0 (thunk to 007c4570) - forwards the secondary-vtable slot 2 call with the adjusted this. |
| `007bb640` | AnimationCompositeEffectReference_Adjustor38_DtorThunk | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e988000000` jmp 007bb6c0 - tail-jump into the class's scalar deleting destructor. |
| `007bb650` | AnimationCompositeEffectReference_Adjustor38_vt1 | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e988ffffff` jmp 007bb5e0 (thunk to 007c4540). |
| `007bb660` | AnimationCompositeEffectReference_DtorBody | confirmed | Destructor body: stores AnimationCompositeEffectReference vftables at +0 and +0x38, replaces +0x38 with the AnimationEvent vftable, then calls 007772b0 (ActorC… |
| `007bb6c0` | AnimationCompositeEffectReference_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls 007bb660 with this in ECX, frees this with _free when flag bit 0 is set, returns this. |
| `007bb610` | — | unknown | 16 bytes (`8b41c8 8b5008 83c1c8 6a01 6a00 ffd2 c3`, decoded): loads the primary vtable from [ecx-0x38], adjusts ECX by -0x38 and calls primary vtable slot +8 w… |

**AnimationEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bab70` | AnimationEvent_DtorVtableStore | confirmed | 7 bytes: `c701ccf97e01c3` = stores the AnimationEvent vftable (0x17ef9cc) at *this and returns; the cleanup-address stub of an AnimationEvent destructor. |
| `007bab80` | AnimationEvent_ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the AnimationEvent vftable at *this, frees this with _free when the flag argument has bit 0 set, and returns this (31 bytes). |

**AnimationParticleReference** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb370` | AnimationParticleReference_Ctor | confirmed | Constructor: runs 007bb300 (ActorParticleReference part at +0), builds the AnimationEvent sub-object at +0x3c (AnimationEvent vftable, zeroed +0x40, +0x44 and … |
| `007bb410` | AnimationParticleReference_Adjustor3c_vt3 | confirmed | 4 bytes: `8d41c4 c3` = lea eax,[ecx-0x3c]; ret - returns the primary object pointer for the secondary vtable slot 3. |
| `007bb440` | AnimationParticleReference_Adjustor3c_DtorThunk | confirmed | 8 bytes: `83e93c` sub ecx,0x3c then `e988000000` jmp 007bb4d0 - tail-jump into the scalar deleting destructor of the class. |
| `007bb450` | AnimationParticleReference_Adjustor3c_vt2 | confirmed | 8 bytes: `83e93c` sub ecx,0x3c then `e9a8ffffff` jmp 007bb400 (thunk to 007c3ec0). |
| `007bb460` | AnimationParticleReference_Adjustor3c_vt1 | confirmed | 8 bytes: `83e93c` sub ecx,0x3c then `e988ffffff` jmp 007bb3f0 (thunk to 007c3e90). |
| `007bb470` | AnimationParticleReference_DtorBody | confirmed | Destructor body: stores AnimationParticleReference vftables at +0 and +0x3c, overwrites +0x3c with the AnimationEvent vftable, then calls 00777210 (ActorPartic… |
| `007bb4d0` | AnimationParticleReference_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls 007bb470 with this in ECX, frees this with _free when flag bit 0 is set, returns this. |
| `007bb420` | — | unknown | 16 bytes: `8b41c4 8b5008 83c1c4 6a01 6a00 ffd2 c3` = loads the primary vtable from [ecx-0x3c], adjusts ECX by -0x3c and calls primary vtable slot +8 with the a… |

**AnimationSoundReference** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bb180` | AnimationSoundReference_Ctor | confirmed | Constructor of the multiply-inherited record: runs 007bb110 (ActorSoundReference part at +0), then builds the AnimationEvent sub-object at +0x38 (stores Animat… |
| `007bb220` | AnimationSoundReference_Adjustor38_vt3 | confirmed | 4 bytes: `8d41c8 c3` = lea eax,[ecx-0x38]; ret - returns the primary object pointer for the secondary vtable's slot 3. |
| `007bb250` | AnimationSoundReference_Adjustor38_DtorThunk | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e988000000` jmp 007bb2e0 - tail-jumps into the class's scalar deleting destructor with the adjusted this. |
| `007bb260` | AnimationSoundReference_Adjustor38_vt2 | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e9a8ffffff` jmp 007bb210 (the thunk to 007c38e0) - forwards the secondary-vtable slot 2 call to the class's real implement… |
| `007bb270` | AnimationSoundReference_Adjustor38_vt1 | confirmed | 8 bytes: `83e938` sub ecx,0x38 then `e988ffffff` jmp 007bb200 (the thunk to 007c38b0). |
| `007bb280` | AnimationSoundReference_DtorBody | confirmed | Destructor body: stores AnimationSoundReference vftables at +0 and +0x38, overwrites +0x38 with the AnimationEvent vftable (unwinding the sub-object), then cal… |
| `007bb2e0` | AnimationSoundReference_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 007bb280 with this in ECX, frees this with _free when the flag argument has bit 0 set, an… |
| `007bb230` | — | unknown | 16 bytes: loads the primary vtable from [ecx-0x38], adjusts ECX by -0x38 and calls primary vtable slot +8 with the arguments (0, 1) - the classic deleting-dest… |

**AnimationTrigger** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bba50` | AnimationTrigger_ScalarDeletingDtor | speculative | Scalar deleting destructor: calls the AnimationTrigger destructor body 007bb970 with this in ECX, frees this with _free when flag bit 0 is set, returns this. |
| `007bb970` | — | unknown | Destructor body: stores AnimationTrigger::vftable at *this, destroys the timeline node list via 007bad50, releases the two refcounted pointers at +0x40 and +0x… |

**(no class)** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a6410` | AnimationTypeTableLoad | confirmed | Reads the AnimationTypes table rows (AnimationType entries): for each row takes two text fields via 00d517f0 (key DAT_017eda5c for the name, DAT_017eda64 for t… |
| `007a6ac0` | SaveAnimationParametersXml | confirmed | Builds an AnimationParameters root element (00d52e70), then fills it through 007a4550 (AnimationTypes section), 007a2cd0 (MountTypes section) and 007a6650 (Ani… |
| `007a6bf0` | AnimationSlotInsertUnique | confirmed | Inserts an AnimationSlot record (param_2) into the id table: 2048 buckets at this+0x10 keyed by id & 0x7ff, global list head/tail at this+0x4/+0x8, count at th… |
| `007a7e40` | MountTypeTableLoad | confirmed | Reads the MountTypes table rows (MountType entries): per row reads the numeric field (key DAT_017eda64, parsed with 0072d5d0), 'animSlotPrefix' and 'descriptio… |
| `007a8390` | AnimationSlotTableLoad | confirmed | Reads the AnimationSlot rows: name (key DAT_017eda5c), id and type (keys DAT_017eda64, DAT_017ee664, parsed with atol), priority (atol; a negative value is log… |
| `007a8920` | BoneMaskTableLoad | confirmed | Reads the BoneMaskSuffixes/Suffix rows into a suffix list through 0079b860. Then for each BoneMask row: reads its name (key DAT_017eda5c; a missing name is log… |
| `007a9130` | LoadTypeEnumTableLoad | confirmed | Reads the LoadTypeEnums/LoadType rows: for each row reads the numeric id (00d51810, key DAT_017eda64) and the name (key DAT_017eda5c), scans the 8-bucket chain… |
| `007a9440` | AnimationTreeBranchLoadRecursive | confirmed | Allocates a 0x28-byte record (operator new) and constructs it through 007a5ed0 (dword at +0x10 = the running counter *param_3, IString at +0x14 from the name f… |
| `007a9840` | AnimationTreeLoad | confirmed | Reads the AnimationTree row (00d50af0). For each AnimationTreeBranch row calls 007a9440 (ECX = this) with a zero counter and -1; after the loop, when the byte … |
| `007a9c80` | AnimationSlotTablesDtorBody | confirmed | Destructor body of the table object: runs 007a98b0 (ResetParameterTables) on this, releases the Set at +0x4ef0 (007a2470 on both children, then the vtable slot… |
| `007a9db0` | AnimationGroupTableLoad | confirmed | Copies the path at this+0x49f4, appends AnimationGroups.xml through 00402770, opens it and checks the root 'AnimationGroup' rows. For each AnimationGroup row r… |
| `007aa2d0` | AnimationParametersLoadFiles | confirmed | Loads the animation parameter files in order, each guarded by a byte flag: when this+0x4a04 is zero, copies the path at this+0x49f4, appends AnimationTypes.xml… |
| `007b9ad0` | AnimationActionPointHashList_Ctor | confirmed | Constructor (this in ECX): stores the SoeUtil::HashListMap<int,AnimationActionPointDefinition,0,-1> vtable then overwrites it with the SoeUtil::HashList<Animat… |
| `007b9b30` | AnimationActionPointHashList_ScalarDeletingDtor | likely | Scalar deleting destructor for the action-point hash list object: calls 00785f40 (the HashListMap member destructor body, stores the HashListMap vtable and dra… |
| `007b9b90` | AnimationSlotRecord_Ctor | likely | Constructor (this in ECX) of a record that holds three SoeUtil::List<AnimationReferenceDefinition,-1> headers at +0, +0x10 and +0x20 (each vtable plus three ze… |
| `007b9c70` | AnimationSlotRecord_CtorSixArgs | likely | Constructor wrapper (this in ECX): calls 007b9b90 to build the base record from four of the five argument pointers with the constant flag 1, then stores the dw… |
| `007b9d00` | AnimationRecordHashList_Insert | likely | Inserts a new animation record into the container at this (vtable +0 slot +4 allocates, order-list head +8 / tail +0xc, count +0x10, 100 bucket heads starting … |
| `007bbe30` | AnimationReferenceList_ScalarDeletingDtor | likely | Scalar deleting destructor: calls 0078a980 (the derived destructor body whose primary base is SoeUtil::List<AnimationReferenceDefinition,-1>) with this in ECX,… |
| `007a73f0` | IdTwoStepNameLookup | speculative | Looks up the 2048-bucket record with id param_2 (chain at this+0x10, key at node+0x4). If found and node+0x30 is positive, finds the record with key node+0x30 … |
| `007a7780` | HashStringKeyInsertAndRegister | speculative | Hashes the string param_2 through 00705b00 and looks the hash up with 007a31e0; if it is absent, inserts it through 007a5140 and calls 00fc0a50 with param_2. T… |
| `007a7a70` | NameKeyedListClearAndRelease | speculative | Frees the block at this+0x230 through 010aac70 and _free when non-null. Then while the list head at this+0x24 is non-null: unlinks it through 007a30e0 (128-buc… |
| `007a7b30` | NameKeyedNodeAppend128 | speculative | Hashes the string param_2 through 00705b00, allocates a 0x24-byte node (00d833a6), constructs it through 007a62d0 (C string param_2, float param_3 stored at no… |
| `007a7be0` | ResolveIdByFormattedNameKey | speculative | Returns param_2 when the count at param_3+8 is zero. Otherwise finds the 2048-bucket record with id param_2 and, when found, formats '%s_%s' through 00402fd0 w… |
| `007a98b0` | ResetParameterTables | speculative | Sets the dword at this+0x401c to -1 and clears the byte at this+0x4a06; calls 007a1650, then 007a54b0 (free the 2048-bucket list), clears the byte at this+0x4a… |
| `007a9940` | WeightedStringElementAppend | speculative | Appends an element through 007a96c0 (array push-back), assigns the string param_2 into its IString at +0 through 00402ce0, stores float param_3 at +0x10, dword… |
| `007aa5d0` | ResetThenLoadFiles | speculative | Calls 007a98b0 (ResetParameterTables) on this, then 007aa2d0 with argument 0 (AnimationParametersLoadFiles without the tree re-sort). |
| `007a6650` | — | unknown | Writes the AnimationTree section into an XML document: an AnimationTree element with 16 TiXmlComment documentation lines (007a10f0), one AnimationTreeBranch el… |
| `007a7920` | — | unknown | Calls 00fc0a50 with param_2, formats '%s_%s' through 00402fd0 from a Ghidra local and param_3, hashes the result through 00705b00 and walks the 32-bucket table… |
| `007a7d10` | — | unknown | Finds the 2048-bucket record with id param_2, then the record with key node+0x30 in the 32-bucket table at this+0x4974. If that record's dword at +0x1c is nonz… |
| `007a9a30` | — | unknown | Constructor for a large table object (ECX = this, no stack arguments): zeroes the 2048-bucket table at this+0x10 (memset 0x2000) through the head fields, const… |

### `anim.skeleton`

**Deep::ModelEntity** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ac650` | ModelEntity::SetSkeleton | confirmed | Installs an owned skeleton from SkeletonDefinition param_2: a definition with zero bones sets this+0x1ac to 'SkeletonDefinition set with 0 bones' and changes n… |
| `010ac770` | ModelEntity::SetBorrowedSkeleton | confirmed | Points the entity at a borrowed Skeleton (param_2): a non-null skeleton whose definition (+8) has zero bones sets this+0x1ac to 'borrowed Skeleton with 0 bones… |

**Deep::Skeleton** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b3e70` | Skeleton::~SkeletonBody | confirmed | Destructor body of Deep::Skeleton: stores Deep::Skeleton::vftable; releases the counted object at this+8 (LOCK-decrement of its count at +4, calling its vtable… |
| `010b4050` | Skeleton::Skeleton | confirmed | Constructor of Deep::Skeleton taking a counted source object: calls 0071b950 (RefCounted base constructor), stores Deep::Skeleton::vftable, stores the source a… |
| `010b4180` | Skeleton::~Skeleton | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::Skeleton: calls 010b3e70 (the Skeleton dtor body) on this, then if bit 0 of the flag argument is set frees… |

**Deep::SkeletonDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ab760` | SkeletonDefinition::~SkeletonDefinition | confirmed | Destructor body of Deep::SkeletonDefinition: stores its vtable, frees the bone buffers at +0x10, +0x14, +0x18 and +0x1c through _free, releases the Array<short… |
| `010ac390` | SkeletonDefinition::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape for Deep::SkeletonDefinition (vtable slot [0]): calls the destructor body 010ab760 on ECX, frees this with _free when flag bit… |
| `010ab900` | — | unknown | Constructor that loads a skeleton definition from a DSKE stream (param_2): runs the RefCounted, integer-array and hash member constructors (0071b950, 00b07d60,… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a6100` | BonePoseUpdateFromModel | likely | Thiscall (ECX = this; stack args: world matrix pointer, unused, model pointer): copies 16 dwords from the matrix argument into this+0x20. With no model it writ… |
| `010a6470` | PoseBoundsUnion | likely | Thiscall (ECX = this, stack args: output box pointer, object pointer, flag byte; returns the output box): with no bones it copies the 8-dword box at (this+0x10… |
| `010ad2e0` | Wrapper_010b44b0_This0c | speculative | Tail-jumps to 010b44b0 with ECX = this+0xc, so the call operates on the object stored at +0xc. |
| `010b41a0` | ParentLocalMatrixPropagate | speculative | Recomputes the output block at this+0xc: when the child at this+0xa4 is null or its vtable slot 2 (+8) returns false, first memcpys count*0x40 bytes from the s… |
| `010b44b0` | BoneHierarchyUpdate | speculative | Recomputes entries of the output block at this+0xc from the source entries at this+0x10 for the bone index argument: when the argument is negative, copies the … |

### `lib.granny.aggregate`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00efc450` | AggregateAdvanceCursor | speculative | Reads the flag byte at ecx+9. If it is set, returns ecx+0x10 + the old cursor (ecx+0xc) and adds edx to the cursor; otherwise adds edx to the cursor and return… |
| `00f0eb40` | AggrOpenTables | speculative | Reserves a 0x2c-byte block in the aggregate context (param_1) through 00f24ef0, then reserves sub-blocks at offsets +0xc and +0x10 of it sized by 00f32a20 and … |

### `lib.granny.alloc`

**(no class)** (38)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f0ae00` | GrannyMemoryAllocTracked | confirmed | Computes total = alignment(param_3) + 0x1f + size(param_4), allocates the raw block through 00f0b0d0 (HeapAlloc), aligns the user pointer to param_3 with 00f25… |
| `00f0aee0` | GrannyMemoryFreeTracked | confirmed | Ignores a NULL pointer. Under the lock (00f0b110), if the header at user-0x20 holds magic 0xca5eca5e and its size field (user-0x1c) is not -1, sets that field … |
| `00f0b140` | ArenaHeaderAlloc | confirmed | Calls the allocation hook PTR_FUN_01b9378c (resolved by Ghidra to 00f0ae00) with ("..\source\granny_memory_arena.cpp", 0x25, alignment 4, size 8); if the resul… |
| `00f0b1d0` | ArenaChainFree | confirmed | Walks the singly linked list whose next pointer is at +4 of each node, starting at param_1; for each node calls the free hook PTR_FUN_01b93790 (resolved by Ghi… |
| `00f0b200` | ArenaAllocBlock | confirmed | Returns 0 for a zero size and reports '%s is likely badly sign coerced' for a negative size (granny_memory_arena.cpp line 0x61). Otherwise builds an aggregate … |
| `00f17fb0` | StackAllocatorNewBlock | confirmed | Creates a new allocator block. Reads UnitSize (dword at the allocator in EAX) and UnitsPerBlock (dword +4), creates a 0x10-byte block header from the aggregate… |
| `00f18030` | StackAllocatorInit | confirmed | Initialises an allocator record at param_1: dword +0 = UnitSize (param_2), +4 = UnitsPerBlock (param_3), +8 = 0 (next index), +0xc = 0 (head block), +0x10 = -1… |
| `00f18260` | StackAllocatorPopUnits | confirmed | Pops param_2 units from the top of the allocator. While the head block's count (block+0) is no more than the remaining count, it subtracts that count from the … |
| `00f18310` | StackAllocatorRelease | confirmed | Pops every unit (00f18260 with the allocator counter at +8), then frees the block table at +0x1c through the free hook (PTR_FUN_01b93790, granny_stack_allocato… |
| `00f1bcd0` | PoolCursorAdvance0x190 | confirmed | Advances a pool cursor for 12-byte entries (cursor at this+0, list head at this+0x10). When the cursor is null it allocates a 400-byte (0x190) block through th… |
| `00f1ed10` | PoolCursorAdvance0x590 | confirmed | Advances a pool cursor for 44-byte (0x2c) entries (cursor at this+0, list head at this+0x10). When the cursor is null it allocates a 0x590-byte block (0x10 hea… |
| `00f24cf0` | SetAlignmentChecked | confirmed | Stores p2 as the alignment in dword 0 of the context p1 when p2 > 0. A zero value reports Alignment is not allowed be <= 0 (level 3, line 0x2c) and a negative … |
| `00f26f00` | FixedAllocatorFindOrCreateBlock | confirmed | Scans the allocator block list (head at [5], sentinel at ctx+8): returns the first block whose free-list head [2] is nonzero or whose count [0] is below the ca… |
| `00f270a0` | FixedAllocatorAllocUnit | confirmed | Asserts Allocator.UnitSize > 0 (report line 0x9a, level 3, returns 0 otherwise). On first use it raises the unit size to at least 4, sets a preset block size t… |
| `00f27140` | FixedAllocatorFreeUnit | confirmed | Finds the block of the allocator (list from [5]) whose data range contains p2 and reports Memory block 0x%x was not allocated by this allocator (level 3, line … |
| `00f0b110` | SpinLockAcquire | likely | Loops on InterlockedCompareExchange(&DAT_01bff660, 1, 0) until it returns a value other than 1, i.e. takes the global lock flag at DAT_01bff660. |
| `00f0b130` | SpinLockRelease | likely | Stores 0 to DAT_01bff660, releasing the lock taken by 00f0b110. |
| `00f18170` | StackAllocatorUnitCount | likely | Returns the dword at param_1+8 (the allocator's next-index / live unit counter). Bytes: mov eax,[esp+4]; mov eax,[eax+8]; ret. |
| `00f18180` | StackAllocatorAllocUnit | likely | Allocates one unit. Writes -1 to *param_2 (or a stack slot when param_2 is null). If there is no current block, or the current block is full and the table has … |
| `00f18210` | StackAllocatorUnitAddress | likely | Returns the address of unit index param_2. With no block table (+0x1c = 0) it starts at the head block (+0xc chain, newest first) and, when the index is below … |
| `00f18350` | StackAllocatorAllocFill | likely | Allocates param_2 consecutive units (returns 0 when param_2 < 1). The first unit index is written to *param_3. When param_4 is non-null the first unit is fille… |
| `00f0af60` | TrackedListHeadGet | speculative | Returns the global tracked-allocation list head stored in PTR_DAT_01b93784 (no arguments). |
| `00f0af80` | TrackedListSentinelAddr | speculative | Returns the address DAT_01b9376c (no arguments); the value is the list sentinel that 00f0ae00 stores into each header's next field. |
| `00f0b3b8` | ArenaAllocConcat | speculative | Gets two lengths from 00f1b0e0 (_Size from the first call, iVar1 from the second), allocates _Size + iVar1 + 1 bytes from arena param_2 through 00f0b200, copie… |
| `00f24cd0` | ZeroInitContextAlign4 | speculative | Initialises an aggregate context at p1: dword 0 = 4 (default alignment), dword 1 = 0, dword 2 = 0, and the byte at +0xc = 0. |
| `00f24d40` | SetFlagByteAt0c | speculative | Stores byte 1 at p1+0xc (mov byte [eax+0xc],1; ret). |
| `00f24d50` | AggrReserveBlockEsi | speculative | Works on the context in ESI and an output pointer in EDI (registers not recovered by Ghidra). It sets ctx[1] = 00f25820(ctx[1], ctx[0]) (round the offset up to… |
| `00f24db0` | AggrReserveBlockSize | speculative | Works on the context in ESI: rounds ctx[1] up to ctx[0] with 00f25820, appends a 7-dword block record with field +8 = p1 and link -1, and advances ctx[1] by p2. |
| `00f24ef0` | AggrOpenBlockDword | speculative | Calls 00f24d50(p3) (block reservation in the ESI context), then returns the dword at p1 + (index+1)*0x1c, where index is the value 00f24d50 returned. |
| `00f24f50` | AggrOpenBlockDwordB | speculative | Calls 00f24db0(p2+p3, p4) and returns the dword at p1 + (index+1)*0x1c, where index is the value 00f24db0 returned. |
| `00f24f80` | AggrOpenBlockArray | speculative | Calls 00f24e10(p2+p3, p2+p4, p5, p5*p6) (reserves an array of p6 elements of p5 bytes) and returns the dword at p1 + (index+1)*0x1c, where index is the value 0… |
| `00f25040` | AggrFixupBlockPointers | speculative | Walks the block records of the context in EDI (count at +8, records from +0x14, stride 0x1c). With base ESI zero it clears each non-null pointer slot. With bas… |
| `00f250c0` | AggrAllocateBlock | speculative | Rounds the context size ctx[1] up to ctx[0] with 00f25820, calls the allocation hook at 0x01b9378c with (p2, p3, ctx[0], the rounded size), zeroes the result w… |
| `00f25150` | AggrEndSizeAndReset | speculative | Rounds ctx[1] up to ctx[0] with 00f25820, stores it back in ctx[1], zeroes the whole 0x710-byte (1808-byte) context with _memset, and returns the rounded size. |
| `00f25240` | AllFourFieldsNonZero | speculative | Returns 1 when dwords +4, +8, +0xc and +0x10 of p1 are all nonzero, else 0. |
| `00f0af90` | — | unknown | Writes five dwords into the output array param_2 from the header pointer param_1: [0] = param_1+0x20 (user pointer), [1] = *(param_1+4) (size), [2] = *(param_1… |
| `00f24e10` | — | unknown | Works on the context in ESI: rounds ctx[1] up to ctx[0] with 00f25820, appends a 7-dword block record that stores p2 and p3 and p1 in its fields with link -1, … |
| `00f25120` | — | unknown | If p4 is nonnull and the flag byte at p1+0xc is set, zeroes p1[1] bytes of p4 with _memset; runs 00f25040 with the context as base; returns p4. |

### `lib.granny.binding`

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00eed740` | AnimationBindingTotalDeltas | confirmed | Computes the total position and orientation deltas of an animation binding: when the binding's record at param_1+0x28 has a nonzero position curve (+4 pointer,… |
| `00eee440` | FlushAllBindingsForAnimation | confirmed | Looks up the first tree node at or after a stack probe whose first dword equals param_1 (00eed490 on DAT_01bfe1f4). While the found node's first dword equals p… |
| `00eedc10` | BuildTrackGroupBinding | likely | Builds one 0x40-byte binding node from an eight-dword key (dword 0 = animation pointer, dword 5 = track group pointer, dwords 6 and 7 = rebasing range). It cop… |
| `00eee0e0` | BindingNodeDestroy | likely | Destroys the binding node in ESI. It removes the node from the tagged tree (00eecb80 when the node equals DAT_01bfe1f8, 00eecc10 when it equals DAT_01bfe1fc, e… |
| `00eee180` | BindingCacheTrimIdle | likely | While DAT_01bfe21c (live binding count) is greater than DAT_01bfe204 and the idle-list tail DAT_01bfe1f0 is non-null, it loads the tail into ESI and destroys i… |
| `00eee1c0` | BindingRefAcquire | likely | Takes one reference on the binding node in EAX. When the refcount at +0x34 is zero the node is unlinked from the idle list (head DAT_01bfe1ec, tail DAT_01bfe1f… |
| `00eee290` | BindingFindOrBuildAcquire | likely | Increments the call counter DAT_01bfe214 and copies the 8-dword key at param_1 to a stack probe. It searches the binding tree DAT_01bfe1f4 with 00eed400: on a … |
| `00eee3a0` | BindingRefRelease | likely | Increments the counter DAT_01bfe218. If the node pointer argument is non-null it decrements the refcount at +0x34; when that reaches zero it decrements DAT_01b… |
| `00eee64b` | BindingNodeRekey | likely | Removes the node in the stack argument from the binding tree (00eecb80, 00eecc10 or 00eecca0 as in 00eee0e0). It stores the animation pointer in EDI as the nod… |
| `00eee790` | AnimationBindingComputeTrackBindings | likely | Validates param_1 with diagnostics via 00f07e00 (lines 0x3aa ID.OnModel null, 0x3ac OnSkeleton null, 0x3ae BoneCount range against the skeleton bone count, 0x3… |
| `00eef1c0` | VertexMorphCurvesEvaluate | likely | Visits the morph-curve list of param_1 (curve count at +0x14, 0x1c-byte entries from +0x18). It compares each curve's attribute name (00f1b100) with VertexMorp… |
| `00eeda90` | TrackRecordClassifyCurvesAndEvalPose | speculative | Operates on the 0x54-byte track record in EAX. Clears flag byte +3. If the record's track pointer (+4) is null it writes an identity transform to +0x10 (00eeaf… |
| `00eee530` | BindingFindFirstByAnim | speculative | Builds an 8-dword probe with dword 0 = param_1 and the rest zero, runs the tree upper-bound search 00eed490 on DAT_01bfe1f4, and returns the found node only wh… |
| `00eeecd0` | BindingTrackFloatStore | speculative | Fastcall (node in EDX, float argument on the stack, merge flag in BL). It reads the node's group (anim+0x14[node+4]) and, when the group count at +0x14 is posi… |
| `00eeed40` | BindingListForEachScale | speculative | Walks a list whose link is the dword at +8: it starts from the +8 of the value that 00eebc80 reads through 00ef5690 (an identity function) and stops when the w… |
| `00eeee00` | GroupSetCreate | speculative | Allocates a 0xc-byte block through the allocation hook (PTR_FUN_01b9378c = 00f0ae00) with file granny_animation_binding.cpp, line 0x4e3, alignment 4. Stores pa… |
| `00ef64f0` | InstanceBindingAttach | speculative | When *param_2 is zero it acquires the binding for the key at param_2+4 with 00eee290; otherwise it acquires the existing binding *param_2 with 00eee340. It sto… |
| `00eee340` | — | unknown | Same refcount transition as 00eee1c0 for the binding node passed on the stack: at refcount zero it unlinks the node from the idle list and sets DAT_01bfe1e8 to… |
| `00ef65f0` | — | unknown | For each track record in the index range (optionally through the index array param_9) whose track pointer at record+4 is nonzero: it decodes the local pose wit… |

### `lib.granny.bink`

**(no class)** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f27af0` | Bink0AllocateTables | likely | Computes the table size from p5 rows, p4 and fixed sizes 0x40 and 0x100 (00f34a90, 00f34a50, 00f34a70), rounds it up to 16 bytes, and allocates it through 0x01… |
| `00f27d80` | Bink0EncodeBlock | likely | Bink0 block encoder: writes a bit-stream of one 2-D block of 16-bit samples (block at p2, width p4, row stride p3, p5 rows). It first writes a 16-bit header fr… |
| `00f28610` | Bink0EncodeNonZeroRowFlags | likely | For each of the rows of the block it writes one flag byte in a temporary buffer (1 when any sample in the row is nonzero, else 0) and counts the zero rows. It … |
| `00eefe10` | ColorLiftInverseClampRows | speculative | Inverse of the lifting transform in 00eefa00 over four 16-bit planes (param_2..param_5): t = plane0 - ((plane1 + plane2 + bias) >> 2) (bias 3 for negative sums… |
| `00eeff50` | PlanesShiftRight2ClampRows | speculative | Converts four 16-bit planes to four clamped bytes per pixel: each plane sample is shifted right by 2 (arithmetic) and clamped to 0..255. Output byte order is t… |
| `00ef0160` | DimensionPairReduceByMode | speculative | Reads a dimension pair through ESI (*ESI) and EDI (*EDI) and returns 0 when their product is below 0x101. Mode 0 calls 00f2c560 on the pair; mode 1 calls 00f2c… |
| `00ef02f0` | GrannyBinkFrameForward | speculative | Frame routine in granny_bink.cpp. Flag bit 2 selects a codec path: mode 0 uses 00f2c4f0 and mode 1 uses thunk 00f356b0 to size the output (local_38). When the … |
| `00ef0580` | GrannyBinkFrameInverse | speculative | Frame routine in granny_bink.cpp, data flow opposite to 00ef02f0. In the codec path (flag bit 2 selects 00f2c580 or thunk 00f356d0 for the scratch size) it all… |
| `00f27510` | ShortArrayMaxDeltaScan | speculative | __thiscall: scans the shorts at p1 (count EAX-1 after the first) and the p3 rows strided by p2, computing the largest absolute residual starting at -0x8000 (fi… |
| `00f27a60` | ShortRowsAllEqualMaxAbs | speculative | __thiscall: compares p1 rows of p3 shorts (row stride from EAX) against the first value, clearing the result when any element differs. It stores the maximum ab… |
| `00f27c50` | ScanLeadingZeroShorts | speculative | __fastcall: counts zero shorts in a strided block (row length p1, p3 rows, row step p4-p5) until the first nonzero value. It stores the current pointer, the re… |
| `00f27ca0` | FindZeroRunGt3 | speculative | __fastcall: scans a strided block for a run of zero shorts. It accumulates the count of samples scanned before the run; when the run is longer than 3 it stores… |
| `00f28700` | Bink0ThresholdFlags | speculative | Writes p2 flag bytes to p1. Each byte is 1 when the ratio ((ctx[6]-ctx[5]+1)*p2 - 1) / (ctx[4]-ctx[5]+1) (unsigned, with the field offsets read from the ESI co… |
| `00f28790` | ShortAbsHistogram | speculative | __thiscall: zeroes EAX+1 dwords at p5 with _memset, then for each sample of a p4-row, p3-column block of shorts (row stride p1) increments the histogram bin p5… |
| `00f288a0` | Bink0BestSplitSearch | speculative | Iterates a split index from ESI[3] to ESI[6], calling 00f287f0 for each candidate with the segment length from the context. It keeps the candidate with the low… |
| `00eefa00` | — | unknown | For each source row of param_9 four-byte samples (stride param_8 bytes) it writes four 16-bit planes: param_1 = (b2 + 2*b1 + b0) >> 2, param_2 = b0 - b1, param… |
| `00eefc20` | — | unknown | For each source row of param_9 four-byte samples (stride param_8) it writes param_1 = b0*4, param_2 = b1*4, param_3 = b2*4 (16-bit) and param_4 = ((dword >> 22… |
| `00f27630` | — | unknown | Calls 00f27510 on the sample block. When the block is constant it writes a flag bit, the constant (masked) and returns. Otherwise it writes a 0 flag and the fi… |
| `00f285a0` | — | unknown | __thiscall: copies p1 rows of p4 shorts from p3 to p2, dividing each by p5 (1 when p5 is zero). The input row stride is (EAX - p4) elements. |
| `00f287f0` | — | unknown | Over the int array p1 it visits segments of p3 elements (up to the end index), accumulating for each segment with a non-zero sum S the term -(S*S)/p2 (double, … |

### `lib.granny.callback`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ef7c60` | CallbackInvokeAndDeref | speculative | Loads obj = dword at param_1+0x18, calls the function pointer stored at obj's first dword (call edx) with param_1 as the only argument, and returns the dword t… |

### `lib.granny.compression`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f0edb0` | Oodle1DecompressSegments3 | confirmed | Copies the 0x24-byte header at param_3 (9 dwords: three 12-byte stream states), byte-swaps it when param_1 is set (00f259b0), initializes an input reader over … |
| `00f0ef10` | Oodle1DecompressSingle | confirmed | Copies the 3-dword header at param_3 (byte-swapped when param_1 is set, 00f259b0), initializes an input reader with 00f33ea0, allocates the working block throu… |

### `lib.granny.container`

**(no class)** (67)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f0f0a0` | ContainerPoolCursorAdvance | confirmed | Thiscall with ESI = cursor record. When the cursor is null it allocates a 0x210-byte block (contain.inl line 0x37f, alignment 4) through the allocation hook an… |
| `00f0fca0` | ContainerBlockListFreeAll | confirmed | Frees every block in the list at param_1+0x10 through the free hook (contain.inl line 0x395), following each block's first dword as the next link, then zeroes … |
| `00f0fce0` | PointerHashCreate | confirmed | Allocates a 0x14-byte header through the allocation hook (granny_pointer_hash.cpp line 0x42, alignment 4), zeroes its fields at +4, +8, +0xc and +0x10, then al… |
| `00f0fd50` | PointerHashDestroy | confirmed | If the hash header is non-null, frees its blocks with 00f0fca0 and then frees the header through the free hook (granny_pointer_hash.cpp line 0x53). |
| `00f0fd80` | PointerHashInsert | confirmed | Takes a node from 00f0f0a0 and stores key param_2 at +0 and value param_3 at +4, then inserts it with 00f0f890. When no node is available it reports 'Out of sp… |
| `00f0fe20` | PointerHashSet | confirmed | Looks up key param_2 in the tree at param_1; when found it stores param_3 in the node's value and returns 1. Otherwise it takes a node from 00f0f0a0, stores ke… |
| `00f10bc0` | RetargeterFindOrBuild | confirmed | Takes From (param_1, Skeleton at +4) and To (param_2, Skeleton at +4). Validates both skeletons (NULL and BoneCount checks, each with a granny_retargeter.cpp a… |
| `00f10e60` | RetargeterRecordRelease | confirmed | Drops one reference on the retarget record at param_1 (refcount at +0xc, decremented). When it reaches zero it unlinks the record from the global cache: if it … |
| `00f1c960` | ContainerBlockListFreeAllReset | confirmed | Frees every pool block in the list at param_1+0x10 through the free hook (PTR_FUN_01b93790, contain.inl line 0x395), following each block's first dword as the … |
| `00f1c9a0` | StringTableCreate | confirmed | Allocates a 0x1c-byte string table header (granny_string_table.cpp line 0x4a) through the alloc hook, zeroes its +0x4, +0x8, +0xc and +0x10 fields, then alloca… |
| `00f1ca20` | StringTableDestroy | confirmed | If the table is non-null, frees each entry on its text list at +0x18 (next link at entry+0x8; source line 0x62) through the free hook, then releases the pool b… |
| `00f1ca70` | StringBufferCreate | confirmed | Creates a 12-byte buffer record from the aggregate context (00f24cd0, source granny_string_table.cpp line 0x71), reserves param_1 bytes of storage at its first… |
| `00f1caf0` | StringTableAppend | confirmed | Appends a string of param_2 characters from param_3 (copied when non-null) plus a NUL to the table's chunk chain (head at +0x18, chunk size at +0x14, default 4… |
| `00f21450` | ResortTrackGroup | confirmed | Re-sorts the track group builder's entries by their remapped track value. It allocates a 0x590-byte pool (contain.inl line 0x37f) and a tagged tree header on t… |
| `00f215f0` | TrackGroupBuilderArrayAlloc | confirmed | When the builder's array count at +0x14 is zero, sets it to the count at +0xc and allocates count*4 bytes through the alloc hook (granny_track_group_builder.cp… |
| `00f21620` | TrackGroupBuilderArrayFree | confirmed | If the count at +0x14 is positive, frees the array at +0x18 through the free hook (granny_track_group_builder.cpp line 0x377) and sets +0x18 and +0x14 to 0. |
| `00efde70` | ContainerPoolSlotAcquire | likely | Takes a 0x18-byte slot from the pool headed at *ESI. When the head is null it allocates a 0x310-byte block through PTR_FUN_01b9378c (contain.inl line 0x37f, al… |
| `00efe650` | RbTreeInsertNode | likely | Inserts a new node (param_1) into the tree whose root pointer is at param_2+0xc: if the tree is empty it becomes the root and the min/max links at +4 and +8 ar… |
| `00efe770` | ContainerPoolGrow | likely | Zeroes the pool fields (+4, +8, +0xc, +0x10), enforces a minimum of 0x20 slots, allocates a block of (n*3-3)*8+0x28 bytes through PTR_FUN_01b9378c (contain.inl… |
| `00efe9d0` | ContainerPoolFreeAll | likely | Walks the block list at pool+0x10 (next pointer at +0 of each block) and deallocates each block through PTR_FUN_01b93790 (contain.inl line 0x395). Then clears … |
| `00f0f400` | TreeRemoveMinA | likely | Thiscall with EAX = tree header (+0 free list, +4 minimum, +8 maximum, +0xc root). Walks from the root down the leftmost path into a 32-entry local stack, spli… |
| `00f0f4a0` | TreeRemoveMaxA | likely | Thiscall with EAX = tree header. Walks from the root down the rightmost path into a 32-entry local stack, splices the rightmost node out (updating the maximum … |
| `00f0f890` | TreeInsertNodeA | likely | Inserts node param_2 into the ordered tree at param_1 (+4 minimum, +8 maximum, +0xc root): descends comparing *param_2 with each node's first dword, records th… |
| `00f0fdd0` | KeyTreeErase | likely | Walks the tree at param_1 (root +0xc) for key param_2. When the node is found it calls 00f0f400 if it is the minimum (+4), 00f0f540 if it is neither the minimu… |
| `00f0fea0` | KeyTreeLookupValue | likely | Searches the tree at param_1 (root +0xc) for key param_2. When found it stores the node's value (+4) into *param_3 and returns 1 (flag in AL); when absent it s… |
| `00f0fef0` | KeyTreeContains | likely | Searches the tree at param_1 (root +0xc) for key param_2 and returns 1 when a node with that key exists, else 0 (flag in AL, no value stored). |
| `00f10600` | PairTreeInsertFixupLeft | likely | Rebalances the pair-key tree after an insert went into the left subtree of the node whose link is at *EDI. Balance tag = low 2 bits of node+0x18 (0 left-heavy,… |
| `00f10700` | PairTreeInsertFixupRight | likely | Mirror of 00f10600 for an insert into the right subtree of the node whose link is at *EDI (right child at node+0x18, tag in its low 2 bits, left child at node+… |
| `00f107f0` | PairTreeInsertNode | likely | Inserts node param_2 into the pair-key ordered tree at param_1 (+4 min, +8 max, +0xc root). Descends comparing the first dword of the key, then the second when… |
| `00f10900` | PairTreeInsertReturnNode | likely | Cdecl wrapper: calls 00f107f0(param_1 header, param_2 node) with cdecl cleanup (add esp,8), then returns param_2 in EAX. |
| `00f10ac0` | PairTreeFind | likely | Looks up the pair key (param_2, param_3) in the ordered tree at param_1 (root +0xc). At each node it compares the first dword with node+0; when equal it compar… |
| `00f1c2f0` | TreeInsertFixupLeftOff8 | likely | Rebalances the string-keyed tree after an insert into the left subtree of the node whose link is at *EDI (node = link with tag bits masked). Balance tag = low … |
| `00f1c3f0` | TreeInsertFixupRightOff8 | likely | Mirror of 00f1c2f0 for an insert into the right subtree (right child at node+0x8 with tag in its low bits, left child at node+0x4). Left-heavy node: sets balan… |
| `00f1c4e0` | StrKeyTreeInsertNode | likely | Inserts node in_EBX into the string-keyed AVL tree at param_1 (+4 min, +8 max, +0xc root). Node key = string pointer at node+0; descends comparing with StrComp… |
| `00f1c840` | StrKeyTreeFind | likely | Looks up the string key param_2 in the tree at param_1 (root at +0xc). Compares the key with each node's string (00f1b1b0); smaller keys go to +0x4, larger key… |
| `00f1f330` | TreeInsertFixupLeftOff28 | likely | Rebalances the tagged tree after an insert into the left subtree of the node whose link is at *EDI. Balance tag = low 2 bits of node+0x28 (0 left-heavy, 1 bala… |
| `00f1f430` | TreeInsertFixupRightOff28 | likely | Mirror of 00f1f330 for an insert into the right subtree: right child at node+0x28 (tag in its low bits), left child at node+0x24. Left-heavy sets balanced and … |
| `00f1f520` | TaggedTreeInsertNode | likely | Inserts node in_EBX into the tagged tree at param_1 (+4 min, +8 max, +0xc root). Node layout: string key at +0, left child at +0x24, right child with balance t… |
| `00f1f750` | TaggedTreeNextOrBoundOff28 | likely | Finds the adjacent node for param_2 in the tagged tree at param_1 (fallback +0x8, root +0xc) using the layout of 00f1f520 (left +0x24, right/tag +0x28). If par… |
| `00f204a0` | TaggedTreeNextOrBoundOff14 | likely | Same adjacent-node search as 00f1ea10 and 00f1f750 for the tagged tree layout with left child at +0x10, right child with tag at +0x14. The key comparison is 00… |
| `00ed2dc0` | StringKeyTableFind | speculative | Cdecl-style, two stack arguments after the record pointer (key, out index pointer), 251 bytes. If the record is null it sets the out index to -1 and returns 0.… |
| `00ed56b0` | StringKeyTableFind98 | speculative | Cdecl-style, two stack arguments after the record (key, out index pointer), 82 bytes. Sets the out index to 0 and, when the record is nonnull and its count at … |
| `00ef2b00` | DwordRangeShiftUpByOne | speculative | Increments the count stored at [EAX] and shifts the 4-byte elements of the array at EDI from index ESI up by one slot (the top element moves to the new last sl… |
| `00ef5770` | FixedPoolUnlinkAndFreeNode | speculative | Clears the low three flag bits of +0x40, unlinks the node from its list (*(+0x90)+0x94 = +0x94 and *(+0x94)+0x90 = +0x90), drains the node's child list through… |
| `00ef5920` | FlaggedSelfLinkedNodeRelease | speculative | Returns 1 and calls 00ef5770 when the node argument is non-null, has bit 2 of +0x40 set and its +8 link points to itself; otherwise returns 0 in AL. |
| `00efe460` | RbTreeFixupStepA | speculative | Operates on the node whose pointer is in EDI, reading the low two bits of its parent/color word and of its sibling and grandparent words, and rewrites the colo… |
| `00f0c0b0` | PairKeyMapInsert | speculative | Orders the two keys (param_3, param_4) so that min <= max, hashes them to bucket (max*0x1d4b + min) & 0x3ffff in the table at this+0xd50, and walks that bucket… |
| `00f0c200` | PairKeyMapNextValue | speculative | Thiscall with EAX = first key and stack param_3 = second key (sorted to min/max), hashes to the same bucket as 00f0c0b0 and finds the node for that pair; walks… |
| `00f0c2c0` | IndexTripleEdgeLinks | speculative | For param_1 index triples at param_2 (3 dwords each): resolves each index through the remap chain at this+0x428/+0x44c (00f18210), inserts its three edges with… |
| `00f0ebc0` | RecordArrayInit | speculative | Stores param_2 as the record count at +0, zeroes the count at +4, sets the cursor +8 to the base at +0x14, clears param_2*0xc bytes at that base, and calls 00f… |
| `00f0ed30` | RecordTableFinish | speculative | Calls 00f33ca0 on the sub-buffer at +0x18, stores the record base (+0x14) into *param_2, reads the element count from +0 and calls 00f34130 on the sub-buffer; … |
| `00f10260` | TreeFixupStepF | speculative | Mirror-style rebalancing step on the second ordered-tree layout (children at +0x18/+0x1c); returns 1 or 0 by the same flag convention as 00f10110. |
| `00f1ea10` | TaggedTreeNextOrBoundOff18 | speculative | Finds the adjacent node for param_2 in the tree at param_1 (fallback +0x8, root +0xc). If param_2 has a non-zero tag-masked link at +0x18 it walks the +0x14 ch… |
| `00ed2ed0` | — | unknown | Cdecl-style, three stack arguments (record, key string, out index pointer), 142 bytes. When the record is nonnull and its count at +0xc is positive it sets the… |
| `00eee260` | — | unknown | Initialises a 0x20-byte record from three stack arguments: dword 0 = arg1, dword 1 = arg3, dwords 2 and 3 = &DAT_01781ca4 (the same sentinel address), dwords 4… |
| `00efe560` | — | unknown | Second rebalance step on the node in EDI: reads the color bits of the node, its parent, and the uncle node; recolours, rotates, and returns 1 for the case that… |
| `00f0f180` | — | unknown | Operates on the tree node at the pointer held in ESI (node pointer with color/flag bits in the low two bits of +0xc, children at +8 and the parent-side link at… |
| `00f0f2d0` | — | unknown | Operates on the tree node at the pointer held in ESI with the same flag layout as 00f0f180, performing the mirrored rotation-and-recolor step and returning 1 t… |
| `00f0f540` | — | unknown | Thiscall with ECX = tree header and stack param_2 = node to erase. Finds the node by its key (*param_1), replaces it with its successor when it has two childre… |
| `00f0f6a0` | — | unknown | Mirror-side variant of the rebalancing step on the node at the pointer held in EDI, using the sibling through +8 and the same color bits in +0xc; returns 0 aft… |
| `00f0f7a0` | — | unknown | Second mirror-side rebalancing step on the node at the pointer held in EDI; returns 1 or 0 with the same flag conventions and performs a rotation through the s… |
| `00f10110` | — | unknown | Rebalancing step on the node at the pointer held in ESI for the second ordered-tree layout (children at +0x18/+0x1c, color bits in the low two bits of +0x18); … |
| `00f10390` | — | unknown | Thiscall with EAX = tree header of the second layout. Walks the leftmost path from the root into a 32-entry local stack, splices the leftmost node out, links i… |
| `00f10420` | — | unknown | Thiscall with EAX = tree header of the second layout. Walks the rightmost path from the root into a 32-entry local stack, splices the rightmost node out, links… |
| `00f104b0` | — | unknown | Fastcall with the tree header in EDX (its root at +0xc) and the key pair at param_2 (first dword compared, then second when the first is equal); finds the node… |
| `00f1ccc0` | — | unknown | Returns the stored copy of the string param_2 from the string table param_1. If a node with that key exists (00f1c840) it returns the node's stored pointer. Ot… |
| `00f20de0` | — | unknown | Finishes a track group builder (param_1) into a new 0x84-byte group allocated from the arena (00f0b200 with context param_2). It copies the builder's counts an… |

### `lib.granny.controlled_animation`

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ef6830` | ControlledAnimationSampleMirrorCache | likely | When param_3 > 0.001 (DAT_017ebf0c): if the cache at param_4+0x50 is null it reports 'Instance.MirrorTransformCache' through 00f07e00 and returns (void). Other… |
| `00ef6b40` | ControlledAnimationEvaluateDispatch | likely | Returns unless the weight from 00ef61a0 is above 0.001. Computes a time fraction from 00ef6460 and the record at *param_1 (+4, +8 fields, rounded to an integer… |
| `00ef7440` | ClipTimeAdvanceAndAccumulate | likely | Proceeds only when param_1[8] is nonzero, the 00ef61a0 weight is above 0.001, and the track data at (*param_1[7])+0x28 has a nonzero count at +4. Reads 00ef619… |
| `00ef7fd0` | AccumulateBoneRangeWeighted | likely | Returns unless the 00ef61a0 weight is above 0.001. Takes the element count from 00ed6b70 applied to the dword at param_1[7], clips [param_2, param_2+param_3) t… |
| `00ef5fc0` | RampWeightEvalCubicEdges | speculative | Expands two packed values at +0x70 and +0x7c with 00ef5e40 into four floats each, then returns a weight: a cubic (00ef5eb0) on the segment selected by the time… |
| `00ef6150` | FlagBit0AndNonzeroFloat64 | speculative | Returns 1 when bit 0 of the byte at +0x40 is set and the float at +0x64 is nonzero, otherwise 0. |
| `00ef61a0` | WeightedRampValue | speculative | Returns 0 when bit 0 of +0x40 is clear. Otherwise it returns 00ef5fc0(param_1) times the float at +0x64. |
| `00ef6f70` | AccumulateTrackDeltaVectors | speculative | Mode param_1[1]==1: takes the track record at (*param_1)->0x14 table entry +0x78 (or the +0x6c/+0x70/+0x74 fields when that is null), scales its two 3-vectors … |

### `lib.granny.curve`

**(no class)** (121)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed2120` | CurveApplyQuatTransform | confirmed | Cdecl-style, six stack arguments (two matrix or quaternion pointers, a float at arg5, and the curve record at arg6), 712 bytes. Returns after logging through 0… |
| `00ee2cd0` | GrannyCurve2_CreateByKey | confirmed | Looks up the format index of key param_1 with 00ee2a90. If the index is negative, logs 'Invalid format specified for a granny_curve2 - creating this curve will… |
| `00ee2d20` | GrannyCurve2_Init | confirmed | If param_1 is nonnull: byte 0 = 0; dword +4 = param_2 (format key); dword +8 = 00ee2a90(param_2), logging the 'Invalid format specified for a granny_curve2 - c… |
| `00ee2e00` | GrannyCurve2_Alloc | confirmed | Opens an allocation context (00f24cd0 with granny_curve.cpp line 0x35a, then 00f24d40), allocates the 0x34-byte granny_curve2 record with 00f24ef0 (size 0x34),… |
| `00ee2ff0` | GrannyCurve2_AggregateSize | confirmed | If param_1 is nonnull: when the format index (dword +8) is negative, logs 'Invalid format specified for a granny_curve2 - returning size of 0' (line 0x3d8) and… |
| `00ee30f0` | GrannyCurve2_AggregateSizeNoPrefix | confirmed | Same error gate as 00ee2ff0 (negative index logs 'returning size of 0' at line 0x3f6 and returns 0). Opens a size context (00f24cd0, line 0x3fb); when the inde… |
| `00ee35b0` | GrannyCurve2_AllocCopy | confirmed | If param_1 is nonnull: when its format index (+8) is negative, logs 'Invalid format specified for a granny_curve2 - returning NULL' (line 0x4eb), releases para… |
| `00ee3880` | GrannyCurve2_CheckedApply | confirmed | Validates the record against the arguments: the dimension check runs first (component count from table 0x01b92b88[format], taken only when the element count fr… |
| `00ee2a90` | CurveFormatIndexLookup | likely | Loops over the 19-entry table at 0x01b92b80 (entries 0x34 bytes apart, loop bound 0x1b92f5c), calling FUN_00eea900(first dword of the entry, param_1) on each; … |
| `00ee2b70` | CurveIsIdentity | likely | Returns 1 when the curve record's format byte is 2, or is 1 with dword +4 equal to 0 and dword +0xc equal to 0; otherwise returns 0. |
| `00ee2c50` | CurveFormat_ComponentCount | likely | Switch on the format byte (cases 0..0x12). Returns the per-format component count (dimension): case 0 = short(+2); case 1 = (+0xc)/(+4); case 2 = short(+2); ca… |
| `00ee2d90` | GrannyCurve2_InitFromSource | likely | If param_1 is nonnull: element count = table-A handler for the source record's format (format byte at *(param_2[1])); component count = table-B handler when th… |
| `00ee3560` | GrannyCurve2_BuildInto | likely | If the builder argument (loaded into ESI) is null returns 0. Otherwise calls 00ee31d0 with out = param_2, copy flag 0, and the buffer forwarded in EAX from the… |
| `00ee3680` | GrannyCurve2_Release | likely | Calls the library hook 01b93790 with ('granny_curve.cpp', line 0x515, param_1). Returns nothing. |
| `00ee3730` | CurveFormat1_AllocBlocks | likely | Opens an allocation context (00f24cd0, granny_curve.cpp line 0x5b8), reserves a 0x14-byte header (00f24ef0), adds two 4-byte-element arrays: count param_1 at h… |
| `00ee39e0` | GrannyCurve2_SourceAggregateSize | likely | Builds a 52-byte temporary granny_curve2 header from the source (param_1) with 00ee2d90, computes its aggregate size with 00ee2ff0, releases the header through… |
| `00ee3f20` | CurveFormat0_ComponentCount | likely | Returns the signed short at record+2 as an int (the component count of a format-0 record). |
| `00ee3f30` | CurveFormat0_ElementCount | likely | Returns dword(record+4) / short(record+2) as a signed int: the element count of a format-0 record. |
| `00ee3f90` | CurveFormat0_CopySamples | likely | If param_5 is nonnull, copies param_3 elements of dim (short +2) floats each into param_5, starting at element param_2 of the record's data at +8; one memcpy o… |
| `00ee4070` | CurveFormat0_ApplyScalePerm | likely | Format-0 case body of 00ee3960: per block of dim floats copies one block into a 16-float stack buffer (memcpy of dim*4 bytes) and writes out[i] = buf[perm[i]] … |
| `00ee4450` | CurveFormat3_ApplyScalePerm | likely | Format-3 case body of 00ee3960: copies count (+4) floats from the data at +8 into a 16-float stack buffer, then writes the scaled, offset and permuted results … |
| `00ee46b0` | CurveFormat4_ApplyScalePerm3 | likely | Format-4 case body of 00ee3960: saves the three floats at record+4, +8 and +0xc into a local buffer (in that order), then writes local[perm[i]] * scale[i] + of… |
| `00ee4830` | CurveFormat5_ApplyScalePerm4 | likely | Format-5 case body of 00ee3960: saves four floats at record+4..+0x10 into a local buffer, then writes local[perm[i]] * scale[i] + offset[i] back to those four … |
| `00ee4f80` | CurveBuildU16Matrix | likely | Writes a 16-bit curve block from a float matrix: param_2 key times from param_4 and param_2 rows of param_3 columns from param_5. Stores the high 16 bits of fl… |
| `00ee5640` | CurveKeyLookup_U8_Bf16_c0c_d10 | likely | Computes key = floor(bf16(+2) * t) with t = param_2, where bf16(+2) = float((ushort +2) << 16), and calls 00f05d80(count, +0x10, low byte of key), count = (+0x… |
| `00ee56c0` | CurveKeyLookup_U8_Bf16_c0c_d10_ExtraArg | likely | Same as 00ee5640 (count (+0xc)/((+4)/2+1), base +0x10, key floor(bf16(+2)*t)) but calls 00f05e40 (8-bit upper_bound, clamped to count-1) and forwards its third… |
| `00ee5970` | CurveBuildU8Matrix | likely | 8-bit twin of 00ee4f80. Stores the high 16 bits of float 255/param_4[count-1] at record +2 (count = (+0xc)/((+4)/2+1)), writes ROUND(param_4[i]*scale) as bytes… |
| `00ee6080` | CurveKeyLookup_U16_F32_c08_d0c_ExtraArg | likely | Same as 00ee6020 (count (+8)>>2, base +0xc, s = float(+4)*t, key floor(s)) but calls 00f05e00 (16-bit upper_bound, clamped to count-1) and forwards the third a… |
| `00ee6100` | CurveBuildQuatSmallest3U16 | likely | Smallest-three quaternion encoder for param_5 (4 floats per key, stride 16 bytes) into a 16-bit layout. Per key it normalises the quaternion (identity when the… |
| `00ee6850` | CurveKeyLookup_U8_F32_c08_d0c | likely | Computes s = float(+4) * t, key = floor(s), and calls 00f05d80(count, +0xc, low byte of key, ...), count = (+8)>>2. 00f05d80 is an 8-bit upper_bound clamped to… |
| `00ee68b0` | CurveKeyLookup_U8_F32_c08_d0c_ExtraArg | likely | Same as 00ee6850 but calls 00f05e40 (8-bit upper_bound clamped to count-1) and forwards its third argument, which the callee ignores. Returns the callee EAX un… |
| `00ee6930` | CurveBuildQuatSmallest3U8 | likely | 8-bit twin of 00ee6100: smallest-three quaternion encoder writing one byte per index and three bytes per key. Keys are normalised (identity when \|q\| <= DAT_0… |
| `00ee70b0` | CurveKeyLookup_U16_Bf16_c1c_d20_ExtraArg | likely | Same as 00ee7040 (count (+0x1c)>>2, base +0x20, key floor(bf16(+2)*t)) but calls 00f05e00 (16-bit upper_bound clamped to count-1) and forwards the third argume… |
| `00ee7270` | CurveBuildVec3U16 | likely | 16-bit builder for three-component keys (param_5 stride 3 floats). Key times from param_4 are written as uint16 at +0x20 with count = (+0x1c)>>2, and the scale… |
| `00ee76d0` | CurveKeyLookup_U8_Bf16_c1c_d20_ExtraArg | likely | Same as 00ee7660 but calls 00f05e40 (8-bit upper_bound clamped to count-1) and forwards the third argument, which the callee ignores. Returns the callee EAX un… |
| `00ee7890` | CurveBuildVec3U8 | likely | 8-bit twin of 00ee7270 for three-component keys: key times from param_4 as bytes at +0x20 with count = (+0x1c)>>2, bf16 scale 255/param_4[count-1] at +2, per-c… |
| `00ee7e80` | CurveBuildScalarStride9U16 | likely | 16-bit builder for single-channel keys with a 9-float stride (param_5 advances by nine floats per key). Key times from param_4 are written as uint16 at +0x10 w… |
| `00ee81d0` | CurveKeyLookup_U16_Bf16_c1c4_d20_ExtraArg | likely | Same as 00ee8160 (count trunc((+0x1c)/4), base +0x20, key floor(bf16(+2)*t)) but calls 00f05e00 (16-bit upper_bound clamped to count-1) and forwards the third … |
| `00ee89a0` | CurveBuildScalarStride9U8 | likely | 8-bit twin of 00ee7e80: key times from param_4 as bytes at +0x10 with count = (+0xc)/2 and bf16 scale 255/param_4[count-1] at +2; min and max over element 0 of… |
| `00ee8ce0` | CurveKeyLookup_U8_Bf16_c1c4_d20_ExtraArg | likely | Same as 00ee8c70 but calls 00f05e40 (8-bit upper_bound clamped to count-1) and forwards the third argument, which the callee ignores. Returns the callee EAX un… |
| `00eea3f0` | CurveRecordCreateAndDecode | likely | Takes a curve builder (param_1; +4 points to a format record whose byte 0 is the format and byte 1 the subformat). Calls the per-format 13-dword row tables at … |
| `00ef2120` | BSplineEvalDispatchByDegree | likely | Checks param_1 (degree) is 0..3, else reports through 00f07e00 (line 0x2b6, Degree out of range). For sample count param_2 below 10 it looks up a specialised e… |
| `00ef2c20` | BSplineSolverErrorScan | likely | Requires a non-null result block param_9 (else reports through 00f07e00, line 0xc9). For sample indices 1 to param_6-1 across the interval param_4 it evaluates… |
| `00ef2d40` | BSplineKnotRefineMidpoints | likely | Reads the knot count *param_2 and the knot array at EAX. It copies knots to the output at param_3 in order and, where the fit error at the interval from 00ef2b… |
| `00ef36c0` | CurveKnotDecimateToTolerance | likely | Loops while the lower bound local_7c is below the upper bound. Each pass builds a candidate curve from the sample block (00ee2cc0, 00ee2d20 GrannyCurve2_Init, … |
| `00ef3b70` | CurveFitBSplineSolve | likely | Allocates a work buffer through the allocation hook (PTR_FUN_01b9378c = 00f0ae00) with granny_bspline_solver.cpp line 0x309, creates the candidate GrannyCurve2… |
| `00ed2940` | CurveSampleErrorMax | speculative | Cdecl-style, seven stack arguments (row width, row count, sample array, reference array or null, metric selector, output a, output b), 584 bytes. Per row of th… |
| `00ed2b90` | CurveSampleErrorTest | speculative | Cdecl-style, ten stack arguments. Zeroes two floats and calls 00ed2940 (CurveSampleErrorMax) on the sample data, passing its two error outputs. When the first … |
| `00ed2d00` | AnyOf3CurvesZeroOrPosConst | speculative | Cdecl-style, one stack argument (a record pointer), 64 bytes. Calls 00ee2b40 on the record at +0x10, then on +8 and +0x18 when the earlier calls return 0. Retu… |
| `00ed2d40` | NotAllOf3CurvesFmt2_3to5_1 | speculative | Cdecl-style, one stack argument (a record pointer), 64 bytes. Calls 00ee2ba0 on the record at +0x10, +8 and +0x18, in that order, stopping at the first call th… |
| `00ed2f60` | GetU16AndS16Fields | speculative | Cdecl-style, three stack arguments (record, out unsigned pointer, out signed pointer), 25 bytes. Stores the unsigned 16-bit field at record+0x68 into the first… |
| `00ee2ae0` | CurveFormatIndexStore | speculative | Calls 00ee2a90 with the dword at *param_1 (the format key) and stores the returned byte (format index or -1) at the address held in param_1[1] (single byte wri… |
| `00ee2b40` | CurveFormatTest_ZeroOrPosConst | speculative | Returns 1 when the curve record's format byte (record = dword at param_1+4) is 0, or is 1 with dword +4 equal to 0 and signed dword +0xc greater than 0; otherw… |
| `00ee2ba0` | CurveFormatTest_Fmt2_3to5_Fmt1 | speculative | Returns 1 for format 2 and for formats 3..5. For format 1 returns 1 when dword +4 == 0 and dword +0xc == 0, or when dword +4 == 1; returns 0 for format 0, for … |
| `00ee2be0` | CurveFormatTest_Fmt3to5_Fmt1Eq1 | speculative | Returns 1 for formats 3, 4 and 5, and for format 1 when dword +4 == 1 (any +0xc). Returns 0 for format 0, format 2, format 1 with +4 == 0, and formats 6 and up. |
| `00ee2c30` | CurveFormat_ElementCount | speculative | Switch on the format byte (cases 0..0x12, no default). Returns the per-format element count from curve fields: case 0 = (+4)/short(+2); case 1 = +4; case 2 = 0… |
| `00ee2c70` | CurveFormat_ElementGateComponentCall | speculative | Calls the table-A handler for the curve's format (table 0x01b92b8c, indexed by format byte times 0x34) with the curve. If it returns nonzero, calls the table-B… |
| `00ee2cb0` | CurveDataByte1 | speculative | Returns the unsigned byte at offset 1 of the record pointed to by dword +4 of param_1 (movzx of [record+1]). |
| `00ee2cc0` | ReturnDword0 | speculative | Returns the dword at offset 0 of param_1 (mov eax,[param_1]; ret). |
| `00ee2f10` | CurveSetDword18ClearDword20 | speculative | If param_1 is nonnull: sets dword +0x20 to 0 and dword +0x18 to param_2. |
| `00ee2f30` | CurveSetDwords24to30 | speculative | If param_1 is nonnull: stores param_2, param_3, param_4 and param_5 into dwords +0x24, +0x28, +0x2c and +0x30. |
| `00ee2f60` | CurveSetDword1cClearDword20 | speculative | If param_1 is nonnull: sets dword +0x20 to 0 and dword +0x1c to param_2. |
| `00ee36b0` | CurveIfFormat1Record | speculative | Returns the record pointer (dword +4 of param_1) when the record's format byte is 1, otherwise 0; implemented branch-free with a mask from the byte != 1 test. |
| `00ee3960` | CurveFormat_ApplyScalePerm | speculative | Switch on the format byte. Case 0: per block of dim floats (dim = short +2, count = (+4)/dim, data at +8) copies one block into a 16-float stack buffer and wri… |
| `00ee3980` | CurveFormat_DispatchWithFloat | speculative | Calls table 0x01b92b90[format] with (param_1, param_2), where param_2 is a float argument (pushed as a 4-byte float). No return value. |
| `00ee4000` | CurveAggregateSetup12_ArrayCountMul | speculative | Opens a 12-byte aggregate header through 00f24ef0 (param_3 slot), then adds a 4-byte-element array of count (param_2+0x14) * (param_2+0x10) at header offset 8 … |
| `00ee43f0` | CurveAggregateSetup12_ArrayCount | speculative | Opens a 12-byte aggregate header through 00f24ef0 (size 0xc, param_3 slot), then adds a 4-byte-element array of count *(param_2+0x10) at header offset 8 throug… |
| `00ee65b0` | AggregateLayout_Slot10_U16x4_Off08 | speculative | Calls 00f24ef0(ctx, param_3, 0x10) with the return discarded, then 00f24f80(ctx, *param_3, 8, 0xc, 4*count, 2) where count = *(builder+0x14): adds a uint16 arr… |
| `00ee6630` | QuatSmallest3U16_ReorderComponents | speculative | Case body of switchD_00ee3977 (selector not shown). Rebuilds the four-nibble header at +2 by moving each nibble to the position given by the permutation in par… |
| `00ee6dc0` | AggregateLayout_Slot10_U8x4_Off08 | speculative | Calls 00f24ef0(ctx, param_3, 0x10) with the return discarded, then 00f24f80(ctx, *param_3, 8, 0xc, 4*count, 1) where count = *(builder+0x14): adds a byte array… |
| `00ee74a0` | AggregateLayout_Slot24_U16x4_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 4*count, 2) where count = *(builder+0x14): adds a uint16… |
| `00ee7ab0` | AggregateLayout_Slot24_U8x4_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 4*count, 1) where count = *(builder+0x14): adds a byte a… |
| `00ee8080` | AggregateLayout_Slot14_U16x2_Off0c | speculative | Calls 00f24ef0(ctx, param_3, 0x14) with the return discarded, then 00f24f80(ctx, *param_3, 0xc, 0x10, 2*count, 2) where count = *(builder+0x14): adds a uint16 … |
| `00ee85c0` | AggregateLayout_Slot24_U16x4_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 4*count, 2) where count = *(builder+0x14): adds a uint16… |
| `00ee8b90` | AggregateLayout_Slot14_U8x2_Off0c | speculative | Calls 00f24ef0(ctx, param_3, 0x14) with the return discarded, then 00f24f80(ctx, *param_3, 0xc, 0x10, 2*count, 1) where count = *(builder+0x14): adds a byte ar… |
| `00ee90e0` | AggregateLayout_Slot24_U8x4_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 4*count, 1) where count = *(builder+0x14): adds a byte a… |
| `00ee9450` | CurveBuildExtentParam3 | speculative | Copies the key-time array (param_4) into the +0x20 block (four keys per unrolled step, then a tail), calls 00ee3a30(count, param_5, +0x10, +4, 1) which, with i… |
| `00ee9680` | AggregateLayout_Slot24_F32x2_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 2*count, 4) where count = *(builder+0x14): adds a 4-byte… |
| `00ee9710` | Box3PermuteScaleOffset | speculative | Reads six floats of record +4..+0x18 (a min triple at +4/+8/+0xc and a max triple at +0x10/+0x14/+0x18) into a local copy, then writes them back permuted by th… |
| `00ee9c70` | AggregateLayout_Slot24_U16x2_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 2*count, 2) where count = *(builder+0x14): adds a uint16… |
| `00ee9d10` | Box3PermuteScaleOffset | speculative | Same body as 00ee9710 (case 11 of switchD_00ee3977): six-float min/max triple at record +4..+0x18 is permuted by param_5, scaled by param_3, and the max triple… |
| `00eea260` | AggregateLayout_Slot24_U8x2_Off1c | speculative | Calls 00f24ef0(ctx, param_3, 0x24) with the return discarded, then 00f24f80(ctx, *param_3, 0x1c, 0x20, 2*count, 1) where count = *(builder+0x14): adds a byte a… |
| `00ef09d0` | BasisDegree2Knots | speculative | Computes three quadratic basis weights from five knot-like values p1..p5 and the parameter p5 by ratios (p5-p2)/(p3-p2), (p5-p1)/(p3-p1) and (p5-p2)/(p4-p2); w… |
| `00ef0a20` | BasisDegree3Knots | speculative | Computes four cubic basis weights from seven values (p1..p7 with the parameter p7) using ratios of differences against p1..p6 and writes the four results to pa… |
| `00ef0af0` | BasisRecursiveAccumulate | speculative | Recursive helper. When param_2 is zero it adds param_6 into *param_5. Otherwise it computes the ratio (param_4 - knot)/(knot' - knot) against the knot at param… |
| `00ef0b80` | BasisValuesByDegree | speculative | Dispatches on the degree in param_1 (switch: 0..3 through a jump table at 00ef0c7c, verified; default for higher values). Degree 0 writes 1.0 to param_4; degre… |
| `00ef21f0` | BSplineTableEvalFlag0 | speculative | Calls the function pointer stored at DAT_01b93570 + (param_2 + param_1*10)*4 with the four stack arguments, with no null check. |
| `00ef2230` | BSplineTableEvalFlag1 | speculative | Calls the function pointer stored at DAT_01b93610 + (param_2 + param_1*10)*4 with the four stack arguments, with no null check. |
| `00ef2710` | CurveEvalSamplesByFormat | speculative | Evaluates the curve record param_4 into param_9 for param_1 samples. Identity curves (00ee2b70) copy param_1 dwords from param_10. Curves tested by 00ee2ba0 sa… |
| `00ef3120` | UniformRampFill | speculative | Fastcall with EAX = span and ECX = count, output pointer EDX. Stores ECX floats: element k = k * (EAX - 1) / (ECX - 1), computed on the x87 stack; does nothing… |
| `00ef3160` | RowNeighbourSquaredDeltaSums | speculative | Takes the row index from EAX and the row length from param_1. It accumulates, over the columns in groups of four with a scalar tail, two sums of squared 4-comp… |
| `00ef3260` | RowSecondDifferenceSumSq | speculative | Returns, as x87 float10 in ST0, the sum over the columns of squared second differences (2*row - previous - next)^2 for the row selected by EBX, with row length… |
| `00ef3340` | KnotPlacementPass | speculative | Loops over rows 1 to param_4-2. With flag bit 4 it first writes a ramp (index * DAT_017f17a4 = 0.125 steps) through 00ef2b00 and clears the slot. For each row … |
| `00ef5eb0` | CubicBernsteinEvalClampedParam | speculative | Computes t = (param_3 - param_2)/(param_4 - param_2), clamped into [0,1] with the degenerate-interval case handled, and returns the cubic Bernstein value of th… |
| `00f21dd0` | EvalCurveComponentsOrDefault | speculative | Calls 00ee2b70 (identity test). If the curve is identity it copies p1 dwords (p1*4 bytes) from p3 to p2 and returns. Else if 00ee2ba0 is true it calls 00ee3820… |
| `00ed1f10` | — | unknown | Cdecl-style, five stack arguments (matrix, second matrix, tolerance/extra float, dimension-3 curve selector and the curve record), 480 bytes. If 00ee2b70 (iden… |
| `00ed23f0` | — | unknown | Cdecl-style, six stack arguments (two matrix pointers, a float at arg5 and the curve record at arg6), 741 bytes. Logs through 00f07e00 (level 3, code 0x38) and… |
| `00ed26f0` | — | unknown | Thiscall-style helper with one record pointer and four stack arguments, 592 bytes. For each of the count entries (dword at record+0xc) of 0x20 bytes starting a… |
| `00ed2c5d` | — | unknown | Undefined-return helper (returns AL), seven stack arguments. Gets the element count (00ee2c30) and the component count (00ee2c70) of the curve format, allocate… |
| `00ee2ea0` | — | unknown | Allocating form of 00ee2d90: element count = table-A handler for the source's format; component count = table-B handler when that count is nonzero; calls 00ee2… |
| `00ee31d0` | — | unknown | Builds a granny_curve2 record from a curve_builder held in ESI (key at +4, format index at +8, subformat byte at +0xc, field groups at +0x10..+0x30) into the o… |
| `00ee36d0` | — | unknown | Stores the vtable DAT_01b91fc0 into param_1[0] and the record pointer param_2 into param_1[1]. In the record writes byte 0 = 1 (format 1), byte 1 = param_4, dw… |
| `00ee3820` | — | unknown | Calls the sample table 0x01b92b98[format] of the record with (record, index param_2, count 1, &local, param_3, param_4), where local is a float starting at 0.0… |
| `00ee3860` | — | unknown | Switch on the format byte (cases 0..0x12, with a default tail). Writes param_3 elements starting at element param_2 into param_4 (values) and param_5 (second o… |
| `00ee4c60` | — | unknown | Calls 00f05d40 with count = (+0xc) / ((+4)/2 + 1), the pointer at +0x10, and an int equal to (int)ROUND(float(short(+2) << 16) * param_2) with param_2 a float. |
| `00ee4ce0` | — | unknown | Same as 00ee4c60 but calls 00f05e00 and forwards param_3 as a fourth argument: count = (+0xc)/((+4)/2+1), pointer at +0x10, int = (int)ROUND(float(short(+2) <<… |
| `00ee6020` | — | unknown | Computes s = float(+4) * t, key = floor(s), and calls 00f05d40(count, +0xc, key, s, control word RC=01), count = (+8)>>2. 00f05d40 is a 16-bit upper_bound retu… |
| `00ee7040` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05d40(count, +0x20, key, scale, control word RC=01), count = (+0x1c)>>2. 00f05d40 is a 16-bit upper_bound clam… |
| `00ee7660` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05d80(count, +0x20, low byte of key, scale, control word RC=01), count = (+0x1c)>>2. 00f05d80 is an 8-bit uppe… |
| `00ee7cd0` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05e00(count, +0x10, key, third argument, scale, control word RC=01), count = (+0xc)/2. 00f05e00 is a 16-bit up… |
| `00ee8160` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05d40(count, +0x20, key, scale, control word RC=01) with count = trunc((+0x1c)/4) (signed divide: adds 3 for n… |
| `00ee87f0` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05e40(count, +0x10, low byte of key, third argument, scale, control word RC=01), count = (+0xc)/2. 00f05e40 is… |
| `00ee8c70` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05d80(count, +0x20, low byte of key, scale, control word RC=01) with count = trunc((+0x1c)/4). 00f05d80 is an … |
| `00ee9840` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05e00(count, +0x20, key, third argument, scale, control word RC=01) with count = (+0x1c)/2. 00f05e00 is a 16-b… |
| `00ee9e40` | — | unknown | Computes key = floor(bf16(+2) * t), and calls 00f05e40(count, +0x20, low byte of key, third argument, scale, control word RC=01) with count = (+0x1c)/2. 00f05e… |
| `00ef2270` | — | unknown | Fills a window of 2*degree samples around index param_8 of the curve param_3 (degree byte at curve+4 +1; element count from 00ee2c30) into param_9 and the vect… |
| `00ef2b70` | — | unknown | Samples the curve (00ef2710) into a 16-float buffer, then returns as x87 float10 either (1 - dot)^2, where dot is the sum of products of the samples with a ref… |
| `00ef3490` | — | unknown | Zeroes the work block at *(param_1+4) with _memset over (param_4+param_3)*4+8 bytes. It writes a descending ramp of -(k+1) into the parameter array at param_5 … |
| `00ef5000` | — | unknown | Selects curve pointer +0x20 or +0x1c of param_1 (the choice depends on flag EBX and the sign of the float at +8), creates a GrannyCurve2 from it with 00ee2e00 … |
| `00f20980` | — | unknown | If the curve in EAX is identity (00ee2b70) it copies its first two dwords to the destination in EBX and returns the running total in EDI unchanged. Otherwise i… |
| `00f211c0` | — | unknown | Returns an 8-byte pointer pair for a curve. If the curve is identity (00ee2b70) the pair is a static template: low dword &DAT_01b92460, high dword DAT_01b939b8… |

### `lib.granny.dag`

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ef8390` | DagLeafAnimationBlendSetModelInstance | confirmed | Asserts Node->Type == DagNodeType_Leaf_AnimationBlend (node+0x18 must be 0, granny_dag.cpp line 0xca) and ModelInstance (param_2) non-null (line 0xcb). If the … |
| `00ef8450` | DagLeafLocalPoseSetPose | confirmed | Asserts Node->Type == DagNodeType_Leaf_LocalPose (node+0x18 must be 1, granny_dag.cpp line 0xe8) and LocalPose (param_2) non-null (line 0xe9). If the owned byt… |
| `00ef8500` | DagLeafCallbackSetSample | confirmed | Asserts Node->Type == DagNodeType_Leaf_Callback (node+0x18 must be 2, granny_dag.cpp line 0x108) and SampleCallback (param_2) non-null (line 0x109). Stores par… |
| `00ef85b0` | DagCrossfadeNodeSet | confirmed | Asserts Node->Type == DagNodeType_Node_Crossfade (node+0x18 must be 4, granny_dag.cpp line 300). If the flag byte at +0x28 is set and a different pointer is he… |
| `00ef8650` | DagWeightedBlendNodeSet | confirmed | Asserts Node->Type == DagNodeType_Node_WeightedBlend (node+0x18 must be 5, granny_dag.cpp line 0x14d) and ReferenceSkeleton (param_2) non-null (line 0x14e). St… |
| `00ef8a30` | SetBlendDagNodeChildren | confirmed | Asserts !IsBlendDagLeafType(Node) (node+0x18 must be >= 3, granny_dag.cpp line 0x218). If the child array param_3 is non-null: zeroes the first dword of each o… |
| `00ef8c70` | AppendBlendDagNodeChild | confirmed | Asserts !IsBlendDagLeafType(Node) (node+0x18 >= 3, line 0x24c), Child != NULL (line 0x24d) and Child->ParentNode == NULL (first dword of the child is 0, line 0… |
| `00ef9500` | DagEvaluateTranslationRotation | confirmed | Asserts ResultRotation (param_4, granny_dag.cpp line 0x3f8) and ResultTranslation (param_3, line 0x3f9) non-null. Switches on node type: 0 calls 00eec230 on th… |
| `00ef9ca0` | DagNodeNewAnimationBlendLeaf | confirmed | Asserts the ModelInstance argument (param_1) non-null (granny_dag.cpp line 0x53, else returns 0). Takes a 0x2c-byte node from the pool with 00f270a0 (DAT_01b93… |
| `00ef9d20` | DagNodeNewLocalPoseLeaf | confirmed | Asserts LocalPose (param_1) non-null (granny_dag.cpp line 0x62, else returns 0). Takes a 0x2c-byte node from the pool with 00f270a0, zeroes it, sets +0xc to 1.… |
| `00ef9da0` | DagNodeNewCallbackLeaf | confirmed | Asserts SampleCallback (param_1) non-null (granny_dag.cpp line 0x74, else returns 0). Takes a 0x2c-byte node from the pool with 00f270a0, zeroes it, sets +0xc … |
| `00ef9ed0` | DagNodeNewWeightedBlend | confirmed | Asserts ReferenceSkeleton (param_1) non-null (granny_dag.cpp line 0xa8, else returns 0). Takes a 0x2c-byte node from the pool with 00f270a0, zeroes it, sets +0… |
| `00f26370` | FindNodeDataTypeMarker | confirmed | Loops i from 0 to DAT_01b93e80-1 over the type-marker table: calls 00f1baa0(dword at (table[i])+4, p1) and returns i for the first nonzero result, else -1. |
| `00ee11bd` | DagEnsureCachedSources | likely | For each source in the DAG (count at param_2+0x10, array base at param_2+0x14): asserts the source pointer is non-null ('TheDag.Sources[Idx]', line 0xf4) and, … |
| `00ee1250` | DagNodeAt | likely | Returns element param_2 of the pointer array at param_1+0xc (count at param_1+8) after asserting 0 <= param_2 < count ('Index' out of range reported at granny_… |
| `00ee12a0` | DagFindNodeIndex | likely | For a non-null node pointer param_2, scans the pointer array at param_1+0xc (count at param_1+8) and returns the first matching index; if none matches it repor… |
| `00ef8f70` | DagEvaluatePose | likely | Asserts PoseCache (param_6, granny_dag.cpp line 0x2dd) non-null, then switches on node type (+0x18): 0 AnimationBlend: acquires a cache entry (00efa440 with th… |
| `00ef9e20` | DagNodeNewCrossfade | likely | Takes a 0x2c-byte node from the pool with 00f270a0, zeroes it, sets +0xc to 1.0 and the type at +0x18 to 4, then calls 00ef85b0 with param_4, param_3, param_5,… |
| `00efa110` | DagNodeInstantiate | likely | Asserts SourceNodeList (param_2, granny_dag.cpp line 0x51d) and DestNodeList (param_3, line 0x51e) non-null when param_4 > 0. Switches on node type (+0x18): 0 … |
| `00ef8940` | ZeroDwordOfPointerArrayTargets | speculative | Loops over count = dword at +4 entries of the pointer array at +8 and writes 0 to the first dword of each pointee. No strings; the caller 00ef8c70 asserts that… |
| `00ef9900` | DagLeafApplyOp | speculative | Switches on node type (+0x18). Type 0 calls 00eebea0(node+0x1c, param_2). Type 2 calls the function pointer at +0x20 with (+0x28, param_2) when non-null. Types… |
| `00f26550` | DispatchNodeDataByKind | speculative | Tests the DAG node kinds in a fixed order with the accessors 00edf420, 00edc560, 00ede220, 00edbd10, 00edb730, 00edaff0, 00eda2f0, 00ed9700, 00ed8bf0 and 00ed8… |
| `00ee1860` | — | unknown | Given the DAG (param_1), a node (param_2) and an output index (param_3 in [0, node+0x20), asserts at lines 0x21d/0x21e). Finds the node's index with 00ee12a0. … |

### `lib.granny.dag.pose_cache`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00efa440` | PoseCacheAcquireEntry | confirmed | If the cache capacity (+0) is below param_2, frees the free-list chain at +8 (00efa410, then 00f0aee0 line 0x3f), clears it, stores the new capacity and sizes … |
| `00efa7a0` | PoseCacheReleaseEntry | confirmed | Given a pose pointer, asserts the entry header lies at pose-0x10 (granny_dag_pose_cache.cpp line 0x101). Unlinks the entry from the in-use list at cache+0xc (n… |
| `00efa410` | PoseCacheEntryFreeChain | likely | Recursively frees a chain of pose-cache entries linked through their first dword: calls itself on the next entry first, then deallocates this entry through PTR… |
| `00efa590` | PoseCacheAcquireFromSkeleton | likely | Asserts that the skeleton holder's +4 field (Skeleton, file line 0x93) is non-null, acquires a pose of the skeleton's bone count (+4 of the skeleton) with 00ef… |

### `lib.granny.dag2.additive_blend`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ee07f0` | AdditiveBlendNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_additive_blend.cpp line 0x1e) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x… |
| `00ee0b50` | AdditiveBlendEvalOutput0 | likely | Asserts instance InstanceData (line 0x58), SourceModel (0x59) and 'OnOutput == 0' (0x5a, returns 0 on failure). For output 0 it resolves five child edges with … |

### `lib.granny.dag2.blend`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edfbd0` | BlendNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_blend.cpp line 0x1e) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 are no… |
| `00edff00` | BlendNodeBlendPoses | likely | Asserts Weight in [0.0, 1.0] (granny_dag2_blend.cpp line 0x58, 'Weight' out of range reports and returns). Checks the BoneCount range against the pose counts (… |
| `00ee0220` | — | unknown | Asserts instance InstanceData (line 0xd2), SourceModel (0xd3) and 'OnOutput == 0' (0xd4 when param_4 is non-zero), each returning 0. For output 0 it resolves c… |

### `lib.granny.dag2.callback`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edf420` | CallbackNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_callback.cpp line 0x1a) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 are… |
| `00edf710` | CallbackEvalOutput | likely | Asserts instance InstanceData (line 0x5e) and SourceModel (0x5f), OnOutput range against node+0x20 (0x60), 'Node.TotalOutputEdgeCount == Node.TotalInputEdgeCou… |
| `00edf850` | CallbackEvalFloatOutput | likely | Float-output counterpart of 00edf710: asserts InstanceData (line 0x84), SourceModel (0x85), OnOutput range (0x86), edge counts (0x87), 'Node.OutputEdges[OnOutp… |

### `lib.granny.dag2.instance`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00efae50` | Dag2InstanceEvaluateNodeOutputPose | confirmed | Asserts IsValidNodeIndex(NodeIndex) (granny_dag2_instance.cpp line 0x13a) and NodeIndex in [0, instance+8) (line 0x13b, '%s %d is out of range [0, %d)'), gets … |
| `00efb1f0` | Dag2InstanceEvaluateTrackmaskOutput | confirmed | Asserts NodeIndex (0x186, 0x187) and OnOutput (0x18c), gets the node with 00ee1250, asserts the edge type is TrackmaskParameterType (0x18e). With no source edg… |
| `00efb8a0` | Dag2InstanceEvaluateFloatOutput | confirmed | Asserts IsValidNodeIndex (granny_dag2_instance.cpp line 0x243), NodeIndex range (0x244), AtNode non-null (0x247), OnOutput range (0x249) and that the edge type… |
| `00efbb90` | Dag2InstanceQueryPoseOutputByte | likely | Asserts NodeIndex (0x288) and OnOutput (0x28e) ranges, gets the node with 00ee1250, and asserts the edge type is PoseParameterType (0x28f). With no source edge… |

### `lib.granny.dag2.locked_blend`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ede220` | LockedBlendNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_locked_blend.cpp line 0x1d) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30… |
| `00ede6f0` | LockedBlendEvalOutput0 | likely | Asserts instance InstanceData (line 0xe5), SourceModel (0xe6) and 'OnOutput == 0' (0xe7, when param_4 is non-zero). For output 0 it resolves two child edges (n… |
| `00edee60` | LockedBlendFloatOnOutput | likely | Asserts instance InstanceData (line 0x20a) and SourceModel (0x20b) and OnOutput range against node+0x1c (line 0x20c). Resolves the two child edges with 00ee125… |

### `lib.granny.dag2.nway_blend`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edd6a0` | NWayBlendNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_nway_blend.cpp line 0x1e) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 a… |
| `00edda60` | NWayBlendEvalOutput0 | likely | Asserts instance InstanceData (line 0x4f), SourceModel (0x50), 'OnOutput == 0' (0x51) and 'Node.TotalInputEdgeCount >= 3' (line 0x52, node+0x14), returning 0 o… |

### `lib.granny.dag2.poseanimsource`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edc560` | PoseAnimSourceNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_poseanimsource.cpp line 0x29) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x… |
| `00edc900` | PoseAnimSourceEvalOutput | likely | Asserts instance InstanceData (line 0xa5) and SourceModel (line 0xa6) and the OnOutput index against param_2+4 (line 0xa7). For a non-null state record at (+0x… |
| `00edcc60` | PoseAnimSourceOutputDuration | likely | Asserts instance InstanceData (line 0x130) and SourceModel (line 0x131), the OnOutput index against node+0x1c (line 0x132), and 'Node.ExternalOutputEdgeCount =… |

### `lib.granny.dag2.scalarsource`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edbd10` | ScalarSourceNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_scalarsource.cpp line 0x1c) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30… |
| `00edc170` | ScalarSourceOnOutputValue | likely | Asserts instance dword +0x10 (InstanceData, granny_dag2_scalarsource.cpp line 0x5d) and the OnOutput index against node+0x1c (line 0x5f, 'OnOutput' range). Ret… |

### `lib.granny.dag2.scaleoffset`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edb730` | ScaleOffsetNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_scaleoffset.cpp line 0x1c) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 … |
| `00edba10` | ScaleOffsetEvalOutput0 | likely | Asserts instance dword +0x10 (InstanceData, granny_dag2_scaleoffset.cpp line 0x62) and 'OnOutput == 0' (line 99) when param_4 is non-zero, returning 0 for both… |

### `lib.granny.dag2.selection`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00edaff0` | SelectionNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_selection.cpp line 0x21) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 ar… |
| `00edb370` | SelectionEvalOutput0 | likely | Asserts instance InstanceData (line 0x73) and SourceModel (line 0x74) non-null and 'OnOutput == 0' (line 0x75, param_4). For output 0 it reads an index dword f… |

### `lib.granny.dag2.sequence`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00eda2f0` | SequenceNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_sequence.cpp line 0x21) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 are… |
| `00eda580` | SequenceFindOutputIndexTime | likely | Asserts param_5 (OutputIndex, line 0x67) and param_6 (OutputTime, line 0x68) non-null. Over the instance's 0x1c state records computes total = sum of (float at… |
| `00eda7b0` | SequenceEvalOutput0 | likely | Asserts instance InstanceData and SourceModel (lines 0xa4, 0xa5) and 'OnOutput == 0' (line 0xa6) when param_4 is non-zero. For OnOutput 0 it calls 00eda580 to … |
| `00edaae0` | — | unknown | Asserts instance dword +0x10 (InstanceData, line 0x13c) and then, for each sequence entry (count at param_2+4, entries at param_2+8 with stride 0x10), writes a… |

### `lib.granny.dag2.subgraph`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed9700` | SubgraphNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_subgraph.cpp line 0x26) and returns 0. Otherwise, when the instance dwords at +0x2c and +0x30 are… |
| `00ed9df0` | SubgraphListNextSlot | likely | Asserts param_2 non-null (granny_dag2_subgraph.cpp line 0xf8, otherwise reports and returns 0). Otherwise, if ((param_2 - dword at param_1+8) >> 2) + 1 is less… |
| `00eda050` | SubgraphRequireSkeleton | likely | Checks instance (param_3) dword +4 (Instance.SourceModel) and the dword at that model +4 (Instance.SourceModel->Skeleton). A null SourceModel reports 'is not a… |
| `00eda0b0` | SubgraphRequireSkeletonPose | likely | Pose-output twin of 00eda050: reports 'Instance.SourceModel' (granny_dag2_subgraph.cpp line 0x19f) when the instance dword +4 is null, and 'Instance.SourceMode… |
| `00eda110` | SubgraphRequireSkeletonFloat | likely | Float-output twin of 00eda050: reports 'Instance.SourceModel' (granny_dag2_subgraph.cpp line 0x1af) when the instance dword +4 is null and 'Instance.SourceMode… |
| `00ed9d60` | — | unknown | If param_2 is zero it reports 'is not allowed to be NULL' (granny_dag2_subgraph.cpp line 0xd8) and returns 0; otherwise it returns the result of 00ee1250(param… |

### `lib.granny.dag2.timeshift`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed8bf0` | TimeShiftNodeDataOrNull | likely | Same shape as 00ed8880 with the timeshift file and marker: a null param_1 reports 'is not allowed to be NULL' (granny_dag2_timeshift.cpp line 0x1a) and returns… |
| `00ed8fe0` | TimeShiftOnOutputEvaluate | likely | Validates the instance (+0x10 InstanceData and +4 SourceModel, else asserts at timeshift lines 0x8a/0x8b) and the OnOutput index param_4 against the node's out… |
| `00ed90e0` | TimeShiftOnOutputEvaluateFloat | likely | Same validation as 00ed8fe0 (InstanceData and SourceModel asserts at lines 0xa8/0xa9, OnOutput range at 0xaa against node+0x20), except the edge must have Para… |

### `lib.granny.dag2.trackmask`

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed0870` | TrackMaskCreateFilled | confirmed | Cdecl-style, two stack arguments (value, bone count), 271 bytes. When the count is below 1 it logs through 00f07e00 (level 3, code 0x2a, granny_track_mask.cpp … |
| `00ed0bc0` | TrackMaskFreeTracked | confirmed | Cdecl, one stack argument (pointer), 25 bytes. Pushes the pointer, the line 0xb3 and the string granny_track_mask.cpp, then calls the function pointer at 01b93… |
| `00ed0be0` | TrackMaskCopy | confirmed | Cdecl, one stack argument (source mask), 91 bytes. Calls 00ed0870(source value at +0, source count at +4) to create a new filled mask. If that succeeds it copi… |
| `00ed8880` | TrackmaskNodeDataOrNull | likely | Null param_1 reports 'is not allowed to be NULL' (granny_dag2_trackmask.cpp line 0x1d) and returns 0. Otherwise, if the instance dwords at +0x2c and +0x30 are … |
| `00ed0ac0` | TrackMaskCreate | speculative | Cdecl-style, one stack argument (bone count), 150 bytes. When the count is below 1 it logs through 00f07e00 (level 3, code 0x2a, granny_track_mask.cpp line 0x8… |
| `00ed8bb0` | TrackmaskOnOutputOffsetSum | speculative | Asserts that param_4 is zero ('OnOutput == 0', granny_dag2_trackmask.cpp line 0xa4); if it is non-zero it reports and returns 0. Otherwise it returns the dword… |
| `00ed0b60` | — | unknown | Cdecl, two stack arguments (table pointer, index), 23 bytes, returns a float in st(0). If index is below the dword at table+4 it loads the float at [table+8 + … |

### `lib.granny.data_type`

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00efc480` | TableFindRecordWithOffset | likely | Walks a null-terminated table of 8-dword records (stops at a record whose first dword is 0). Calls 00f1baa0(record+4, param_3) as a key predicate and, on the f… |
| `00efc610` | ReadScalarAsFloat10 | likely | Fastcall (eax = value pointer, edx = type code): returns the value as an x87 float10 by type code. 8 returns thunk 00f1bb10(value, 1) when the value is nonzero… |
| `00efc760` | DataTypeFormatScalarAligned | likely | Given a cursor record (flag byte +9, cursor +0xc, base +0x10) and a type code, produces the scalar's text (or only its size) and advances the cursor by the 4-a… |
| `00efc8c0` | DataTypeConvertMemberArray | likely | Converts or copies one member array from a source buffer (param_2) to the destination at EAX, using member definition param_3 (type code at +0) and a destinati… |
| `00efcc80` | DataTypeWriteMemberRecords | likely | Walks a member list (param_4) against a lookup table (param_2, param_3 via 00efc480) and writes each member by its own type: type 1 recurses into its sub-list;… |
| `00efd320` | DataTypeWriteObjectOnce | likely | Looks up the object key param_3 with 00f0fea0. If it is not yet registered: when param_5 is 0 it reserves a new output offset (00eea520 of the type size, curso… |
| `00efd420` | DataTypeWalkWithoutFlag | likely | Builds a 0x14-byte walk context with byte +8 = 0 and byte +9 = (param_4 != 0), then calls 00efcb40 with it and the remaining arguments. |
| `00efd470` | DataTypeMemberRemoveShift | likely | Asserts Type, Object and MemberName non-null (lines 0x4a3-0x4a5, file granny_data_type_conversion.cpp). Looks up MemberName with 00efc480 and, when found, comp… |
| `00efd5a0` | DataTypeObjectByteSize | likely | Builds a zeroed context with byte +8 = 1 and byte +9 = 0, calls 00efd320 (size-only pass, param_5 = 0) and returns the size accumulated in the context (local_8… |
| `00efd600` | DataTypeWriteObjectRelocated | likely | Builds a context with byte +8 = 1 and byte +9 = (param_4 != 0). Creates two containers with 00f0fce0, writes the object with 00efd320, and when both containers… |
| `00efddb0` | DataTypeBuildObjectBuffer | likely | Computes the object's byte size with 00efd5a0, allocates it through PTR_FUN_01b9378c (granny_data_type_conversion.cpp line 0x4c6, alignment 4), and when the al… |
| `00efd3d0` | DataTypeWalkWithWriteFlag | speculative | Builds a 0x14-byte walk context with byte +8 = 1 and byte +9 = (param_4 != 0), then calls 00efcb40 with it and the remaining arguments. |
| `00f0bef0` | ChainSlotsInitThree | speculative | Runs three times over the table of dwords at EDI+0xd8c (entries at +0xd8c, +0xd90, +0xd94): each pass clears the dword at EDI+0xd58, clears the dword at EDI+0x… |
| `00efc510` | — | unknown | Fastcall (ecx = value pointer, edx = type code): returns the value as a float by type code. 8 returns 00f1bc30(value) when the value is nonzero, else 0; 10 ret… |
| `00efcb40` | — | unknown | Walks a null-terminated member definition list (records of 8 dwords). For each member: if the flag at ctx+8 is set calls 00eeaaf0; looks up the member's table … |

### `lib.granny.data_type.fixup`

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed4e00` | DataTypeRemapStringIndices | confirmed | Cdecl-style, two stack arguments (database record, type record pointer), 146 bytes, recursive. If the type record is not already in the visited tree (00f0fef0 … |
| `00efd0d0` | DataTypeFixupReferences | likely | If the member list at param_3+4 is a valid type (00eea6c0), iterates members (00f25240). For each new key (00f0fef0 test, 00f0fd80 mark) it switches on the mem… |
| `00efd750` | FixupRangeRegisterAndRebase | likely | Registers the slot at param_2 in the container at param_1+4 (skipping it when already registered by 00f18170/00f18210), records its old value (00f18180), then … |
| `00efd870` | DataTypeRelocateMemberPointers | likely | If the member count (00eea6c0 test) is nonzero, walks members by type (00f25240 loop): 1 recurses into inline sub-objects; 2 and 5 rebase the reference by addi… |
| `00efdcc0` | DataTypeRelocateWithUndo | likely | Creates an undo container (00f18030 with element size 8 and capacity 0x400), builds an object view (00f25360), runs the relocation 00efd870 with delta param_5,… |
| `00efd6d0` | FixupSlotIsRegistered | speculative | Returns true when the offset in eax is 0; otherwise reads the dword at base(ecx)+eax, and when it is nonnegative and below the container count (00f18170), retu… |
| `00efd720` | FixupSlotRecord | speculative | When the offset in eax is nonzero, takes the slot at base(ecx)+eax, saves its old value, inserts a record keyed by the base into the container at +4 (00f18180)… |
| `00ed4d90` | — | unknown | Cdecl-style, one stack argument (file object pointer), 111 bytes. Calls 00f01060 on the file to get the object type and reads the run-time type tag at file+4 (… |
| `00ed4ea0` | — | unknown | Cdecl-style, three stack arguments (database record, member list, instance pointer), 847 bytes, recursive, returns AL as success. Calls 00ed4e00 on the member … |
| `00ed53a0` | — | unknown | Cdecl-style, one stack argument (file object pointer), 111 bytes. Same logic as 00ed4d90: calls 00f01060 on the file and reads the run-time type tag at file+4 … |

### `lib.granny.data_type.io`

**(no class)** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00efea10` | DataTypeIoContextCreate | likely | Allocates a 0x60-byte context through the pool allocator (00f24cd0 line 0xa0, 00f24ef0, 00f250c0 line 0xa4) and, if the allocation and the scratch node (00f0fc… |
| `00efebf0` | DataTypeIoContextRelease | likely | If the context is non-null, releases its map at +0x38 (00f0fd50), its container at +0x18 (00f18310), frees its pool with 00efe9d0 at +0x44, and deallocates the… |
| `00efec30` | DataTypeIoEntryFindOrAdd | likely | Looks up param_3 in the keyed map at ctx+0x38 (00f0fea0) and returns the entry if found. Otherwise allocates a new entry from the container at ctx+0x18 (00f181… |
| `00efed40` | DataTypeIoNameEntryRefresh | likely | Finds or adds the entry for the static key DAT_01b930a8 through 00efec30 on the context, and when the entry's stored value differs from the context's +8 value,… |
| `00efeea0` | DataTypeIoRegisterDefinition | likely | Computes a definition signature with 00efed90, looks it up in the ordered tree at ctx+4 (compare by first dword, then 00eea900 for equal keys), and returns the… |
| `00efecd0` | SetFieldAt0xC | speculative | Cdecl: stores its second stack argument (esp+8) into the dword at the first argument's +0xc (mov eax,[esp+8]; mov ecx,[esp+4]; mov [ecx+0xc],eax; ret). |
| `00efed90` | DataTypeSignatureWrite | speculative | Recursively writes a canonical signature of a member definition list into a growing buffer (00f25670 start, 00f25680 writes of dword/length values, 00f256d0 en… |
| `00eff300` | DataTypeStoreFirstDwordOfPairs | speculative | Thiscall (eax = member list): for each member in the list (stopping at a zero type), advances the destination by the member size (00eeaa40); for type-8 members… |
| `00eff350` | DataTypeStoreSecondDwordOfPairs | speculative | Thiscall (eax = member list): like 00eff300 but stores the second dword of each pair ((param_2[1]+4)+index) into the destination for type-8 members. |
| `00eff130` | — | unknown | Recursive walk over a member list (records of 8 dwords) against a value array: inline (1) recurses into the sub-list; type 2 zeroes the value when the referenc… |
| `00eff240` | — | unknown | Counts the type-8 members (string members, per the measuring in 00efc760) of the member list, allocates an array of 8-byte pairs through PTR_FUN_01b9378c (gran… |
| `00eff3d0` | — | unknown | Looks up or adds the entry for the object (00efec30), returns it if already done, otherwise records the element count and stride, allocates pair arrays for cal… |
| `00eff640` | — | unknown | Switches on the member type at *param_2 and writes the member value at offset param_1 of the object: type 1 applies 00eff920 to each element of the sub-list; t… |

### `lib.granny.dispatch`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ef58c0` | OwnerSlot1cDispatch | speculative | Takes the node in arg1. Its owner is the dword at +8; when the node is not its own owner and the owner's self link (+8 of the owner) equals the node, and the t… |
| `00ef58f0` | OwnerSlot20Dispatch | speculative | Same shape as 00ef58c0 but jumps through the owner table slot at +0x20 with the owner as the argument; returns 0 in AL when the owner check or the slot fails. |
| `00f26280` | DispatchInitSlot0 | speculative | Initialises the pointers PTR_FUN_01b93e70, 01b93e74 and 01b93e78 to 00f0a660, 00f0a7b0 and 00f0a8f0 and replaces them with 00f261a0, 00f261d0 and 00f26200 when… |
| `00f262d0` | DispatchInitSlot1 | speculative | Same initialisation as DispatchInitSlot0 (00f26280), then calls 00f0ab10 and jumps through PTR_FUN_01b93e74 (indirect tail jump). |
| `00f26320` | DispatchInitSlot2 | speculative | Same initialisation as DispatchInitSlot0 (00f26280), then calls 00f0ab10 and jumps through PTR_FUN_01b93e78 (indirect tail jump). The Ghidra body is 695 bytes … |
| `00ef6490` | — | unknown | Dispatches on the flag dword *param_1, testing values in order: 4 calls the hook at PTR_FUN_01b9433c with no arguments; 2 calls PTR_FUN_01b94338 with (param_1+… |

### `lib.granny.file`

**(no class)** (31)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed8650` | FileHeaderDecodeFlags | confirmed | Requires non-null param_2 (TotalHeaderSize), param_3 (IsByteReversed) and param_4 (IsPointerSizeDifferent) (asserts at granny_file_format.cpp lines 0xfe, 0xff,… |
| `00eea740` | DataTypeMemberElementCount | confirmed | Returns the member's ArrayWidth (dword +0xc) for member types other than 2, 3, 4, 5, 7, 8 and 0x16 when it is nonzero, otherwise 1. For those listed types with… |
| `00f0b5b0` | MemoryFileReaderCreate | confirmed | Asserts MemorySize (param_1) >= 0 and that Memory (param_2) is non-null unless the size is 0, reporting '%s was not true' through 00f07e00 and returning 0 on f… |
| `00f0b650` | MemoryWriterGrow | confirmed | With ESI = writer and EBX = requested size: rounds the size up to a multiple of the granularity at +0x28 to get the new capacity, allocates it through the allo… |
| `00f0b6b0` | MemoryWriterRelease | confirmed | If param_1 (writer) is non-null: when its buffer at +0x20 is non-null, frees the buffer through the free hook (granny_memory_file_writer.cpp line 0x5e) and cle… |
| `00f0b6f0` | MemoryWriterSeek | confirmed | Returns 0 for a NULL writer (reporting '%s is not allowed to be NULL' with 'Writer') and for a whence value other than 0, 1 or 2. Converts the offset: whence 1… |
| `00f0b7b0` | MemoryWriterWrite | confirmed | Returns 0 for a NULL writer. When the checksum flag at +0x14 is set, updates the CRC state at +0x18 over the data with 00f25680 (64-bit count). It computes the… |
| `00f0b8e0` | MemoryWriterCreate | confirmed | Asserts BlockSize (param_1) > 0 ('BlockSize > 0'; returns 0 on failure). Allocates a 0x34-byte object through the allocation hook (granny_memory_file_writer.cp… |
| `00f0b990` | MemoryWriterDetachBuffer | confirmed | Checks the writer and both output pointers (reports '%s is not allowed to be NULL' with 'BufferPtr' or 'BufferSize' otherwise and returns 0). If the flag at +0… |
| `00f0ba30` | MemoryWriterFreeObject | confirmed | If param_1 is non-null, frees the object through the free hook (granny_memory_file_writer.cpp line 0x130) without touching its buffer. |
| `00ed83b0` | FileInfoEnsureTypeConverted | likely | Calls 00f01060(param_1, &local_8) to fetch a file-info record; reads the int at (param_1+4)+0x24 and returns the cached dword local_4 when it equals the sentin… |
| `00eea4d0` | DataTypeMemberTableValue | likely | Returns a per-member-type dword from a 0x14-byte-stride table: 01b9336c when param_2 == 0x20, 01b93370 when param_2 == 0x40 (indexed by param_1); returns 0 for… |
| `00eea520` | DataTypeChainBytes | likely | Walks a data-type member list (0x20-byte entries, the type dword at +0, the list continues while the pointer is non-null and the type is nonzero) and returns t… |
| `00eea660` | DataTypeMemberHasRef | likely | Predicate on one data-type member: returns 1 for member types 5, 7 and 8; for types 2, 3 and 4 returns 1 when the child pointer at +8 is non-null; for inline t… |
| `00eea6c0` | DataTypeChainHasRef | likely | Walks a member chain (0x20-byte entries, while the pointer is non-null and the type dword is nonzero) and returns 1 as soon as 00eea660 returns nonzero for an … |
| `00eea6f0` | DataTypeMemberKindOr | likely | Walks a member list (0x20-byte entries while the type dword is nonzero). For inline members (type 1) it recurses into the child list at +8; for other types it … |
| `00eea7c0` | DataTypeMembersMatch | likely | Recursively compares two member chains (0x20-byte entries: type at +0, a payload at +4, child list at +8, ArrayWidth at +0xc). Returns 1 when they match: each … |
| `00eea960` | DataTypeChainPrefixMatch | likely | Walks two member chains in step: returns 0 on a type or ArrayWidth mismatch, asserts the switchable-type message for type 6, and compares child lists with 00ee… |
| `00eea9f0` | DataTypeMemberByteSize | likely | Returns the byte size of one data-type member. For types other than 1 it returns the dword at 01b9336c + type*0x14. For inline type 1 it asserts 'Member.Refere… |
| `00eeaa40` | DataTypeMemberTotalBytes | likely | Returns the total bytes of one data-type member: for types other than 1, table dword 01b9336c + type*0x14 times 00eea740 (element count); for inline type 1 it … |
| `00eeaac0` | DataTypeMemberKindLookup | likely | Returns the kind dword from the 0x14-stride table at 01b93374 for non-inline member types; for inline type 1 it returns the OR over the children from 00eea6f0. |
| `00eeaaf0` | DataTypeInitInstance | likely | Initialises one data-type instance at param_2 and returns the bytes it covers. For types other than 1 and 9: size = 00eeaa40(member), memset to zero, return si… |
| `00eeabe0` | DataTypeWalkKindCallbacks | likely | For each of param_2 elements, walks the member list at param_1 with a running destination (param_3): inline members recurse with their element count (00eea740)… |
| `00ed85b0` | MagicLookupDescriptor16 | speculative | Calls 00ed84d0 on param_1's magic, and on success looks up a descriptor with 00ed8420 using the format dword and reversal byte that 00ed84d0 produced; it copie… |
| `00ed8860` | Wrapper_FileMagicIdentify | speculative | Forwards its two arguments with the constant pointer 0x0186bb14 as the first argument to 00ed84d0 (FileMagicIdentify), discards the result and returns nothing … |
| `00eea560` | DataTypeChainStride32 | speculative | Counts 0x20-byte entries from param_1: the loop adds 0x20 per iteration and reads the next pointer from the first dword of the entry at the current stride, so … |
| `00eea900` | DataTypeMembersMatchWithCallback | speculative | Calls 00eea7c0(param_1, param_2, 00f1b1b0, 0, 0) and returns its result in EAX (the body ends with add esp; ret, so the value passes through though Ghidra show… |
| `00eea920` | DataTypeMembersMatchPlain | speculative | Calls 00eea7c0(param_1, param_2, 0, 0, 0) and returns its result in EAX (no callback). |
| `00eea940` | DataTypeMembersMatchStop | speculative | Calls 00eea7c0(param_1, param_2, 0, param_3, 0) and returns its result in EAX; param_3 becomes the stop pointer that makes the comparison fail when param_2 equ… |
| `00eeabb0` | DataTypeChainInit | speculative | Walks a member chain and calls 00eeaaf0 on each member at a running destination (param_2, advanced by each returned size). Returns the total bytes covered. |
| `00ed84d0` | — | unknown | Identifies the 16-byte magic at param_1 against known Granny file magics: first the magic at DAT_0186ba74 (on match writes *param_2 = 0x20 and *param_3 = the r… |

### `lib.granny.format`

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f247d0` | FormatPrintfCore | likely | Walks the format string p3, copying literal characters into the buffer p2 (capacity p1) and expanding % conversions: d/i (32-bit, or 64-bit with the 0x40 size … |
| `00f24240` | ReturnPosInfinity | speculative | Returns the double +infinity (bit pattern 0x7ff0000000000000) as an x87 value via fld qword. |
| `00f24260` | ReturnNegQuietNaN | speculative | Returns the double negative quiet NaN (bit pattern 0xfff8000000000000) as an x87 value via fld qword. |
| `00f24280` | ReturnNegInfinity | speculative | Returns the double -infinity (bit pattern 0xfff0000000000000) as an x87 value via fld qword. |
| `00f242a0` | PutCharIfRoom | speculative | If the capacity p1 is above 1 it stores byte p3 at p2 and returns 1; otherwise it returns 0 and writes nothing. |
| `00f242f0` | FormatUnsigned64Radix | speculative | Formats the unsigned 64-bit value (p3 low dword, p4 high dword) in radix p5 into p2 with capacity p1, using digit table p6. Digits are produced least significa… |
| `00f24740` | FormatSigned64 | speculative | Formats the signed 64-bit value (p3 low dword, p4 high dword) into p2 with capacity p1. When the high dword is negative it stores a minus sign (if there is roo… |
| `00f247a0` | FormatSigned32 | speculative | Sign-extends p3 to 64 bits (high dword = p3 arithmetic shift right 31) and calls 00f24740(p1, p2, p3, high, p4, p5); returns that result in EAX. |
| `00f24ca0` | FormatPrintfVarargs | speculative | Calls 00f247d0(p1, p2, p3, &variadic-argument pointer) so that the arguments after the format string are consumed by the formatter. |
| `00f243f0` | — | unknown | Formats the double given by p3 (low dword) and p4 (high dword) into p2 with capacity p1, radix p5 (double) and p6 fractional digits, digit table p7. Zero print… |

### `lib.granny.iterator`

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f25410` | IterateOverSubMembers | confirmed | If the iterator type [0] is not 1 it reports Illegal iterator type (%d) in IterateOverSubMembers (level 3, line 0xde), zeroes the 4-dword output and refreshes.… |
| `00f254d0` | IterateSubMembersByType | likely | Type 4 writes the output through 00f25390(0, *[10], *[9], *[0xc], out). Types 3 and 7 write output {2, *[10], *[0xc], *[9]} and, when the element and the count… |
| `00f25270` | IterStateRefresh | speculative | Operates on the iterator object in EAX. Sets the self-pointers [9]=&[8], [10]=&[7], [11]=&[6] and clears [12] and [13]. If the source pointer [1] and counts [2… |
| `00f25360` | IterInitType1 | speculative | Initialises the iterator record p3 as type 1: [0]=1, [1]=p1, [2]=p2, [3]=1, and [4]=0x17 when p1 is nonzero; then calls 00f25270 to refresh. |
| `00f25390` | IterInitWithCount | speculative | Stores p1 in [0], p2 in [1], p4 in [2] and p3 in [3]. When p3 and p2 are nonzero it sets [5] = 00eea520(p2) and [4] = *[1]. Then calls 00f25270 to refresh. |
| `00f253e0` | IterAdvance | speculative | Advances the iterator record p1: when [0] is 0 it adds 4 to [2]; otherwise it adds [5] to [2] and, when [0] is 1, adds 0x20 to [1], calls 00f25270 and returns … |
| `00f25590` | IterGetCurrentPointer | speculative | For iterator type 0 the value is the dword at [2] dereferenced ([2]->value) with [1]; for type 2 the value is [2] with [1]. It writes output {1, [1], value, 1,… |

### `lib.granny.localpose`

**(no class)** (40)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed69d0` | LocalPoseAggregateSize | likely | Opens a size-only aggregate with a 1808-byte stack context (00f24cd0 with granny_local_pose.cpp line 99, 00f24cf0 with 0x10, 00f24ef0 with 0x14, 00f24f80 with … |
| `00ed6a80` | LocalPoseBindMemory | likely | Requires param_2 to be 16-byte aligned (else reports 'IS_ALIGNED_16(Memory)' at line 0x72 and returns 0). Opens an aggregate of granny_local_pose.cpp line 0x76… |
| `00ed6b80` | LocalPoseEntryAt | likely | Takes a struct pointer (count dword at +0, base pointer at +4) and an index; if 0 <= index < count it returns base-relative address index*0x50 + 8 + param_1[1]… |
| `00ed6be0` | LocalPoseFillBoneRange | likely | Arguments: param_1 skeleton (bone array base at +8, stride 0x98), param_2 FirstBone, param_3 BoneCount, param_4 local pose header (count at +0, entry base at +… |
| `00ed6ec0` | LocalPoseBlendBoneRangeLod | likely | Asserts 0 <= FirstBone < header count and FirstBone+BoneCount <= count (lines 0x144, 0x145, else reports via 00f07e00 and returns). If the skeleton dword at +0… |
| `00ed7150` | LocalPoseChainMatrix | likely | Starting from a 4x4 identity, walks the bone chain from the given bone up the ParentIndex links (bone record +4 in the skeleton at param_1+8, stride 0x98) unti… |
| `00ed7440` | LocalPoseBlendRange | likely | Requires three non-null poses (assert 'is not allowed to be NULL' at granny_local_pose.cpp) and BoneStart >= 0, BoneCount >= 0 (asserts 'BoneStart >= 0' / 'Bon… |
| `00ed7d50` | LocalPoseCopyBones | likely | Asserts param_3 >= 0 (line 0x353 'BoneCount >= 0'), Dst count (param_2 dword 0) >= param_3 (line 0x354), Src count (param_1 dword 0) >= param_3 (line 0x355); e… |
| `00ed8100` | LocalPoseAllocateBind | likely | Calls 00ed69d0 with its argument to get a size, then calls the allocator through PTR_FUN_01b9378c with ('..\source\granny_local_pose.cpp', line 0x44, alignment… |
| `00ed8160` | LocalPoseCopyAllBones | likely | Asserts Dst.BoneCount >= Src.BoneCount (line 0x34a; here param_2 dword 0 >= param_1 dword 0) and otherwise calls 00ed7d50(param_1, param_2, param_1 dword 0), c… |
| `00eead70` | TransformRecordFill | likely | Writes a 0x44-byte (17-dword) transform record at param_1: dword 0 = 7 (a placeholder later overwritten by the flags), dwords 1..3 copy param_2[0..2] (position… |
| `00eeadf0` | TransformFlagsFromIdentity | likely | Sets the record's dword 0 to flags: bit 0 when the position (+4, 3 floats) L1 distance from 0 is >= the float at 0x017ebf0c, bit 1 when the orientation (+0x10,… |
| `00eeaed0` | TransformRecordBuildAndFlags | likely | Calls 00eead70(param_1, param_2, param_3, param_4) to fill a transform record, then 00eeadf0(record) to set its flags dword. Returns nothing. |
| `00eeaf00` | TransformSetIdentity | likely | Writes an identity transform record at param_1: dword 0 (flags) = 0, position dwords 1..3 = 0, orientation dwords 4..6 = 0 and dword 7 = 1.0f (0x3f800000), sca… |
| `00eeb120` | TransformApplyToPoint | likely | Transforms the float3 at param_1 by the transform record at param_2: the 3x3 at record+0x20 is applied to the point (x' = m0*x + m1*y + m2*z per row), then 00f… |
| `00eeb220` | TransformCompose | likely | Composes two transform records: param_2 (A) and param_3 (B) into param_1. Each orientation is turned into a 3x3 by 00f09990, the scale-shear at param_1+0x20 is… |
| `00eeb370` | TransformBlend | likely | Blends two transform records param_2 and param_4 with weight param_3 (t) into param_1. flags = A.flags \| B.flags. Position is lerped when bit 0 is set and zer… |
| `00eeb4b0` | TransformInvert | likely | Inverts the transform record at param_2 into param_1: the position and the quaternion xyz are negated, the 3x3 of the inverted orientation is built with 00f099… |
| `00eeb6d0` | TransformToMatrix4x4 | likely | Writes a column-major 4x4 matrix into param_2 (16 dwords) from the transform record at param_1: the 3x3 is the matrix of the orientation (00f09990), multiplied… |
| `00eebd00` | Mat3FromAxisAngle | likely | Writes the row-major 3x3 rotation matrix for the unit axis param_2 (3 floats) and angle param_3 (radians, fsin/fcos): diagonal (1-c)x^2+c and the off-diagonal … |
| `00f0a1c0` | PartsTransformCompose | likely | Calls 00f0a060(param_1, param_2, param_4) to transform the translation vector at param_4, 00f0a0d0(param_2, param_3, param_5) to sandwich the quaternion at par… |
| `00f10ec0` | PoseQuatPositionTransform | likely | Writes a transformed position and the orientation flag: calls 00f09990 to fill a 3x3 matrix (9 floats) on its stack, sets EDI[1..3] = M * (ESI[1], ESI[2], ESI[… |
| `00f10f40` | LocalPoseCompose | likely | Composes two 17-dword local pose records param_2 (A) and param_3 (B) into param_1. Position = A.pos, rotated by B.quat when B has the orientation bit (2), then… |
| `00ed6980` | Wrapper_LocalPoseCallbackLine77 | speculative | Pushes its one argument, the constant 0x4d (line 77) and the string '..\source\granny_local_pose.cpp', then calls through the function pointer stored at 01b937… |
| `00eeb620` | TransformFillFromParts | speculative | Fills the transform record at param_1 by calling 00f0a1c0(param_2, param_3, param_4, record+4, record+0x10, record+0x20), which writes the position, orientatio… |
| `00eeb780` | TransformToAffine12 | speculative | Writes a 12-dword affine form from the transform record at param_1: the 3x3 (orientation, multiplied by scale-shear through 00f096c0 when flags bit 2 is set) t… |
| `00eeb870` | TransformLeftComposeInPlace | speculative | Composes the record at param_2 (left operand) with the record at param_1 (right operand) through 00eeb220 into a 17-dword temporary, then copies the temporary … |
| `00eeb8a0` | TransformRightComposeInPlace | speculative | Composes the record at param_1 (left operand) with the record at param_2 (right operand) through 00eeb220 into a 17-dword temporary, then copies the temporary … |
| `00f22010` | CallPtr01b94334WithVec3At14 | speculative | Copies 3 floats from p3+0x14 into a local vector and calls the function pointer stored at 0x01b94334 (cdecl) with (&vector, p5, p6), then returns. |
| `00f22050` | CallPtr01b94338WithPosQuatAt14 | speculative | Copies 7 dwords from p3+0x14 into a local block (3 position dwords followed by 4 orientation dwords) and calls the function pointer at 0x01b94338 with (&positi… |
| `00f22400` | DecodeLocalPose17FromFlags | speculative | Writes a 17-dword local-pose record into p4: [0] presence bits (1 position, 2 orientation, 4 scale/shear), [1..3] position, [4..7] quaternion, [8..16] scale/sh… |
| `00f225b0` | EvalPoseAtIndex0Normalized | speculative | Writes a 17-dword local-pose record from the curve block p1 at sample index 0: position via 00ee3820(p1+0x10, 0, p2+1, DAT_01bfe130) with presence bit 1 when 0… |
| `00f226d0` | EvalPosOrientCurvesOrIdentity | speculative | If 00ee2b70(p2+0x10) is 0 calls 00f21dd0(3, p4, DAT_01bfe130, 0); otherwise zeroes p4[0..2]. If 00ee2b70(p2+8) is 0 calls 00f21dd0(4, p5, DAT_01b92f68, 1); oth… |
| `00f22790` | EvalScaleShearTrack | speculative | Uses the block p2+0x18. If 00ee2b40 is true it sets local_44[0]=4, samples 00ee3820(p2+0x18, *(p1+0xc), local_24, DAT_01b92f78) and calls the function pointer … |
| `00f22870` | DecodeLocalPoseAndApply | speculative | Sets dword 0 of a 17-dword local buffer to zero, decodes it with 00f22400(p1, p2, p3, buffer), applies 00eeb370(buffer, buffer, p7, p3+0x10) and calls 00ef6490… |
| `00f22920` | SelectTrackEvalHandler | speculative | Calls 00ed2d00(p1); if nonzero it returns the address 0x00f22750. Otherwise it classifies the blocks at p1+0x10, p1+8 and p1+0x18 each as 0 (identity via 00ee2… |
| `00f22939` | SelectTrackEvalHandlerTail | speculative | Continuation of 00f22920 (entered by a jump from 00f22930 with the block pointer in EBP). It classifies the three blocks at ebp+0x10, +8 and +0x18 as in 00f229… |
| `00ed81a0` | — | unknown | Asserts the param_1 header count (dword 0) equals param_5 dword 0 (BoneCount, string 'GetLocalPoseBoneCount(CompositePose) == BoneCount'). Computes an amount p… |
| `00eeafc0` | — | unknown | Replaces the float3 at param_1 with the product of the row-major 3x3 at param_2+0x20 and that vector (x' = m0*x + m1*y + m2*z, and the same for y and z). Posit… |
| `00f22240` | — | unknown | Calls 00ee3980 with (p2+0x10, *p1) to evaluate the position curve at the dword *p1, then 00ef2270 for 3 components and 00ef21f0 with the format byte at (p2+0x1… |

### `lib.granny.lookat`

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f08620` | LinkChainAdvanceOne | likely | Reads the bone index at *(this+0x54), calls LocalPoseEntryAt (00ed6b80) for that index, ORs 2 into the entry's flag dword, multiplies the entry quaternion at +… |
| `00f08dd0` | LinkChainSolveStep | likely | If LinkCount (this+0x4c) is 0 returns 1. Otherwise calls 00f088c0 for the current link axis and angle (returns 0 on failure), writes into this+0x3c a quaternio… |
| `00f088c0` | — | unknown | Builds frames from the 12-float input at param_1 (normalizes vectors at +0 and +9 with 00ecf060, crosses them, uses the swing helpers 00f087a0/00f086d0 and 00f… |
| `00f08e50` | — | unknown | Validates the head bone index (param_1) against the skeleton bone count at *(param_9+4), LinkCount (param_2) in 1..0x20 with LinkCount < HeadBoneIndex and Link… |

### `lib.granny.math`

**(no class)** (58)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f086d0` | QuatFromVectorPair | likely | If \|a\|^2 and \|b\|^2 (param_2 and param_3) both exceed the epsilon DAT_0186fe10, writes the quaternion (a x b, a.b + \|a\|\|b\|) to param_1 via the SQRT(fVar… |
| `00f093a0` | QuatFromAxisAngle | likely | Half-angle conversion: t = angle * 0.5 (constant at 0x17649d4), writes out.xyz = axis.xyz * sin(t) and out.w = cos(t) to param_1 (quaternion, 4 floats). |
| `00f093d0` | Mat3Invert3x3 | likely | Computes the determinant of the row-major 3x3 (9 floats at param_2). If it is not zero writes its inverse (adjugate divided by det) to param_1 and returns 1; o… |
| `00f094e0` | Mat4AffineInverseColMajor | likely | Inverts the 3x3 part of the column-major 4x4 affine at param_2 (elements 0-2, 4-6, 8-10) with det = fVar1; writes the 3x3 inverse only when det is not zero; al… |
| `00f096c0` | Mat3Mul | likely | Writes the row-major 3x3 product param_1 = param_2 * param_3 (9 floats each) with no aliasing guard; each output element is a sum of three products. |
| `00f097b0` | Mat3MulTransposeLhs | likely | Writes the row-major 3x3 product param_1 = transpose(param_2) * param_3 (9 floats each). |
| `00f09990` | QuatToMat3 | likely | Writes the row-major 3x3 rotation for quaternion (x,y,z,w) at param_2 to param_1 using scale 2 (DAT_017ea718 = 2.0): diagonal 1-2(y^2+z^2) etc. No normalizatio… |
| `00f09a60` | Mat3ToQuatLargestDiag | likely | Thiscall with ECX = row-major 3x3 matrix and EAX = index i (0..2); stack argument is the output quaternion pointer. Computes s = sqrt(M[i][i] - M[k][k] - M[j][… |
| `00f09b00` | Mat3ToQuat | likely | Converts the row-major 3x3 at param_2 to quaternion (x,y,z,w) at param_1. If trace (M[2][2]+M[0][0]+M[1][1]) > 0 it writes w = sqrt(trace+1)*0.5 and xyz = (M[2… |
| `00f09b90` | QuatMultiply | likely | Hamilton product of quaternions param_2 and param_3 (x,y,z,w) written to param_1; all inputs are loaded into locals before the write, so the output may alias a… |
| `00f09c20` | QuatRotateVec3 | likely | Rotates the vector at param_1 in place by the quaternion at param_2 (x,y,z,w): v' = R(q)*v with R(q) the matrix with scale 2 (DAT_017ea718); all three componen… |
| `00f09d40` | QuatSequenceSignFix | likely | Walks param_1 quaternions (4 floats each) in order; for each one whose dot product with the previous (already sign-fixed) quaternion is below DAT_01762a30 (0.0… |
| `00f09de0` | Mat3PolarDecompose | likely | Polar decomposition of the 3x3 M at param_1 by Newton iteration: R0 = M, or M*diag(1,1,-1) when det(M) < 0 (00f096c0). Each of up to 1000 steps inverts R with … |
| `00f0a060` | Mat3ApplyPointAdd | likely | Computes param_3 = M * param_3 + param_1, where M is the row-major 3x3 at param_2 (only elements 0-8 read) and param_1 is a float3 offset. Writes the 3-float v… |
| `00f0a0d0` | QuatSandwichWith3x3 | likely | Normalizes the quaternion at param_3 (4 floats) and keeps its length, converts it to a 3x3 with 00f09990, multiplies param_1 * M (00f096c0) and then that produ… |
| `00f0a190` | Mat3TripleMulInPlace | likely | Replaces the row-major 3x3 at param_3 by param_1 * M * param_2 (M = the original param_3), using two 00f096c0 products through a 36-byte temporary. |
| `00f0a590` | QuatRelativeRotationVec3 | likely | Negates the xyz of the quaternion at param_2 into a local, multiplies that conjugate by the quaternion at param_3 (00f09b90), then writes the rotation-vector f… |
| `00f0a660` | Affine4x4MulRowVec | likely | Writes the row-vector affine product param_1 = A*B for two 4x4 matrices with the translation in elements 12-14 (row 3), using the 3x3 product for elements 0-10… |
| `00f0a7b0` | Affine4x4MulColumnMajor3x4 | likely | Computes P = A*B for two row-vector affine 4x4 matrices (translation in elements 12-14) and writes its first three columns as 12 floats, out[4c+r] = P[r][c] fo… |
| `00f0a8f0` | Mat4Mul | likely | Writes the full row-major 4x4 product param_1 = param_2 * param_3 (16 floats), each element a sum of four products with no aliasing guard. |
| `00ecefe0` | FloatSqrt | speculative | Loads the float argument, applies fsqrt and returns the result in st(0) (7 bytes: fld, fsqrt, ret). |
| `00eceff0` | Vec3ScaleInPlace | speculative | Multiplies the three float fields at [arg1+0], [arg1+4] and [arg1+8] by the float argument arg2 in place (31 bytes; x87 fmul/fstp sequence, ret with no stack c… |
| `00ecf010` | Vec3Zero | speculative | Stores 0.0f into the three float fields at [arg1+0], [arg1+4] and [arg1+8] (15 bytes: fldz, three fst/fstp stores, ret). |
| `00ecf020` | Vec3LengthSquared | speculative | Returns x*x + y*y + z*z for the three float fields at [arg1+0], [arg1+4] and [arg1+8] in st(0) (27 bytes, x87 fmul/faddp sequence, no stack cleanup). |
| `00ecf040` | Vec3Length | speculative | Returns sqrt(x*x + y*y + z*z) of the three float fields at [arg1+0], [arg1+4] and [arg1+8] in st(0) (29 bytes, x87 fmul/faddp then fsqrt, ret with no stack cle… |
| `00ecf060` | Vec3NormalizeOrZero | speculative | Normalizes the vec3 at arg1 in place when its length exceeds the float constant at 018212d0 (bytes acc52737, about 1e-5); otherwise it zeroes the three fields.… |
| `00ecf0f0` | Vec3TransformByMat3InPlace | speculative | Transforms the vec3 at arg1 in place by the 3x3 float matrix at arg2 (nine floats at +0..+0x20), treating the vector as a row: out0 = m0*x + m1*y + m2*z, out1 … |
| `00ed17f0` | FloatSquare | speculative | Cdecl, one float stack argument, 7 bytes: loads it, squares it with fmul st(0),st(0) and returns the value in st(0). |
| `00ed1820` | FloatAbs | speculative | Cdecl, one float stack argument, 7 bytes: loads it, applies fabs and returns the value in st(0). |
| `00ed1860` | Vec4Copy | speculative | Cdecl, two stack arguments (destination, source), 31 bytes: copies four floats (16 bytes) from the source to the destination with x87 fld/fstp pairs and return… |
| `00ed18b0` | Mat3Copy | speculative | Cdecl, two stack arguments (destination, source), 61 bytes: copies nine dwords (36 bytes, a 3x3 float block) from the source to the destination and returns not… |
| `00ed1960` | QuatPrefixMultiplyArray | speculative | Cdecl, three stack arguments (base record, count, array), 91 bytes. For each of count quaternions in the array (16 bytes each) it calls 00f09b90 with the outpu… |
| `00ed19c0` | Mat3ConjugatePrefixArray | speculative | Cdecl, three stack arguments (base record, count, array of 3x3 float matrices), 180 bytes. Builds M1 = QuatToMat3(base+0x10) with 00f09990, M2 = Mat3Mul(base+0… |
| `00ed1c50` | Vec3MaxAbsIndexAndValues3 | speculative | Cdecl-style, three stack arguments (vec3 array pointer, index array pointer, output array pointer). Runs three iterations; each takes the 3-float vector at the… |
| `00ed3740` | Vec3Copy | speculative | Cdecl, two stack arguments (destination, source), 25 bytes: copies three floats from the source to the destination with x87 fld/fstp and returns nothing. |
| `00ed3760` | Vec3Sub | speculative | Cdecl, three stack arguments (destination, first, second), 37 bytes: writes first minus second componentwise (three floats) to the destination and returns noth… |
| `00ed3790` | Vec3Dot | speculative | Cdecl, two stack arguments, 29 bytes: returns the dot product of the two three-float vectors in st(0) (x*x' + y*y' + z*z'). |
| `00ed37b0` | Vec3Cross | speculative | Cdecl, three stack arguments (destination, first vector, second vector), 61 bytes. Writes the cross product of the first and second three-float vectors to the … |
| `00ed37f0` | Vec3ScaledAddInPlace | speculative | Cdecl, three stack arguments (destination, scale, source), 43 bytes: adds scale times the source to the destination componentwise, in place, and returns nothin… |
| `00ed3850` | Vec3LinearCombine2 | speculative | Cdecl, five stack arguments (destination, scale a, vector a, scale b, vector b), 63 bytes: writes a times vector a plus b times vector b componentwise to the d… |
| `00ed40e0` | Mat3DeterminantSignCompare | speculative | Fastcall: ECX = first 3x3 matrix (param_1), EAX = second 3x3 matrix (this), no stack arguments, 245 bytes, returns ushort. Computes the determinant-like triple… |
| `00ef07f0` | FloatArrayScaleInPlace | speculative | Multiplies the first param_3 floats at param_1 by the scalar param_2 in place, unrolled four at a time with a scalar tail loop. |
| `00ef0860` | FloatArraySumOfSquares | speculative | Returns the sum of squares of the first param_2 floats at param_1 as an x87 extended value (float10 in ST0), unrolled four at a time with a scalar tail. |
| `00ef4220` | Vec4TimesMatrix4InPlace | speculative | Replaces the 4-float vector at param_1 with its product with the 4x4 float matrix at param_2 treated row-major: out[j] = sum over i of v[i] * M[i][j] (computed… |
| `00ef49d0` | ProjectPointTwoMatrices | speculative | Builds the homogeneous point (2x - param_2)/param_2, (2y - param_3)/param_3 and z (with (z - DAT_01762a34)*0.5 when *(param_1+0x1c) == 1) from the input vector… |
| `00ef4b20` | ProjectSampleTwoMatricesNormalized | speculative | Copies the three dwords at param_1+0xec, +0xf0 and +0xf4 to param_6. It builds normalised x and y from param_4 and param_5 in the (2x - a)/a form of 00ef49d0, … |
| `00ef55f0` | PackFourFloatsToDword255 | speculative | Stores through param_1 one dword made of four bytes: each of param_2..param_5 is multiplied by DAT_017ed9d0 (255.0), rounded with ROUND, and placed in order fr… |
| `00ef5e40` | UnpackDwordToFourFloats | speculative | Writes four floats to param_2: each byte of the dword param_1 (low byte first) multiplied by DAT_017f1ae4. |
| `00ef6550` | FractionSplitByConstant | speculative | Writes 0 to both output floats; when the input is positive it writes c*p to the first output and 1/(p - c*p) to the second, where c is the float at DAT_01b936d… |
| `00f087a0` | QuatNormalSwing | speculative | Computes n = B x A from in_EAX (A) and ESI (B), normalizes it with 00ecf060 (length returned in ST0); if the length is zero writes (0,0,0,1) to EBX and returns… |
| `00f0a300` | Vec3PerpendicularBasis | speculative | From the input vector param_1, zeros the axis of its largest absolute component in a (1,1,1) helper, writes the normalized cross product of the helper and the … |
| `00f0ab10` | AffineProductVariantsProbe | speculative | Fills local_270[1..16] with 0..15 and local_220[1..16] with 16..31 (index 0 NaN, then zeroed), calls 00f0a660, 00f0a7b0, 00f0a8f0 and the function pointers PTR… |
| `00f25a60` | MultiplyAffine4x4 | speculative | Writes into p1 (16 floats) the product of p2 and p3: for rows 0..3 result[i][j] = A[i][0]*B[0][j] + A[i][1]*B[1][j] + A[i][2]*B[2][j], with A taken as 4 rows o… |
| `00f25b50` | MultiplyAffine4x4B | speculative | Same body as MultiplyAffine4x4 (00f25a60): writes the product of the 4x3 matrix p2 and the 4x4 matrix p3 into p1, adding the last row of p3 to row 3. |
| `00f25c40` | MultiplyMat4ByAffineOut | speculative | Writes 12 floats through EAX: three rows of four values, where row c (c = 0..2) holds sum over k of p1[j][k] * p2[k][c] for column j = 0..3, using the 4x4 matr… |
| `00f25dd0` | MultiplyMat4ByAffineOutB | speculative | Same body as MultiplyMat4ByAffineOut (00f25c40): writes the 12-float product through EAX. |
| `00ed1cc0` | — | unknown | Cdecl-style, six stack arguments (matrix pointer, second matrix pointer, inverse output pointer, a value, a float tolerance at arg5, and a repair flag at arg6)… |
| `00ed4240` | — | unknown | Thiscall with one float stack argument (the threshold), 276 bytes, returns a status-style word. Copies two 3-float vectors from the ECX pointer and two from th… |

### `lib.granny.mesh`

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ecf4c0` | EnsureExactOneNorm | confirmed | Thiscall-style helper, two stack arguments (record pointer and data pointer), 597 bytes. Reads the type at [record+0] and the count at [record+0xc]. Types 10 a… |
| `00ed3ee0` | VertexTangentStackAllocate | confirmed | Parameterless helper (object pointer in EAX), 150 bytes, returns bool. When the unit count of the allocator at this+0xdc0 (00f18170) is nonzero it logs through… |
| `00ed45e0` | TriangleTangentFramesBuild | confirmed | Cdecl-style, one stack argument (object pointer), returns 1 (0 on failure), 894 bytes. Asserts the triangle tangent stack at this+0xda0 is empty (00f18170); lo… |
| `00f0d660` | MeshBoneWeightsSetup | confirmed | Allocates an entry from the pool at this+0x42c (00f18180), runs the top-N normalizer 00f0d390, walks the data types for the object at this+0x418 (00efd3d0 with… |
| `00f0e2c0` | MeshDeformerRecordCreate | confirmed | Calls 00f31f20 with (param_1, param_2, param_3, param_4 == 1, local_1c, &local_28). If it returns nonzero it allocates a 0x28-byte record through the allocatio… |
| `00ed4d30` | TangentFramesBuildDriver | likely | Cdecl-style, one stack argument (object pointer), returns 1 on success, 93 bytes. Calls 00ed45e0 (TriangleTangentFramesBuild) and, if that succeeds, 00ed3ee0 (… |
| `00f0d390` | TopNWeightNormalize | likely | For N = this+0x4e0 slots and M = this+0x4d8 items with weights at this+0x4dc: keeps each item's N largest weights in a descending insertion-sorted list (weight… |
| `00ecf950` | VertexDataTransformGroups | speculative | Eight-argument function, 1200 bytes. Computes the stride with 00eea520(arg2) and, for arg1 vertices (guarded by arg1 > 0), walks one vertex through 00efd3d0 in… |
| `00ed00d0` | FindBoneWeightsOrIndices | speculative | Cdecl helper, one stack argument (table pointer), 79 bytes. Calls 00efc480(table, 0, 'BoneWeights', &out); if out is null it calls 00efc480(table, 0, 'BoneIndi… |
| `00ed0150` | DataTypeWalkAtStrideOffset | speculative | Cdecl, five stack arguments (the first four used in the call), 42 bytes. Computes eax = 00eea520(arg1) * arg3 + arg2 (imul with the arg3 stack slot, then add a… |
| `00ed3fd0` | TriadAddSignAlignedScaled | speculative | Fastcall: ECX = destination (nine floats, three 3-float vectors at +0, +3, +6), EDX = source (same layout), one float stack argument (the scale), 263 bytes. Fo… |
| `00ed4360` | StackUnitsAccumulateTriadFrames | speculative | Parameterless helper (object pointer in EAX), 633 bytes. Allocates count*36 bytes through the tracked allocator (01b9378c, line 0x12e, count from 00f18170 on t… |
| `00f0d170` | IndexedPoolFindOrAdd | speculative | Calls 00efd3d0 and 00efd420 for the element at this+0xd78/+0xd8c. Walks the ring at this+0x44c from the bucket at this+0xd5c+4*param_2; for each candidate call… |
| `00ed0250` | — | unknown | Cdecl, three stack arguments, 110 bytes. Builds a 0x40-byte local member descriptor: type dword 10 at +0, name pointer 'Normal' at +0xc, count 3 at +0x14, all … |
| `00ed4980` | — | unknown | Thiscall-style, one object pointer, returns 1 (0 on an allocation failure), 918 bytes. Reads the vertex data stride (00eea520 of 00f0d9c0 result) and the trian… |
| `00f0c700` | — | unknown | Walks the list at this+0x42c (count 00f18170, elements 00f18210) and passes each element to 00efd420 with descriptor DAT_01b901d0 and object this+0x418. Calls … |
| `00f0cfa0` | — | unknown | Thiscall with EAX = object and stack param_2 = entry index. Computes the squared 3-vector deltas of the entry against the table at this+0xd8c and compares them… |

### `lib.granny.mirror`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f0e920` | MirrorSpecTransformsApply | confirmed | Validates the specification (param_1, 'Specification'), transforms (param_3, 'Transforms') and SourceParents (param_5, 'SourceParents'), and TransformCount (pa… |
| `00ed5610` | TransformArrayComposeByIndex | likely | Cdecl-style, seven stack arguments (source stride, source pointer, index stride, index array, count, destination stride, base pointer), 157 bytes. When the sou… |
| `00ed04f0` | MirrorFlipTriangleWindings | speculative | Thiscall-style helper with one stack argument (record pointer), 239 bytes. Three passes over arrays in the record: (1) the dword array at +0xc, count at +8: fo… |
| `00f0e840` | TableTransformPositionCopy | speculative | Thiscall with EAX = index and ESI = destination record. Composes a transform from the table entry at DAT_018704e8 + index*0x44 (00eeb220), copies the three com… |
| `00f0e8c0` | FramePolarSplit | speculative | Thiscall with EBX = frame record. Calls 00f09de0 with the 3x3 at EBX+0x20 and epsilon DAT_018212d0 to get the orthogonal factor R and the symmetric factor S = … |

### `lib.granny.model`

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00eec600` | ModelInstanceRelease | likely | Releases a model instance at param_1 when non-null: drains the list at param_1+0x10 through 00eebbd0, unlinks it from its neighbours (+0x54/+0x58 links), frees… |
| `00eebab0` | NotifySlot0C | speculative | Calls the function pointer at slot +0x0c of the table pointed to by param_1+0x18, passing param_1 and the five further arguments (six in total), if the pointer… |
| `00eebaf0` | NotifySlot10 | speculative | Calls the function pointer at slot +0x10 of the table pointed to by param_1+0x18, passing param_1 and four further arguments (five in total), if the pointer is… |
| `00eebb20` | NotifySlot14 | speculative | Calls the function pointer at slot +0x14 of the table pointed to by param_1+0x18, passing param_1 and five further arguments (six in total), if the pointer is … |
| `00eebea0` | AdvanceEntryTimes | speculative | Walks the list at param_1+0x24 until the sentinel at param_1+0x10: for each entry (owner = dword 0) it adds (param_2 - owner+0x44) to owner+0x48 and stores par… |
| `00eebf00` | DispatchWhilePredicate | speculative | Walks the list at param_1+0x24 until the sentinel at param_1+0x10. While the predicate 00ef6150(entry dword 0) is nonzero it calls 00eebab0(entry, param_2..par… |
| `00eebf60` | IncrementThenDispatch | speculative | Calls 00ed6cb0(param_4, param_2, param_3, param_6), then 00eebf00(param_1, ...) to dispatch the list, then 00ed6ec0(param_4, param_2, param_3, dword(param_1+4)… |
| `00eec1d0` | DispatchListSlot14WhilePredicate | speculative | Same walk as 00eebf00 but calls 00eebb20(entry, param_2..param_6) (slot +0x14) for each entry while the predicate 00ef6150(entry dword 0) is nonzero. |
| `00eec230` | NormalizeAccumulatedPair | speculative | Zeroes two vec3 outputs (param_3, param_4), accumulates a scalar into a local through the list dispatch 00eec1d0(param_1, param_2, &local, param_3, param_4, pa… |
| `00eec060` | — | unknown | Samples the model instance at param_1 into the buffer at param_4 (asserts it non-null, 'the &Scratch argument'). The bone count param_2 is clamped to the model… |
| `00eec660` | — | unknown | Creates a model instance for the model at param_1 (returns 0 and asserts 'Model.Skeleton' when the model's skeleton at +4 is null). Takes a block from pool 01b… |

### `lib.granny.sampler`

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f21940` | TrackGroupSampler_GetTrackPositionSamples | confirmed | Bounds-checks p2 against [p1+4]; out of range it reports through 00f07e00 (level 3, line 0xc0) and returns 0. In range it returns the pointer at [[p1+0xc]+p2*4… |
| `00f219b0` | TrackGroupSampler_GetTrackOrientationSamples | confirmed | Bounds-checks p2 against [p1+4]; out of range it reports through 00f07e00 (level 3, line 0xcb) and returns 0. In range it returns the pointer at [[p1+0x10]+p2*… |
| `00f21a20` | TrackGroupSampler_GetTrackScaleShearSamples | confirmed | Bounds-checks p2 against [p1+4]; out of range it reports through 00f07e00 (level 3, line 0xd6) and returns 0. In range it returns the pointer at [[p1+0x14]+p2*… |
| `00f21a90` | TrackGroupSampler_ReadSample | likely | Fetches the position, orientation and scale/shear arrays of track p2 via 00f21940, 00f219b0 and 00f21a20. For sample index p3 it copies 3 dwords from the posit… |
| `00f21b20` | TrackGroupSampler_WriteSample | likely | Mirror of ReadSample: fetches the three sample arrays of track p2 via 00f21940, 00f219b0 and 00f21a20, then stores p4 (3 dwords) into the position array at ind… |
| `00ed1a80` | TransformRecordFromThreeTracks | speculative | Cdecl-style, two stack arguments (record pointer, output pointer), 113 bytes. Calls 00ee3820 three times on the record at +0x10, +8 and +0x18 with index 0 and … |
| `00ef5500` | IndexEdgeFlags | speculative | Writes two byte flags through param_2 and param_3. When flag bit 5 of +0x40 is set both are 0; when the count at +0x5c is zero both are 1; otherwise the first … |
| `00ef5f40` | CursorAdvanceAndWrap | speculative | If the value at +0x48 is zero it returns the time at +0x4c. Otherwise it sets the time to +0x50 * +0x48 + +0x4c, and when that leaves the span it re-indexes th… |
| `00ef60f0` | AdvanceAndEdgeFlags | speculative | Advances the cursor through 00ef5f40 and then writes the two edge flags as 00ef5500 does: both 0 when +0x40 bit 5 is set, both 1 when the count is zero, otherw… |
| `00ef6190` | ReturnFloatAt54 | speculative | Returns the float at +0x54 of its argument as an x87 value (fld dword [eax+0x54]; ret). |
| `00ef6460` | CursorTimeClampToStep | speculative | Advances the cursor through 00ef5f40 and returns its result clamped to [0, the float at +0x54]: 0 when the result is negative, the step value when the result e… |
| `00ef5560` | — | unknown | Computes the index as +0x58 plus ROUND(+0x4c / +0x54), minus one when +0x4c is negative, clamps it into [0, count-1] when the count at +0x5c is nonzero, and wh… |

### `lib.granny.skeleton`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed5810` | SkeletonBoneCountForLodError | likely | Arg1 (edi) is the skeleton pointer (null-tested, assert via 00f07e00 with the granny_skeleton.cpp line, returns 0); arg2 is a float threshold (fld dword [esp+0… |
| `00f15e40` | BoneArraySortByKeyDesc | speculative | Sorts an array of count = *(param_1+4) bone records of 0x98 bytes (38 dwords) by the float key at record+0x8c, into descending key order, using an odd-even com… |

### `lib.granny.texture`

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ed32c0` | TextureExtractMipImage | confirmed | Cdecl-style, eight stack arguments, 666 bytes. Validates the image index (arg2 against the image count at +0x3c; logs 'ImageIndex' out of range) and the mip in… |
| `00f1d810` | TextureBuilderAlignedStride | confirmed | Computes the aligned row stride for a texture build: with the builder in EAX, returns AlignUp(param_1 * builder+0x2c (BytesPerPixel), builder+0x54 (RawStrideAl… |
| `00f112c0` | ColorExpandTablesInit | likely | One-time table setup guarded by DAT_01c004f8. Fills DAT_01bff678[k] = 17k and DAT_01bff6b8[k] = 17k << 24 for k = 0..15; DAT_01c00478[i] = (i>>2 \| i*8) << 16 … |
| `00f14f50` | Dxt1DecodeToArgb | likely | Calls 00f112c0 (table init), then decodes a grid of 8-byte blocks (two 565 colours at +0 and +2, 32-bit index word at +4) into ARGB dwords. Each colour is expa… |
| `00f15280` | Dxt3DecodeToArgb | likely | Calls 00f112c0, then decodes 16-byte blocks: an 8-byte explicit alpha section (sixteen 4-bit values, expanded through DAT_01bff6b8 to the top byte) followed by… |
| `00f156b0` | Dxt5DecodeToArgb | likely | Calls 00f112c0, then decodes 16-byte blocks. Alpha: two 8-bit endpoints (alpha0 at +0, alpha1 at +1) and 48 bits of 3-bit indices (+2..+7). If alpha0 > alpha1 … |
| `00f113d0` | BlockEndpointPairSearch | speculative | Chooses the best pair of palette indices for a block. It reads the candidate range at param_4 ([0] = low, [1] = high, [2] = high-1, [3] = limit taken from EAX … |
| `00f115d0` | BlockColorCostWeighted | speculative | Returns the weighted sum of block colour error over param_3 texel groups (4 bytes each, read at param_1 + 2 advancing by 4). For each group it evaluates four c… |
| `00f1d870` | TextureRowPackSetup | speculative | Calls 00f1d810 for a stride from its first stack argument, stores that result in the record at ESI (dword 0), stores its stack argument at entry offset +0x30 i… |
| `00f131e0` | — | unknown | Encodes one 4x4 single-channel block (bytes read at param_2 with row stride param_3, four rows) into an 8-byte alpha block at param_1. It collects the distinct… |

### `lib.granny.util`

**(no class)** (55)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f1b420` | StringCopyWithNul | confirmed | Copies the NUL-terminated string at param_1 to param_2 including the terminator and returns the number of bytes copied (length plus one). A null argument repor… |
| `00f1b0e0` | NullSafeStrLen | likely | Returns the number of characters before the terminating NUL of param_1, or 0 when param_1 is null. |
| `00f1b100` | StrEqualNullSafe | likely | Byte-by-byte equality of two NUL-terminated strings. Both null is equal; one null is unequal. Stops at a mismatch or a NUL in either string and returns whether… |
| `00f1b1b0` | StrCompareNullSafe | likely | Ordering compare of two NUL-terminated strings: when param_1 is null returns -1 if param_2 is not null, else 0; when only param_2 is null returns 1; otherwise … |
| `00f1b2a0` | StrCompareHookDispatch | likely | If the global hook pointer DAT_01c0054c is set it tail-jumps to it with the caller's arguments unchanged; otherwise it tail-jumps to 00f1b1b0 (StrCompareNullSa… |
| `00f1b2b0` | StrEqualHookable | likely | When the hook DAT_01c0054c is set it calls the hook with the two arguments (cdecl), and returns 1 when the hook returns 0 (equal). Otherwise it tail-jumps to 0… |
| `00f1b600` | CharClassIsDigit | likely | Returns 1 when bit 0x40 of the byte table at DAT_01b938b0 is set for the character param_1 (movzx of the argument, table lookup, mask 0x40, compare), else 0. |
| `00f1b6e0` | StrNEqualFastcall | likely | __fastcall: ECX = first string, EDX = second string, EAX = maximum count. Compares up to EAX characters, stopping at a NUL in either string. Returns true when … |
| `00f1b830` | PatternIsNullOrStar | likely | Returns 1 when the pattern pointer is null or the pattern is exactly the one-character string '*', otherwise 0. |
| `00f1b940` | StrEqualCaseInsensitive | likely | Case-insensitive equality of two NUL-terminated strings. ASCII upper-case letters are folded to lower case (add 0x20 when the byte is in the A-Z range), compar… |
| `00f1baa0` | StrEqualCaseHookable | likely | When hook DAT_01c0054c is set it calls the hook with the two arguments and returns 1 when the hook returns 0; otherwise it returns the result of 00f1b940 (case… |
| `00f1bb10` | AsciiToDouble | likely | Parses a number from the NUL-terminated text at param_1: an optional '-' (sign -1) or '+', then digits (digit class from table DAT_01b938b0, bit 0x40) accumula… |
| `00f1bbf0` | AsciiToIntRound | likely | Calls 00f1bb10 with param_2 = 0 (no fraction) and converts the ST0 result to a 32-bit integer with fistp, which rounds to nearest under the FPU control word. R… |
| `00f1bc30` | AsciiToIntRoundViaFloat | likely | Calls 00f1bb10 with param_2 = 0, stores the result as a 32-bit float (fstp dword), reloads it and converts to a 32-bit integer with fistp (round to nearest). R… |
| `00ecb0f0` | GrannyAssertWrapper_Flag0 | speculative | Cdecl-style wrapper (44 bytes): loads five stack arguments as float/dword values, pushes a constant 0 as a sixth argument and calls 00ed81a0 with those argumen… |
| `00ed3b70` | StackUnitsNormalizeTriads | speculative | Parameterless helper (object pointer in EAX), 304 bytes. Gets the unit count of the allocator at this+0x42c (00f18170) and, for each unit index, takes the unit… |
| `00ed3ca0` | StackUnitsRebuildTriadFrame | speculative | Parameterless helper (object pointer in EAX), 570 bytes. Gets the unit count of the allocator at this+0x42c (00f18170) and, for each unit (00f18210 on the allo… |
| `00ed52c0` | StringListCrc32Hash | speculative | Cdecl-style, one stack argument (a record: count at +0, string pointer array at +4, CRC dword at +8), 99 bytes. Sets the CRC dword to 0xffffffff (00f25670), th… |
| `00ef5690` | ReturnFirstArg | speculative | Returns its first argument unchanged (mov eax,[esp+4]; ret, five bytes). |
| `00ef5ab0` | ReturnFloatAt50 | speculative | Returns the float at +0x50 of its argument as an x87 value (fld dword [eax+0x50]; ret). |
| `00ef65a0` | FlagsFromByteTest | speculative | Calls 00ed2f60 with two stack output slots; returns 1 when bit 0 of the first output byte is set, otherwise (byte >> 1) & 2. |
| `00f15c40` | BoneSubtreeMaxReach | speculative | Walks a table of count = *(this+4) nodes with 0x8c-byte records. Allocates a float array (one per node) from an arena (00f0b140, 00f0b200) initialised to DAT_0… |
| `00f1b500` | StringCopyAdvanceBounded | speculative | Copies characters from param_1 to the cursor at *param_2 while the remaining count at *param_3 is positive. Each copied character decrements the count. On the … |
| `00f1b9a0` | PathStrEqualCaseInsensitive | speculative | Case-insensitive comparison of two NUL-terminated strings in which '\' and '/' are treated as equal. Stops at a mismatch or a NUL and returns whether the stop … |
| `00f21660` | DwordArrayFillCount | speculative | Stores the dword param_2 into every element of the array at +0x18 for count +0x14 elements (no effect when the count is not positive). |
| `00f21680` | FillFloatArrayWith7f0fffff | speculative | Loops i from 0 to [p1+0x14]-1 and stores the dword 0x7f0fffff (float about 1.8e38) into the array at [p1+0x18][i]. No return value. |
| `00f22850` | CallEf6490AtOffset10 | speculative | Pushes p6, p5 and p3+0x10 and calls 00ef6490 (cdecl), cleans 0xc bytes and returns. |
| `00f229e0` | ReturnAddr00f228d0 | speculative | Returns the constant address 0x00f228d0 (mov eax, imm32; ret) and does nothing else. |
| `00f22a00` | ReturnAddr00f22790 | speculative | Returns the address of 00f22790 (mov eax, 0xf22790; ret) and does nothing else. |
| `00f23280` | ReturnDwordAtArg1 | speculative | mov eax,[esp+4]; mov eax,[eax]; ret: returns the dword stored at the address passed as its first argument. |
| `00f234d0` | StrideAt64IfBelowCount | speculative | Returns 0 when p2 is not below [p1]; otherwise returns [p1+4] + p2*0x40 (the address of entry p2 in a 64-byte-stride array). |
| `00f234f0` | StrideAt64IfField8NonNull | speculative | Returns 0 unless [p1+8] is nonzero and p2 is below [p1]; then returns [p1+8] + p2*0x40. |
| `00f23520` | ReturnDwordAtArg1Plus8 | speculative | mov eax,[esp+4]; mov eax,[eax+8]; ret: returns the dword at the first argument plus 8. |
| `00f25600` | RangeInitFirstCount | speculative | Sets [0]=0 and [1]=p2 when p2 is at least 1, else [1]=0. |
| `00f25620` | RangeNonEmpty | speculative | Returns 1 when dword +4 of p1 is greater than zero, else 0. |
| `00f25630` | RangeMidpoint | speculative | Returns [0] + [1]/2 (signed division of the count by two plus the first index). |
| `00f25640` | RangeSetCountFromEnd | speculative | Sets [1] = p2 - [0] (count from the end index p2 and the first index). |
| `00f25650` | RangeNarrowAbovePivot | speculative | Sets [1] = [1] + ([0] - p2) - 1 and then [0] = p2 + 1. |
| `00f25670` | SetDwordMinusOne | speculative | Stores 0xffffffff at the dword p1. |
| `00f25680` | Crc32UpdateTable | speculative | Updates the 32-bit CRC at p1 over the byte run given by the 64-bit count (p2 low, p3 high) at p4, using the reflected CRC-32 table at DAT_01b93a70 (entry 1 = 0… |
| `00f256d0` | InvertDword | speculative | Replaces the dword at p1 with its bitwise complement. |
| `00f25720` | FillDwordsValue | speculative | Stores dword p2 into p1 consecutive dwords starting at p3; no-op when p1 is zero. |
| `00f25740` | FillDwordsValueB | speculative | Same body as FillDwordsValue (00f25720): stores dword p2 into p1 consecutive dwords starting at p3. |
| `00f25760` | ZeroDwords | speculative | Stores zero into p1 consecutive dwords starting at p2; no-op when p1 is zero. |
| `00f25780` | MemMoveOverlapSafe | speculative | Copies p1 bytes from p2 to p3 and returns p3. It copies backwards when p3 lies inside [p2, p2+p1) and forwards otherwise. |
| `00f257e0` | MemEqualN | speculative | Compares p1 bytes at p2 with p1 bytes at p3 and returns 1 when all are equal, else 0 at the first mismatch. |
| `00f25820` | AlignUpToMultiple | speculative | Returns p1 rounded up to a multiple of p2 (p2 must be nonzero): uVar1 = p2 - p1 % p2; when uVar1 is not p2 the result is p1 + uVar1. |
| `00f25840` | AlignUpToMultipleB | speculative | Same round-up as AlignUpToMultiple (00f25820), returning the rounded value directly. |
| `00f258c0` | ByteSwap32 | speculative | Returns the dword p1 with its four bytes reversed. |
| `00f259b0` | ByteSwapDwords | speculative | Byte-reverses p1 >> 2 dwords in place starting at p2. |
| `00f25a00` | ByteSwapHalfwords | speculative | If (p1 >> 1) is even it swaps the two bytes inside each 16-bit half of p1 >> 2 dwords; otherwise it byte-swaps p1 >> 1 16-bit words starting at p2. |
| `00ecf320` | — | unknown | Thiscall with in_EAX = vertex count, five stack arguments (dst, dst stride, src base, src stride, member descriptor array) and a member count, 112 bytes. For e… |
| `00ed3af0` | — | unknown | Cdecl, one stack argument (the object pointer), 126 bytes. Gets the unit count of the allocator at this+0x42c (00f18170) and loops over the units: for each ind… |
| `00f1b2e0` | — | unknown | Copies the string at param_3 into the buffer param_1 with capacity param_2, keeping one byte for the NUL. Copies until the source NUL or until one byte of capa… |
| `00f1b730` | — | unknown | Matches the name at param_1 against the pattern at param_2 using '?' (one character), '*' (any run, with the next literal segment searched by 00f1b6e0) and ';'… |

### `lib.granny.variant`

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f22ba0` | VariantBuilderAppend | confirmed | Takes EAX = an 8-dword header (copied into the new node) and p1 = payload. Obtains the payload size from 00eeaa40 (called with ESI = header pointer), allocates… |
| `00f22d30` | AppendTag13FromStack | speculative | Calls 00f1ccc0(*p1, p2) and then builds a stack header with first dword 0x13, pushes an address from the wrapper's own frame as payload and calls 00f22ba0 with… |
| `00f22d90` | AppendTag13FromArg4B | speculative | Same shape as AppendTag13FromArg4 (00f22cd0): calls 00f1ccc0(*p1, p2) and then 00f22ba0 with a header whose first dword is 0x13 and p4 as payload. |
| `00f22df0` | AppendTag14FromStack | speculative | Calls 00f1ccc0(*p1, p2), then builds a stack header with first dword 0x14 and calls 00f22ba0 with EAX = header and a stack-frame address as payload. |
| `00f22e50` | AppendTag0aFromStack | speculative | Calls 00f1ccc0(*p1, p2), then builds a stack header with first dword 0x0a and calls 00f22ba0 with EAX = header and a stack-frame address as payload. |
| `00f22eb0` | AppendTag0aFromArg4 | speculative | Calls 00f1ccc0(*p1, p2), then builds a stack header with first dword 0x0a and calls 00f22ba0 with EAX = header and p4 as payload. |
| `00f22f80` | AppendTag02FromStack | speculative | Calls 00f1ccc0(*p1, p2), then builds a stack header with first dword 2 and calls 00f22ba0 with EAX = header and a stack-frame address as payload. |
| `00f22cd0` | — | unknown | Calls 00f1ccc0(*p1, p2), then builds a stack header whose first dword is 0x13 and calls 00f22ba0 with EAX = that header and p4 as payload. The 00f1ccc0 result … |
| `00f22f10` | — | unknown | Calls 00f1ccc0(*p1, p2) and then 00f1ccc0(*p1, p3); the second result is stored into the frame and passed by address to 00f22ba0 as payload, with EAX = a stack… |
| `00f22fe0` | — | unknown | Calls 00f1ccc0(*p1, p2), zeroes a 5-dword block, sets a stack header with first dword 3 and fields from p4 and p5, and calls 00f22ba0 with EAX = header and a s… |

### `lib.granny.worldpose`

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f232e0` | WorldPoseAggregateSize | confirmed | Starts an aggregate layout (00f24cd0, granny_world_pose.cpp line 0x85), adds a 0xc-byte header (00f24ef0), aligns to 0x10 (00f24cf0), adds p1 entries of 0x40 b… |
| `00f233c0` | WorldPoseAllocAggregate | confirmed | Lays out a world-pose block like 00f232e0: begin (line 0x99), 0xc-byte header whose pointer is stored into the result, 16-byte alignment, p1 entries of 0x40 by… |
| `00f236a0` | WorldPoseCompositeBoneRange | confirmed | If p4[2] (CompositeTransformBuffer) is NULL reports Result.CompositeTransformBuffer is not allowed to be NULL (level 3, line 500) and returns. Otherwise checks… |
| `00f231b0` | WorldPoseFreeBlock | likely | Pushes p1, the line number 0x3d (61) and the path of granny_world_pose.cpp, then calls the hook at 0x01b93790 with (file, line, p1) and cleans 0xc bytes. |
| `00f23cc0` | WorldPoseEvalSubtree | likely | Checks p2 (ParentBone) against [p1+4] (reports line 0x2b6 level 3 when out of range). Otherwise increments the counter at p3+0xc and uses it as the mark for th… |
| `00f23f10` | WorldPoseFillThenComposite | likely | Calls 00f23530 with the eight arguments it received, then calls 00f236a0(p1, p2, p3, p8) when p8[2] (the composite buffer) is nonzero. |
| `00f23f60` | WorldPoseFillRangeWithOverrides | likely | Checks p2+p3 against [p1+4] (reports line 0x176, or 0x177 against the count [p8], level 3) then for bones p2..p2+p3-1 calls 00ef6490(record, source, dest). The… |
| `00f240c0` | WorldPoseFillRangeSimple | likely | Checks p2+p3 against [p1+4] (reports line 0x1be, or 0x1bf against [p5], level 3). For bones p2..p2+p3-1 it calls 00ef6490(bone+8 record, source, dest), dest = … |
| `00f241f0` | WorldPoseFillThenCompositeAlt | speculative | Calls 00f23530 with (p1, p2, p3, p2, p3, p4, p5, p6) and then 00f236a0(p1, p2, p3, p6) when p6[2] is nonzero. Same flow as 00f23f10 with the bone range passed … |
| `00f23530` | — | unknown | Checks the local bone range (p4+p5, against the count read by 00ed6b70 from p6, reports OnePastLastValidLocalBone line 0x12f) and the bone range (p2+p3 against… |
| `00f23e80` | — | unknown | Sizes a world pose for p1 bones with 00f232e0(p1, 0), which includes the composite buffer. It then calls the allocation hook at 0x01b9378c with (granny_world_p… |

## Source shards

[[S0164|S0164]] (16), [[S0165|S0165]] (1), [[S0167|S0167]] (6), [[S0168|S0168]] (39), [[S0169 00787e70-0078b5f0|S0169]] (10), [[S0170|S0170]] (4), [[S0173|S0173]] (13), [[S0174|S0174]] (30), [[S0175|S0175]] (9), [[S0177|S0177]] (58), [[S0179|S0179]] (14), [[S0185|S0185]] (5), [[S0186|S0186]] (11), [[S0252|S0252]] (21), [[S0253|S0253]] (45), [[S0255 0095a840-00960520|S0255]] (1), [[S0257|S0257]] (2), [[S0258 00969a10-0096de80|S0258]] (4), [[S0259 0096e490-00972d30|S0259]] (1), [[S0304 00a6c630-00a70790|S0304]] (29), [[S0305 00a707f0-00a75b20|S0305]] (1), [[S0426 00cfc4e0-00d01c70|S0426]] (1), [[S0510|S0510]] (66), [[S0511|S0511]] (59), [[S0512|S0512]] (45), [[S0513|S0513]] (87), [[S0514|S0514]] (73), [[S0515|S0515]] (71), [[S0517|S0517]] (87), [[S0518|S0518]] (71), [[S0519|S0519]] (119), [[S0587|S0587]] (2), [[S0588|S0588]] (14), [[S0589|S0589]] (6), [[S0590|S0590]] (5)
