---
type: build-guide
section: Libraries and Runtime
updated: 2026-10-12
---

# Unclassified

Part of [[Build Guides]] › Libraries and Runtime.

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

3425 accepted functions: 28 confirmed, 535 likely, 1847 speculative, 1015 unknown. Network-linked (`wire`): 0.

Subsystems: `unknown` (2104), `unknown.unclassified` (860), `unknown.accessor` (113), `unknown.field_accessor` (91), `unknown.math` (34), `unknown.wrapper` (29), `unknown.dtor` (18), `unknown.queue` (15), `unknown.locked_accessor` (13), `unknown.global` (12), `unknown.cleanup` (10), `unknown.tree` (10), `unknown.userdata_command` (9), `unknown.field` (9), `unknown.timewindow` (8), `unknown.dispatch` (7), `unknown.locked_state` (6), `unknown.struct` (6), `unknown.class_dtor` (6), `unknown.granny` (6), `unknown.list` (5), `unknown.fixup` (5), `unknown.compare` (3), `unknown.array` (3), `unknown.dag2` (3), `unknown.layout` (3), `unknown.spawn` (2), `unknown.heap` (2), `unknown.datetime` (2), `unknown.getter` (2), `unknown.pool` (2), `unknown.bitfield` (2), `unknown.string` (2), `unknown.intmap` (2), `unknown.globalobj` (2), `unknown.guid_lookup` (1), `unknown.parse` (1), `unknown.seh` (1), `unknown.memory` (1), `unknown.subobject` (1), `unknown.counter` (1), `unknown.blend` (1), `unknown.table` (1), `unknown.dag` (1), `unknown.anim` (1), `unknown.data` (1), `unknown.forward` (1), `unknown.global_reset` (1), `unknown.thunk` (1), `unknown.deck` (1), `unknown.synchcommand` (1), `unknown.timer` (1), `unknown.indexed_slot` (1), `unknown.cached_ref` (1)

_Large or library guide: only confirmed and likely functions are listed (563 of 3425)._

### `unknown`

**ActorHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072f0c0` | ActorHandler::ScalarDeletingDtor | confirmed | Calls 00798db0 on this, then frees this with _free when bit 0 of param_2 is set. Returns this. |

**BaseAttachmentGroup** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0072f560` | BaseAttachmentGroup::CheckHasEntries | confirmed | Returns 0 after logging 'BaseAttachmentGroup has no entries!' through the logger object at this+4 (+0xe4, then vtable slot [5]) when the count at +0x20 is belo… |
| `0072fb60` | BaseAttachmentGroup::SetChildHandleState | confirmed | Takes one stack argument and finds the child whose slot-[4] handle equals it (0072f920). When found, sets child+0x180 to 2 and, when the argument is non-null, … |
| `00730380` | BaseAttachmentGroup::FailedGroupLoad | confirmed | Sets this+0x65 to 1 and starts a message with 'BaseAttachmentGroup::FailedGroupLoad - message:' (0070bd80). When param_2 is non-null it appends ' Actor name: %… |
| `00730630` | BaseAttachmentGroup::ValidateLoadResult | confirmed | Calls vtable slot 21 (+0x54) of *this; when that is non-zero and the slot [5] (+0x14) call on param_2 returns 0 it returns. Otherwise it logs 'Invalid ActorGro… |
| `00730f10` | BaseAttachmentGroup::AppendEntry | confirmed | Virtual slot [6]: builds a local list object around param_2 and calls 00730990 on it with the second argument; when param_4 is positive it releases the IString… |
| `00731c50` | BaseAttachmentGroup::DtorBody | confirmed | Installs BaseAttachmentGroup::vftable, walks the entry list at this+0x18 (next link +0x190) calling each entry's owner slot [4] with entry+0x184 (and 0076b940 … |
| `00731d30` | BaseAttachmentGroup::GetFlag67 | confirmed | Returns the byte at this+0x67 (mov al,[ecx+0x67]; ret). |
| `00731e30` | BaseAttachmentGroup::BaseAttachmentGroup | confirmed | Installs ActorHandler::vftable and then BaseAttachmentGroup::vftable at *this, stores param_2 at +4 and param_3 at +8, zeroes the dword at +0xc and the bytes a… |
| `00731ee0` | BaseAttachmentGroup::ScalarDeletingDtor | confirmed | Calls 00731c50 on this, then frees this with _free when bit 0 of param_2 is set. Returns this. |

**SoeUtil::Array<RailDefinition::PointData,0,1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010713f0` | RailDefinitionPointArrayCtor | confirmed | Zeroes dwords +0x0, +0x4 and +0xc and bytes +0x8 and +0x9 of the object, installs the SoeUtil::Array<RailDefinition::PointData,0,1> vtable at +0x10, and zeroes… |

**(no class)** (433)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007822b0` | CtorBody_FiveIStringSeatArray_007822b0 | confirmed | Fastcall (ECX=this): stores the dword 1 at +0, then constructs five IString<char> members at +4, +0x14, +0x24, +0x34 and +0x44 (vtable, shared empty buffer DAT… |
| `00782370` | DtorBody_FiveIStringSeatArray_00782370 | confirmed | Destructor body (fastcall, this in ECX) for the record built by 007822b0: releases the SoeUtil::Array<ActorDefinition::MountData::Seat,0,1> at +0x54 (element d… |
| `008ae070` | DtorBody_LoggableAt8 | confirmed | Byte-verified destructor body: inside an SEH frame it stores Loggable::vftable into the dword at this+8 (lea eax,[ecx+8]; mov dword ptr [eax],0x017e8c54) and r… |
| `008ca8a0` | DtorRecord_RefPtr10_IString0 | confirmed | If pointer this+0x10 is nonnull, decrements its refcount at pointer+4; when the new count is zero, calls that object vslot +4 with no stack arguments, then cle… |
| `01062540` | InitNodeStringsAndTimer | confirmed | Fastcall with the node in ECX (no stack arguments, 213 bytes). Stores two dwords from globals at +0 and +4, zeroes the bytes at +8, +9, +0x10..+0x15 and the wo… |
| `01062620` | NodeStringsDtorBody | confirmed | Fastcall with the node in ECX. Stores SoeUtil::IString<char>::vftable at +0x40, releases the string at +0x44 when its length word at +0x4c is positive (decreme… |
| `01062dd0` | InitNodeThreeStringsAndWord | confirmed | Thiscall with one stack argument (ret 4). Zeroes the words at +4, +8 and +0xc, zeroes the 0x50 bytes from +0x10 with _memset, zeroes the word at +0. Stores Soe… |
| `005448f7` | AdjustSegmentElementRanges | likely | Walks param_2 records of 0x10 bytes based at param_1 (each record: pointer to a 0x1c-stride element vector at +4, count at +8, offset at +0xc). For every recor… |
| `00547127` | RebaseRangesAtThisPosition | likely | Thiscall (ECX=this) with two stack arguments, ret 8. It zeroes a 12-byte scratch and calls the range worker through the thunk 00547122 (= 005448f7) with (recor… |
| `005471a7` | RebaseRangesFromVectorValue | likely | Helper with three stack arguments (ret). It measures the container param_1 with the labeled VectorSize16 (0048295a); when the container is non-empty it builds … |
| `00708660` | Call00952ba0OnField22034 | likely | Copies its two stack arguments into an 8-byte scratch area, loads ECX from [this+0x22034], calls 00952ba0 with that object context and scratch arguments, then … |
| `0070d290` | CopyRecordAndStringMembers | likely | Copies two leading dwords, five subrecords through 006f9e60, and a byte at effective offset +0xa4 (0x29 dwords) from source to destination. |
| `0070d300` | FilterRowsByStringFields | likely | Requests row index 2 from the interface at this+0x2226c+0x10 (receiver in ECX, explicit argument 2), then iterates subsequent rows. It builds each row string, … |
| `0070e3f0` | CopyThreeEmbeddedRecords | likely | With destination in ECX, initializes/copies embedded string members from source argument offsets +0, +0x94, and +0x128 into the corresponding destination membe… |
| `0071b000` | CountMatchingRowsBeforeIndexRoundedDownBy9 | likely | Iterates rows from the object at this+0x10, counts rows matching two stored fields and a category expression until the selected field matches, then returns the… |
| `0071b1b0` | CopyMatchedRecordFields | likely | Scans records from the object at this+0x10 until two fields match the stored values, then copies eight dwords from record offsets 0xb8 through 0xd8 into paired… |
| `0071b2b0` | CountRowsMatchingFields | likely | Iterates rows from the object at this+0x10 and returns the count whose fields at offsets 0x1c and 0x20 match stored values and whose field at 0x18 plus one equ… |
| `0071b5e0` | InitializeFieldsAndGlobal | likely | Sets object fields at offsets 0x0c, 0x9fa0, 0x9fac, 8, 0x9f50, and 0x9f54; clears global DAT_01bdf5c8 and returns 1. |
| `0072f530` | Hash16FindNodeByKey | likely | Looks up the chain for bucket (param_2 & 0xf) at this+0x24 (next link +0x19c) and returns the first node whose key at +0x198 equals param_2, or 0 when none mat… |
| `0072f970` | Hash16ContainsKey | likely | Searches the 16-bucket chain at this+0x24 (bucket param_2 & 0xf, key at +0x198, next link +0x19c) for param_2 and returns 1 when found, otherwise 0. |
| `007307d0` | Hash16_UnlinkAndFreeNode | likely | Unlinks node param_2 from its 16-bucket chain (bucket key(+0x198) & 0xf at this+0x14, chain link +0x19c) and from the list (prev link +0x194, next link +0x190;… |
| `007308c0` | Hash16List_InsertNewNode | likely | Allocates a node through vtable slot [1] (+4) of *this; when non-null constructs it with 007306e0 from the arguments, appends it at the tail (next link +0x190,… |
| `00730a40` | Hash16_RemoveNodeByKey | likely | Finds the node whose key at +0x198 equals param_2 in bucket (param_2 & 0xf) at this+0x24 (chain link +0x19c). When found it calls vtable slot [2] (+8) of the n… |
| `00769240` | GlobalObjectCtor | likely | fastcall with ECX = object (its size is 0x1f0 per the allocation in 00769710). Zeroes +0x40, copies 16 dwords (0x40 bytes) from the table at DAT_01bc2660 into … |
| `0076b9d0` | TimedTransitionSetup | likely | thiscall with ECX = transition state (flags dword at +4, latched fields at +0x10/+0x14/+0x1c, +0x20/+0x24/+0x2c, duration at +0xc, +8 start flag), and four arg… |
| `0076bab0` | CopyTransitionStateToOwner | likely | thiscall with ECX = source transition state (param_1) and one stack argument, the owner object (param_2). If the owner's +0x220 block is null, allocates 0x30 b… |
| `0076bd10` | GetTableNameOrUnknown | likely | thiscall with ECX = object and an index argument. Reads the table pointer at this+0x370; the count is its +0xc dword (0 when the table is null). If the index i… |
| `0076f2d0` | EnsureTransitionBlockThenInit | likely | thiscall with ECX = object and four dword arguments. If the pointer at this+0x220 is null, allocates 0x30 bytes through operator new (00d833a6); on success sto… |
| `0077f070` | DtorBody_0077f070 | likely | Destructor body with this in ECX (kept in ESI): destroys the fixed string at +0xec and the one at +0xb8 through 00703350 (fixed-string release), destroys the I… |
| `0077f160` | ThisForward3_0077b310 | likely | Thiscall with three stack arguments (ret 0xc): reads one dword through param_2, one dword through param_3 and one byte through param_4, then calls 0077b310 wit… |
| `0077f1d0` | SetDword30_ForwardB1c0 | likely | Thiscall with three stack arguments (ret 0xc): calls 0077b1c0 with ECX=this, pushing the dword at *param_3 and the float bits of *param_4 (copied through x87 f… |
| `0077f580` | RegisterObjectByKeyCc | likely | Thiscall with one stack argument (object pointer, ret 4): key = dword at object+0xcc; searches the bucket at this+0xbc+(key&1)*4 for a node with key at +0xc; i… |
| `0077f5d0` | CopyCtor_IString3_0077f5d0 | likely | Copy constructor (thiscall, one stack argument = source, ret 4): zeroes the leading dword and writes three embedded IString<char> headers at this+4, +0x14 and … |
| `0077f6a0` | CtorBody_AnimProcessedMapDerived_0077f6a0 | likely | Constructor body with this in ECX (kept in ESI): runs the base constructor 0077e270 (HashListMap<int,Actor::AnimProcessed,4,-1> body), constructs StringFixed<1… |
| `0077f790` | CopyCtorAndSetTwoFlags_0077f790 | likely | Thiscall with three stack arguments (ret 0xc): calls the copy constructor 0077f5d0 with param_2 as the source, then stores the bytes param_3 and param_4 at thi… |
| `0077ff30` | LoadFiveSlots_0077b710 | likely | Thiscall with no arguments: for index 0..4 reads the 16-byte element at this+0x2b0 + 16*index; passes its dword at +0xa8 when the count at +0xac is at least 1,… |
| `0077ff80` | UpdateStateAndPostEvents | likely | Fastcall with this in ECX: calls 007c6fb0 and 0077d0f0 (gates), sets this+0x2b0 = this+0x2ac+0x28 and, when both gates pass, updates flags from the element at … |
| `007802d0` | SetPendingEntryPayload_007802d0 | likely | Thiscall with three stack arguments (p2, p3 byte, p4 pointer): returns when the dword at p2+8 is zero. Otherwise stores byte p3 at this+0x238; if byte this+0x2… |
| `00781bc0` | CtorBody_WeightedEntryList_00781bc0 | likely | Fastcall (ECX=this): calls the List<WeightedEntry,0> constructor 0077e8d0 with this, then clears the dword at this+0x24 and returns this. |
| `00781ed0` | AllocConstructAndRegister_0077f1d0 | likely | Thiscall (ECX=factory object; three stack arguments). Allocates through the object's vtable slot [1] (+4) with no arguments; if non-null, calls 0077f1d0 with E… |
| `00781fe0` | CtorWrap_0077f6a0 | likely | Thiscall with one stack argument (pointer to a dword, ret 4): calls the constructor body 0077f6a0 on this, then stores the dword *param_2 at this+0x160 and ret… |
| `00782100` | ForwardToCopyCtorFlags_0077f790 | likely | Thiscall with three stack arguments (ret 0xc-style forwarder, returns this): calls 0077f790 with ECX=this, the dword param_2, and the bytes *param_3 and *param… |
| `00782550` | SetPendingFlagAndPayload_007802d0 | likely | Thiscall with two stack arguments (p2 object, p3 dword): when the dword at p2+8 is positive sets the byte at this+0x238 to 1; then calls 007802d0 with this, p2… |
| `007826e0` | StageListFlushAndDrain_007826e0 | likely | Fastcall (ECX=this): for each node of the list whose head is at this+0x4f0 (next link +0x30) calls 0077bf00 with the node and either 0 or node+0x10, chosen by … |
| `00782f20` | SetEnabledFlagMaybeNamed_00782f20 | likely | Thiscall with a byte p2 (enable) and a dword p3 (optional name pointer). When p2 is set it sets this+0x390 to 1, and when this+0x3b4 is zero and this+0x2b0 is … |
| `0078f940` | ScalarDeletingDtor_007cacf0 | likely | Scalar deleting destructor shape (ret 4): calls 007cacf0 with this in ECX, then _free(this) only when the flag bit 0 is set. Returns this in EAX. |
| `00790600` | DtorBody_00790600 | likely | Teardown body (ret 0, EH frame): stores the SoeUtil::IString<char> vtable at this+0x10 and, when the length at +0x1c is positive, releases the heap buffer at +… |
| `00790730` | ScalarDeletingDtor_007373b0 | likely | Scalar deleting destructor shape (ret 4): calls 007373b0 with this in ECX, then _free(this) only when flag bit 0 is set. Returns this in EAX. |
| `00790850` | ScalarDeletingDtor_00813230 | likely | Scalar deleting destructor shape (ret 4): calls 00813230 with this in ECX, then _free(this) only when flag bit 0 is set. Returns this in EAX. |
| `00790870` | ScalarDeletingDtor_007f6320 | likely | Scalar deleting destructor shape (ret 4): calls 007f6320 with this in ECX, then _free(this) only when flag bit 0 is set. Returns this in EAX. |
| `00790890` | ScalarDeletingDtor_010b24d0 | likely | Scalar deleting destructor shape (ret 4): calls 010b24d0 with this in ECX, then _free(this) only when flag bit 0 is set. Returns this in EAX. |
| `007913a0` | ScalarDeletingDtor_00d023c0 | likely | Scalar deleting destructor shape (ret 4): calls 00d023c0 with this in ECX, then _free(this) only when flag bit 0 is set. Returns this in EAX. |
| `007b63f0` | GetField480 | likely | Fastcall getter: bytes 8b 81 80 04 00 00 c3 (mov eax, [ecx+0x480]; ret); returns the dword at this+0x480. |
| `007b6400` | GetDword4OfArg | likely | Stdcall getter (ret 4): bytes 8b 44 24 04 8b 40 04 c2 04 00 (mov eax,[esp+4]; mov eax,[eax+4]; ret 4); returns the dword at offset +4 of its pointer argument. |
| `007b7420` | FreeThisIfFlagBit0A | likely | Thiscall with a flag byte: when bit 0 of the flag is set, calls _free(this) inside a frame whose handler label is LAB_0155eab3; returns this. No destructor bod… |
| `007b7480` | FreeThisIfFlagBit0B | likely | Thiscall with a flag byte: when bit 0 of the flag is set, calls _free(this) inside a frame whose handler label is LAB_0155ead8; returns this. No destructor bod… |
| `00893310` | GuardedStoreArgAt0xbec | likely | Enters the guard object at this+0x1058 (00fc45f0), stores the dword argument at this+0xbec, then leaves that guard (00fc4610); returns with ret 4. |
| `00893600` | GuardedStoreArgAt1050 | likely | Enters the guard at this+0x1058 (00fc45f0), stores its dword argument at this+0x1050, then leaves the guard (00fc4610); returns with ret 4. |
| `00893e20` | GuardedWaitStateTimeout | likely | Saves the TLS value and sets the TLS slot to this+0x1050, takes the start time from 007027f0, then while 00893340 returns 1 and the dword at this+0xcd10 is pos… |
| `008954b0` | GuardedAssignStringsReplaceMachine | likely | Enters the guard at this+0x1058 (00fc45f0), sets the TLS scope to this+0x1050, assigns three C-string arguments into string members (the first at ECX=ebp, the … |
| `0089ee70` | ClearEightDwordsAt0x358 | likely | Field-clearing helper: builds a local 8-dword block of zeros and copies it to param_1+0x358, i.e. it zeroes the 32 bytes starting at offset 0x358 of the object… |
| `0089f5f0` | SelectFloatByByte0x650 | likely | Returns one of two float constants through the x87 stack: 0.25 when the byte at +0x650 of the object is non-zero, otherwise 1.0 (the two .rdata dwords at 0x017… |
| `0089f890` | StoreFourDwords_0x18c0 | likely | Copies four consecutive dwords from the caller's structure (*param_2, param_2[1], param_2[2], param_2[3]) into this+0x18c0, +0x18c4, +0x18c8 and +0x18cc, i.e. … |
| `008a54d0` | CallVtblSlot10IfFlagClear | likely | Virtual-call shim: reads the byte at this+0x2c0; when it is zero calls FUN_00996a80 (no arguments), then always tail-jumps through the object vtable slot +0x10… |
| `008a5500` | NotifyIfField8Set | likely | Loads ECX from this+8; if non-null, tail-jumps to 0099a7a0 with that pointer as receiver; otherwise returns immediately. |
| `008a5930` | StoreRefAndSentinel | likely | Initialises a two-dword structure: stores the first argument at +0 and the sentinel 0xffffffff at +4, returns nothing; no calls. |
| `008a5c80` | ScalarDelete_983500 | likely | Compiler-generated scalar deleting destructor: calls FUN_00983500(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5ca0` | ScalarDelete_9c0450 | likely | Compiler-generated scalar deleting destructor: calls FUN_009c0450(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5cc0` | ScalarDelete_9c2cc0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009c2cc0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5ce0` | ScalarDelete_9c6ac0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009c6ac0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5d00` | ScalarDelete_9c85f0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009c85f0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5d20` | ScalarDelete_9c9b90 | likely | Compiler-generated scalar deleting destructor: calls FUN_009c9b90(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5d40` | ScalarDelete_9ccca0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009ccca0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5d60` | ScalarDelete_9ce180 | likely | Compiler-generated scalar deleting destructor: calls FUN_009ce180(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5d80` | ScalarDelete_9ce500 | likely | Compiler-generated scalar deleting destructor: calls FUN_009ce500(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5da0` | ScalarDelete_9cefe0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009cefe0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5dc0` | ScalarDelete_9cf340 | likely | Compiler-generated scalar deleting destructor: calls FUN_009cf340(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5de0` | ScalarDelete_9d22b0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009d22b0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5e00` | ScalarDelete_9d2eb0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009d2eb0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5e20` | ScalarDelete_9e4410 | likely | Compiler-generated scalar deleting destructor: calls FUN_009e4410(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5e40` | ScalarDelete_9f11a0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009f11a0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5e60` | ScalarDelete_9f5a70 | likely | Compiler-generated scalar deleting destructor: calls FUN_009f5a70(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5e80` | ScalarDelete_9856d0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009856d0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5ea0` | ScalarDelete_9f8370 | likely | Compiler-generated scalar deleting destructor: calls FUN_009f8370(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5ec0` | ScalarDelete_01024c70 | likely | Compiler-generated scalar deleting destructor: calls FUN_01024c70(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5ee0` | ScalarDelete_01026590 | likely | Compiler-generated scalar deleting destructor: calls FUN_01026590(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5f00` | ScalarDelete_9fc150 | likely | Compiler-generated scalar deleting destructor: calls FUN_009fc150(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5f20` | ScalarDelete_9fd850 | likely | Compiler-generated scalar deleting destructor: calls FUN_009fd850(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5f40` | ScalarDelete_9ff0c0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009ff0c0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5f60` | ScalarDelete_a03490 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a03490(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5f80` | ScalarDelete_a07500 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a07500(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5fa0` | ScalarDelete_01027ef0 | likely | Compiler-generated scalar deleting destructor: calls FUN_01027ef0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5fc0` | ScalarDelete_9763a0 | likely | Compiler-generated scalar deleting destructor: calls FUN_009763a0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a5fe0` | ScalarDelete_790600 | likely | Compiler-generated scalar deleting destructor: calls FUN_00790600(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6000` | ScalarDelete_a0b180 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a0b180(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6020` | ScalarDelete_a0f650 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a0f650(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6040` | ScalarDelete_a1d6d0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a1d6d0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6060` | ScalarDelete_a29370 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a29370(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6080` | ScalarDelete_0102a360 | likely | Compiler-generated scalar deleting destructor: calls FUN_0102a360(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a60a0` | ScalarDelete_a29b40 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a29b40(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a60e0` | ScalarDelete_a31860 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a31860(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6100` | ScalarDelete_a381a0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a381a0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6120` | ScalarDelete_0102af20 | likely | Compiler-generated scalar deleting destructor: calls FUN_0102af20(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6140` | ScalarDelete_0102b850 | likely | Compiler-generated scalar deleting destructor: calls FUN_0102b850(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6160` | ScalarDelete_a397e0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a397e0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6180` | ScalarDelete_0102cdc0 | likely | Compiler-generated scalar deleting destructor: calls FUN_0102cdc0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a61a0` | ScalarDelete_a3a710 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a3a710(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a61c0` | ScalarDelete_a3d8a0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a3d8a0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a61e0` | ScalarDelete_a3fe30 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a3fe30(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6200` | ScalarDelete_a41800 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a41800(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6220` | ScalarDelete_a43e10 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a43e10(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6240` | ScalarDelete_d0ccb0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00d0ccb0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6260` | ScalarDelete_d2ea60 | likely | Compiler-generated scalar deleting destructor: calls FUN_00d2ea60(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6280` | ScalarDelete_01000290 | likely | Compiler-generated scalar deleting destructor: calls FUN_01000290(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a62a0` | ScalarDelete_01000750 | likely | Compiler-generated scalar deleting destructor: calls FUN_01000750(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a62c0` | ScalarDelete_010f3590 | likely | Compiler-generated scalar deleting destructor: calls FUN_010f3590(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a62e0` | ScalarDelete_010f64f0 | likely | Compiler-generated scalar deleting destructor: calls FUN_010f64f0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6300` | ScalarDelete_0108ca80 | likely | Compiler-generated scalar deleting destructor: calls FUN_0108ca80(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6320` | ScalarDelete_0108c810 | likely | Compiler-generated scalar deleting destructor: calls FUN_0108c810(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6340` | ScalarDelete_d2e1b0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00d2e1b0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6360` | ScalarDelete_97a460 | likely | Compiler-generated scalar deleting destructor: calls FUN_0097a460(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6380` | ScalarDelete_a466c0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a466c0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a63a0` | ScalarDelete_a46bf0 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a46bf0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a63c0` | ScalarDelete_a4bc10 | likely | Compiler-generated scalar deleting destructor: calls FUN_00a4bc10(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a63e0` | ScalarDelete_a4e1e0 | likely | Compiler-generated scalar deleting destructor: calls thunk_FUN_00a4dc90(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, retu… |
| `008a6400` | ScalarDelete_98d860 | likely | Compiler-generated scalar deleting destructor: calls FUN_0098d860(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6420` | ScalarDelete_795700 | likely | Compiler-generated scalar deleting destructor: calls FUN_00795700(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6440` | ScalarDelete_71a8f0 | likely | Compiler-generated scalar deleting destructor: calls FUN_0071a8f0(); (the class teardown body) and then calls _free(this) when param_2 bit 0 is set, returning … |
| `008a6650` | SubPtr1d008 | likely | Pointer accessor: returns the pointer at this+0x3e0 plus the fixed offset 0x1d008 with no null check and no calls. |
| `008a6660` | SubPtr19a60 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x19a60, otherwise returns 0; no call… |
| `008a6680` | SubPtr2c340 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x2c340, otherwise returns 0; no call… |
| `008a66a0` | SubPtr2cf00 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x2cf00, otherwise returns 0; no call… |
| `008a66c0` | SubPtr2cf30 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x2cf30, otherwise returns 0; no call… |
| `008a66e0` | Member860 | likely | Raw field read: returns the 32-bit value at this+0x860; no calls and no writes. |
| `008a66f0` | Member864 | likely | Raw field read: returns the 32-bit value at this+0x864; no calls and no writes. |
| `008a6700` | SubPtr2c2c8 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x2c2c8, otherwise returns 0; no call… |
| `008a6720` | SubPtrb9c | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0xb9c, otherwise returns 0; no calls. |
| `008a6740` | SubPtr1add0 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x1add0, otherwise returns 0; no call… |
| `008a6760` | SubPtr1bee8 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x1bee8, otherwise returns 0; no call… |
| `008a6780` | SubPtr1d310 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x1d310, otherwise returns 0; no call… |
| `008a67a0` | SubPtr1d3d0 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x1d3d0, otherwise returns 0; no call… |
| `008a67c0` | SubPtr34f40 | likely | Guarded pointer accessor: reads the pointer at this+0x3e0; when it is non-null returns that pointer plus the fixed offset 0x34f40, otherwise returns 0; no call… |
| `008a6870` | CompoundFlagPredicate | likely | Compound boolean predicate over this: returns 1 only when the pointer at +0x6f4 is non-null, the five referenced objects at +0x11c, +0x130, +0x120, +0x124 and … |
| `008a68f0` | Wrapper_009d0f30 | likely | Adjustor thunk loads ECX from this+0x874 and jumps to 009d0f30, leaving stack arguments untouched. |
| `008a6900` | Call00963d10IfC8Set | likely | When this+0xc8 is non-null, calls FUN_00963d10 with ECX equal to that object and arguments (param_2 == 0, 0), then returns; otherwise returns. |
| `008a6920` | Call00a6dd70If5E0Set | likely | If this+0x5e0 is non-null, loads it into ECX and tail-jumps to 00a6dd70 with the original two stack arguments; otherwise returns 0 and consumes those arguments… |
| `008a6950` | Call00a14090If3E0SetDefault1 | likely | If this+0x3e0 is non-null, loads it into ECX and tail-jumps to 00a14090 with the original stack argument; otherwise returns 1 and consumes that argument with r… |
| `008a6970` | Call00a153f0If3E0SetDefault1 | likely | If this+0x3e0 is non-null, loads it into ECX and tail-jumps to 00a153f0 with the original stack argument; otherwise returns 1 and consumes that argument with r… |
| `008a6990` | Field464Via3E0 | likely | Conditional field read: when the pointer at this+0x3e0 is non-null returns the dword at that object +0x464; otherwise returns 0. |
| `008a6b70` | Call009be8e0If009ad3c0 | likely | Loads ECX from this+0xf8 and calls 009ad3c0; if the returned byte is nonzero it reloads that receiver and tail-jumps to 009be8e0, otherwise returns. |
| `008a6b90` | Call009be8e0And009be8f0If009ad3c0 | likely | When FUN_009ad3c0 returns non-zero: calls FUN_009be8e0, then calls FUN_009be8f0 with the value returned by FUN_009ad8d0(0); returns otherwise. No direct caller… |
| `008a6bd0` | SetFlag873 | likely | Writes the byte 1 at this+0x873 and returns; no calls and no callers in the export (reached through a vtable or table). |
| `008a6be0` | Call0094c910IfVtbl10True | likely | Calls the virtual at slot +0x10 of the object pointed to by this+0xec; when the returned char is non-zero it calls FUN_0094c910(param_2, param_3), otherwise it… |
| `008a6c20` | CallVtbl5COnEc | likely | Loads the object at this+0xec into ECX, calls its vtable slot +0x5c with param_2 as the stack argument, then returns with ret 4; this is a normal call, not a t… |
| `008a6c40` | IfType27Store154 | likely | Type-gated setter: when the object pointer at this+0xec is non-null it calls that object vtable slot 0 (a type/id getter) and, when the result equals 0x27 (39)… |
| `008a6c70` | AllocAndReplaceAtEC | likely | Creates a replacement object for the slot at this+0xec: when the current object exists and its vtable-slot-0 type equals 10 it returns early; otherwise it allo… |
| `008a6d40` | IfTypeBInvokeVtblF8 | likely | Type-gated virtual call: when the object at this+0xec is non-null and its vtable-slot-0 type equals 0xb (11), calls that object vtable slot +0xf8 with param_2;… |
| `008a6d80` | IfTypeFCall00a6fdb0 | likely | If this+0xec is non-null, calls that object vtable slot 0 for its type; when result is 15, reloads this+0xec into ECX and tail-jumps to 00a6fdb0. Other paths r… |
| `008a6db0` | IfTypeFCall00a6fe20 | likely | If this+0xec is non-null, calls that object vtable slot 0 for its type; when result is 15, reloads this+0xec into ECX and tail-jumps to 00a6fe20. Other paths r… |
| `008a6de0` | IfTypeFReturn00a6f7d0 | likely | Type-gated value call: when the object at this+0xec is non-null and its vtable-slot-0 type equals 0xf (15) it returns the result of FUN_00a6f7d0(param_2); othe… |
| `008a6e20` | CurrentTypeOrZero | likely | Returns 0 when this+0xec is null; otherwise loads that object into ECX and jumps through its vtable slot 0, returning the target value. |
| `008a6e40` | AdjustorThunk_00790d00 | likely | Adjustor thunk loads ECX from this+0xe8 and jumps to 00790d00; stack arguments are unchanged. |
| `008a6e50` | AdjustorThunk_00790d10 | likely | Adjustor thunk loads ECX from this+0xe8 and jumps to 00790d10; stack arguments are unchanged. |
| `008a6e60` | SetByte258 | likely | Writes the byte argument at this+0x258 (600) and returns; no calls and no callers in the export (reached through a vtable). |
| `008a6e70` | Flag258AndGlobal67010Predicate | likely | Predicate: returns 1 when the byte at this+0x258 is non-zero and the dword at (*(this+0x20) + 0x67010) equals 0; returns 0 otherwise. |
| `008a78d0` | Set194AndNotify | likely | Stores param_2 at this+0x194, loads ECX from this+0x23c and tail-jumps to 00a44b70 with param_2 still on the stack. |
| `008a7910` | ApplyFlag40000003IfInBounds | likely | Bounds-checked apply: calls FUN_00793560(param_1, &local_8, &local_4) to resolve coordinates and requires success plus non-negative values, checks local_8 agai… |
| `008a7a40` | UpdateHighWater82C | likely | High-water update: when the dword at param_2+0x82c is less than param_1, it raises +0x82c to param_1 and writes param_3 to +0x168 and max(param_3, param_4) to … |
| `008a7a90` | ScalarDelete_0102f9e0 | likely | Compiler-generated scalar deleting destructor: calls FUN_0102f9e0 (class teardown) then calls _free(this) when param_2 bit 0 is set, returning the object point… |
| `008a7b20` | Call00a0f640If648Set | likely | When this+0x648 is non-null, loads it into ECX and tail-jumps to 00a0f640; otherwise returns. |
| `008a7bc0` | RouteMessageToComponents | likely | Routes param_2 in order: if this+0x1a8 exists and byte this+0x83d is set, calls 00acae30 with that object in ECX; if this+0xec exists and *param_2==2, calls 00… |
| `008a7c20` | RouteMessage13ToComponents | likely | Message router: when this+0x1a8 exists and either the byte at this+0x83d is set or the first word of param_2 equals 0x13, it calls FUN_00acc160(param_2); then,… |
| `008a7c60` | GetOrCreate654 | likely | Lazy factory: when this+0x654 is null it allocates 0x30 bytes with FUN_00d833a6, constructs the object with FUN_009f5a00 and stores it at +0x654 (0 on allocati… |
| `008a7ce0` | Destroy654IfIdle | likely | Teardown: when this+0x654 is non-null and either param_2 is non-zero or the byte at that object +0x18 is non-zero, it calls FUN_009f6890, calls FUN_009f5a70 on… |
| `008a7d30` | Call009f6aa0If654Set | likely | When this+0x654 is non-null, loads it into ECX and tail-jumps to 009f6aa0; otherwise returns. |
| `008a7d50` | ReadVector4FromC8 | likely | Reads a four-dword vector through the object at this+0xc8: when that pointer is non-null it calls its virtual slot +0x1c, copies the four returned dwords into … |
| `008a7d90` | ReadBlock610FromC8 | likely | Reads four consecutive dwords at +0x610, +0x614, +0x618 and +0x61c of the object at this+0xc8 into param_2[0..3] and returns 1; when the pointer is null it ret… |
| `008a7dd0` | IfFlagB4ClearCallVtbl88ThenSet1 | likely | If the object at this+0x17c has byte +0xb4 clear, calls this vtable slot +0x88, then loads the +0x17c object into ECX and calls 009cf7d0(1); otherwise returns. |
| `008a7e00` | Call009cf7d0With0 | likely | Loads ECX from this+0x17c, pushes 0 and makes a normal call to 009cf7d0, then returns. |
| `008a7e10` | HeadingFromC8Vector | likely | Computes fpatan of the two floats at +0x610 and +0x618 of the object at this+0xc8, stores the resulting angle through param_2 and returns 1; when the pointer i… |
| `008a7f50` | Flag6E4 | likely | Raw byte getter: returns the byte at this+0x6e4; no calls and no writes. |
| `008a7f90` | StoreTwoVectors740 | likely | Snapshot store: sets the byte at this+0x73a to 1, copies four dwords from param_2 into +0x740..+0x74c, four dwords from param_3 into +0x750..+0x75c, and stores… |
| `008a8000` | Clear73AAndNotify | likely | Clears the byte at this+0x73a; if this+0x1ec is non-null, loads it into ECX and tail-jumps to 009ce2f0, otherwise returns. |
| `008a8430` | MapStateToKind | likely | Maps the dword at (*(this+0x8f8)+0x14) to a small kind: values 1 and 2 return 1, values 3 and 4 return 2, anything else returns 3; no calls. |
| `008a8470` | Field3C | likely | Raw field read: returns the dword at (*(this+0x8f8)+0x3c); no calls. No callers in the export. |
| `008a8480` | CallThreeHelpers | likely | Calls 00769710, loads ECX from the returned object+0x28 and calls 007ae0c0, then loads ECX from this+0x8ec and tail-jumps to 01000560. |
| `008a84a0` | StoredDimensionsMatchRenderer | likely | Compares the two dwords at (*(this+0x8f8)+4) and +8 with the render device virtuals +0x2c and +0x30 (device pointer taken from *(*(*(this+0xe8)+0x25c)+0x1d0));… |
| `008a8b60` | SetActiveByte83d | likely | When the byte at this+0x83d differs from the incoming value it stores the value there and then calls FUN_0094c8c0(param) followed by FUN_0094c8a0(param); when … |
| `008a8be0` | Field20Plus66f20 | likely | Byte-verified field accessor (8b 41 20 / 05 20 6f 06 00 / c3): returns the pointer formed by adding 0x66f20 to the dword at this+0x20. No calls. |
| `008a8c80` | Field644Get60 | likely | Byte-verified two-level field read (8b 81 44 06 00 00 / 8b 40 60 / c3): returns the dword at (*(this+0x644))+0x60. No calls. |
| `008a8c90` | Field644Set60_Bool | likely | Byte-verified (33 c0 / 39 44 24 04 / 0f 95 c0 / 89 41 60 / c2 04 00): normalises the incoming parameter to 0/1 and stores it at (*(this+0x644))+0x60; returns n… |
| `008a8d30` | Field20PlusA0 | likely | Byte-verified accessor (8b 41 20 / 05 a0 00 00 00 / c3): returns the pointer formed by adding 0xa0 to the dword at this+0x20. No calls; called from 00c18b10, 0… |
| `008a8d80` | GuardedTailCall_00ad6760 | likely | Byte-verified (8b 89 dc 0b 00 00 / 85 c9 / 74 05 / e9 / c2 04 00): when this+0xbdc is non-zero it tail-jumps to FUN_00ad6760 with the caller's argument, else r… |
| `008a8da0` | GuardedActivityListCountForName | likely | Byte-verified (8b 44 24 04 / 85 c0 / 74 13 / 8b 89 e0 01 00 00 / 85 c9 / 74 09 / 89 44 24 04 / e9 / 33 c0 / c2 04 00): returns 0 when the dword argument is zer… |
| `008a8dc0` | GuardedTailCall_00ada8b0 | likely | Byte-verified (8b 44 24 04 / 85 c0 / 74 13 / 8b 89 e0 01 00 00 / 85 c9 / 74 09 / 89 44 24 04 / e9 / 33 c0 / c2 04 00): returns 0 when the dword argument is zer… |
| `008a8e60` | ClearByte92AndField700 | likely | Byte-verified (33 c0 / 88 81 92 00 00 00 / 89 81 00 07 00 00 / c2 0c 00): zeroes the byte at ecx+0x92 and the dword at ecx+0x700 and returns with ret 0xc, popp… |
| `008a8fd0` | Field778GetByte12 | likely | Byte-verified (8b 81 78 07 00 00 / 8a 40 12 / c3): loads the pointer at this+0x778 and returns its byte at +0x12 in the low byte of EAX (the high bytes are the… |
| `008a8fe0` | Predicate_128_NotNull_And_Plus18 | likely | Byte-verified (8b 81 28 01 00 00 / 85 c0 / 74 0c / 80 78 18 00 / 74 06 / b8 01 00 00 00 / c3 / 33 c0 / c3): returns 1 only when this+0x128 holds a pointer whos… |
| `008a9000` | Wrapper_009ad3c0_FieldF8 | likely | Byte-verified thunk (8b 89 f8 00 00 00 / e9 b5 43 10 00): loads this+0xf8 into ecx and tail-jumps to FUN_009ad3c0. No stack arguments. Called from 009c3270. |
| `008a90c0` | SetPair270_274 | likely | Calls FUN_0101c8a0(param_2) first, then stores param_3 at [this+0x20]+0x270 and param_4 at [this+0x20]+0x274; no branches, no return value. |
| `008a9140` | Wrapper_00ace110_Const1 | likely | Two-argument forwarder that appends the constant 1: calls FUN_00ace110(param_1, param_2, 1) and returns nothing. Seven callers in the 00c4d9f0..00c4def0 block. |
| `008a91c0` | Wrapper_00ae00b0_Field234 | likely | Byte-verified thunk (8b 89 34 02 00 00 / e9 e5 6e 23 00): loads this+0x234 into ecx and tail-jumps to FUN_00ae00b0; no stack arguments. Called from 00995a90. |
| `008a91d0` | Predicate_80_And_81 | likely | Returns 1 only when both the byte at this+0x80 and the byte at this+0x81 are non-zero; returns 0 otherwise. No calls. Six callers including 009ce560 (the vecto… |
| `008a91f0` | GetByte80 | likely | Byte-verified (8a 81 80 00 00 00 / c3): returns the byte at this+0x80 in AL; no calls. Eight callers (00958240, 0095d380, 0096de80, 00970b60, 00a8de60, 00a8e00… |
| `008a9230` | IsEven65c | likely | Signed mod-2 test on the dword at this+0x65c (field & 0x80000001 with the negative fix-up): returns true when the value is even. No calls. Called from 00973200. |
| `008a9270` | IsEven660 | likely | Signed mod-2 test on the dword at this+0x660 (field & 0x80000001 with the negative fix-up): returns true when the value is even. No calls. Called from 00970b60. |
| `008a9290` | IsLessThanTwo660 | likely | Returns whether the signed dword at this+0x660 is less than 2. No calls. Called from 00970b60. |
| `008a9460` | ScalarDeletingDtor_00ae1600 | likely | Deleting-destructor shape: calls FUN_00ae1600 (the class teardown body) and, when bit 0 of the flag argument is set, frees this with _free before returning thi… |
| `008a94d0` | CopyTwoDwords | likely | Copies two consecutive dwords: *param_1 = *param_2 and param_1[1] = param_2[1]; no branches, no return. Called from 00973200 (which also calls the 65c/660 pred… |
| `008a9510` | NotEqualEightBytes | likely | Returns 0 when both dwords of the two 8-byte arguments are equal, 1 otherwise. No calls. Called from 00ce5380. |
| `008a9760` | CopyTwoDwords9760 | likely | Copies two consecutive dwords (*param_1 = *param_2; param_1[1] = param_2[1]); identical emission to 008a94d0 but called from 00b5ce60. |
| `008a97a0` | ZeroInit_Af0_At8 | likely | Plain zero-initialiser: sets this+4 to 0, memsets 0xaf0 bytes starting at this+8 to zero, sets *(this) to 0 and returns this. No vtable is installed and it has… |
| `008a97e0` | ZeroInit_Af0_At10 | likely | Zero-initialiser: clears this+4, +8 and +0xc, memsets 0xaf0 bytes at this+0x10, sets *(this) to 0 and returns this; no vtable, no callers. |
| `008a9ce0` | ZeroInit_40_At10 | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 0x40 bytes at this+0x10 and zeroes *(this); no vtable, no callers. |
| `008aa140` | ZeroInit_80_At10 | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 0x80 bytes at this+0x10 and zeroes *(this); no vtable, no callers. |
| `008aa590` | ZeroInit_8000_At8 | likely | Zero-initialiser: clears this+4, memsets 0x8000 bytes starting at this+8 and zeroes *(this); no vtable, no callers. |
| `008aa7f0` | ZeroInit_40_At10_C | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 0x40 bytes at this+0x10 and zeroes *(this); no vtable, no callers. Third emission of the 16-entry zero-init sh… |
| `008aaa50` | ZeroInit_200_At10 | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 0x200 bytes at this+0x10 and zeroes *(this); no vtable, no callers. |
| `008aac20` | ZeroInit_100_At10 | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 0x100 bytes at this+0x10 and zeroes *(this); no vtable, no callers. |
| `008aae30` | ZeroInit_190_At10 | likely | Zero-initialiser: clears this+4/+8/+0xc, memsets 400 (0x190) bytes at this+0x10 and zeroes *(this); no vtable, no callers. |
| `008ab5d0` | ScalarDeletingDtor_00ae1d50 | likely | Deleting-destructor shape: calls FUN_00ae1d50 (class teardown body) and, when bit 0 of the flag argument is set, frees this with _free before returning it. No … |
| `008ab820` | InitAndStore234 | likely | Calls FUN_01039a40 with ECX still holding this; that helper initializes the object members at +4 and +0x18. Then copies *param_2 into this+0x234 and returns th… |
| `008ab880` | DtorMemberAt18 | likely | Destructor body for an object received in ecx (saved to esp+4 for the SEH frame): byte-verified (83 c1 18 before the call) it adds 0x18 to this and calls FUN_0… |
| `008aca80` | DtorMemberAt18_0103d010 | likely | Byte-verified SEH-framed destructor wrapper: adds 0x18 to this (83 c1 18) and calls FUN_0103d010 - the reward-bundle-family destructor (RewardBundleBase vtable… |
| `008acad0` | Wrapper_0103da90 | likely | Sets up an SEH frame and calls FUN_0103da90 with incoming ECX unchanged; restores ExceptionList and returns. Raw bytes show no explicit stack arguments at this… |
| `008acb20` | Wrapper_0101cd70 | likely | Sets up an SEH frame and calls FUN_0101cd70 with incoming ECX unchanged; restores ExceptionList and returns. Raw bytes show no explicit stack arguments at this… |
| `008acb70` | Wrapper_01039910 | likely | Sets up an SEH frame and calls FUN_01039910 with incoming ECX unchanged; restores ExceptionList and returns. Raw bytes show no explicit stack arguments at this… |
| `008ade60` | Wrapper_007e90b0 | likely | Sets up an SEH frame, saves incoming ECX, calls FUN_007e90b0 with ECX unchanged, restores ExceptionList and returns. Raw bytes show no explicit stack arguments… |
| `008adeb0` | Wrapper_010b4ac0 | likely | Sets up an SEH frame and calls FUN_010b4ac0 with incoming ECX unchanged; restores ExceptionList and returns. The callee is an IString-owning teardown body. Raw… |
| `008adfa0` | Wrapper_01039590 | likely | Sets up an SEH frame, calls the one-instruction return stub FUN_01039590, restores ExceptionList and returns. The stub has no body effects; the XOR-local value… |
| `008afd60` | InitStore10 | likely | Constructor-like initialiser: calls FUN_0101cd40 (an unmapped member init taking this in ecx), sets this+4 to 0, copies *param_2 into this+0x10 and returns thi… |
| `008afd80` | CallAndReturn_0089cf50 | likely | SEH-framed wrapper: saves incoming ECX, calls FUN_0089cf50 with ECX unchanged and no explicit stack arguments, restores ExceptionList, then returns the saved p… |
| `008b0d10` | InitAndStore1c | likely | SEH-framed initializer: saves this from ECX, calls FUN_01039570 with the same ECX (the helper zeroes this+0 through this+0x10), copies *param_2 into this+0x1c,… |
| `008b1bf0` | DoubleZeroInit_Af0 | likely | Large two-block zero-init inside an SEH frame: clears this+4, memsets 0xaf0 bytes at this+8, zeroes *(this), then clears this+0xafc/+0xb00/+0xb04 (indices 0x2b… |
| `008b1ce0` | CopyPair920 | likely | Copies two dwords from this+0x920 and this+0x924 into param_2[0] and param_2[1]; no branches, no calls. |
| `008b1d40` | GetPairFrom28OrDefault | likely | Reads the pointer at this+0x28 (falling back to the global DAT_01be48b8 when null) and copies its first two dwords into param_2[0] and param_2[1]. |
| `008b1f20` | ForwardWithField4Argument | likely | Loads the first stack argument, replaces it with that pointed object's dword at +4, then tail-jumps to 00976560. |
| `008b2b00` | CtorDefaultId0x27e7 | likely | Zeroes field 0, constructs an embedded object through 00720bf0 (the IconData/Loggable constructor), zeroes +0x10, +0x14, +0x18, +0x24, +0x28, +0x2c and the byt… |
| `008ba830` | CallHelperForResolvedObject | likely | Resolves an object from two ids; if present, fills a local structure with FUN_0095a400 and passes it with the final two arguments to FUN_00793560, returning th… |
| `008bb7b0` | SetPointerAndAuxWord | likely | When param_2 is zero, clears fields +4/+8; otherwise passes param_2 to FUN_00d833a6, stores that result at +4, and stores param_2 at +8. |
| `008c6300` | WrapScalarVec4_CallSub290 | likely | Stores param_2 in a local 4-dword block (other three dwords zero), then calls FUN_008c6290(param_1, &block, param_3, param_4, param_5, param_6). Returns void (… |
| `008c63c0` | WrapScalarVec4_CallSub350 | likely | Stores param_2 in a local 4-dword block (other three dwords zero), then calls FUN_008c6350(param_1, &block, param_3..param_7). Returns void (stack args). |
| `008c6420` | Owner3e0_MatchPairThenCall | likely | Returns 0 unless this+0x3e0 is nonnull, this+0x1b8 is nonzero, (this+0x3e0)+0x3c8 equals param_2 and (this+0x3e0)+0x3cc equals param_3. On a match it calls FUN… |
| `0091b670` | ScalarDeletingDtor_91b670 | likely | Scalar deleting destructor: under an SEH frame calls FUN_0090ba10 (the destructor body that restores Loggable::vftable at +0x1fc and frees two members) and the… |
| `0091dfb0` | UnwindCleanup_00917e70 | likely | SEH-framed cleanup leaf whose only action is to call FUN_00917e70 (the destructor body used by callers in the 00917xxx family); a stack cookie value is passed … |
| `00922510` | ScalarDtor_00914a10 | likely | Scalar deleting destructor shape: under an SEH frame calls 00914a10 with this (ECX), then frees the object through _free when bit 0 of the second argument is s… |
| `009265f0` | ReleaseStringMembersAndFree | likely | Runs 0091b170 cleanup, then frees the object only when the deleting flag is set. |
| `00926650` | ReleaseStringMembersAndFree | likely | Runs 0091b260 cleanup, then frees the object only when the deleting flag is set. |
| `0092b110` | CleanupAndConditionalFree | likely | Runs cleanup helper 009208e0 and frees the object only when the deleting flag is set. |
| `00956810` | CleanupThenMaybeFreeObject | likely | Calls cleanup helper 00713bf0 on the incoming ECX receiver with no stack arguments; if deletion flag bit 0 is set, frees the object, otherwise returns it uncha… |
| `009582c0` | UpdateIndex13fAndConditional140Clear | likely | When param_3 is zero, clears param_1[0x140]; calls the object virtual at offset 0x10 with a two-word stack temporary, lets 008c6420 replace param_2 when it ret… |
| `009586d0` | ConditionalContextQuery | likely | Calls virtual slot 9 and, when nonzero, returns 0076c590's result; otherwise returns zero. |
| `009614c0` | StoreThreeKeysAndPropagate | likely | Stores three supplied values at this+0x17c, +0x180 and +0x184; if the virtual lookup returns an object, copies those values to that object's +0x3c4..+0x3cc fie… |
| `00964540` | InitializeRecordAndStoreField1dc | likely | Calls FUN_00963e50 to initialize a record, stores the first supplied dword at this+0x1dc, and returns param_1. |
| `00983ea0` | GetField04 | likely | 4-byte getter decoded from bytes 0x00983ea0 (8b 41 04 c3): returns the dword field at [this+4] in eax. |
| `00983eb0` | GetField68 | likely | 10-byte getter decoded from bytes 0x00983eb0 (8b 44 24 04 8b 40 68 c2 04 00): takes the node pointer as a stack argument and returns the dword at [arg+0x68] - … |
| `00984010` | WalkNodesAndCallHelperForGuidMatches | likely | When [this+0x344] is zero and param_2 is positive, walks the node list at [this+4] (next +0x68): for every node whose GUID pair [node+0x17a0]/[node+0x17a4] equ… |
| `009841c0` | ToggleFlag338AndCallNodeHook | likely | Flips the byte at [this+0x338] between 0 and 1, walks the list starting at [this+4] through each node?s +0x68 link, and calls 00958510 once per node with the f… |
| `00984250` | ForwardConst2ToNodes | likely | Walks the node list at [this+4] (next +0x68) and calls 00957fe0(2) once for every node. |
| `009842a0` | CallNodeVirtual4WhenPredicatesMatch | likely | Walks the node list at [this+4] (next +0x68); for each node it evaluates 00961750() and, when that is true and 009586d0() is false, calls the node's virtual sl… |
| `009842f0` | ForwardToNodesIfFlag | likely | Walks the node list at [this+4] (next +0x68) and calls 0096c3b0() for every node for which 009586d0() returns true. |
| `00984320` | ForwardToNodeWithField18d0 | likely | Walks the node list at [this+4] (next +0x68) and calls 009622f0() for every node whose dword field at +0x18d0 equals the argument param_2. |
| `00984350` | GetField358 | likely | 7-byte getter decoded from bytes 0x00984350 (8b 81 58 03 00 00 c3): returns the dword field at [this+0x358] in eax; called from three sibling functions 00a79d2… |
| `009849e0` | GetNodeNextLink08 | likely | Loads and returns the dword at object+8. |
| `00984ac0` | TraverseFlaggedNodesAndCallGlobal | likely | Walks the node list at [this+4] (next +0x68) and, for nodes whose byte at +0x140 has bit 0x80 set and for which 00957f90() is false and which resolve to an obj… |
| `00984d10` | LookupNodeAndAttach | likely | Calls the 100-bucket table lookup 00984960 with (param_1, param_2) and, when it returns a non-null node, passes that node plus param_3 to the table insert help… |
| `00984d50` | SetFlag40OnNodes | likely | 30-byte loop decoded from 0x00984d50 (8b 41 04 85 c0 74 16 b1 40 8d a4 24 00 00 00 00 08 88 46 01 00 00 8b 40 68 85 c0 75 f3 c3): walks the list at [this+4] th… |
| `0099d970` | CopySelectedFieldsAndInitialize | likely | Copies dwords at offsets +4, +8, +0xC, +0x10, and +0x20 through +0x34 (including +0x24,+0x28,+0x2C,+0x30); skips +0x14 and +0x18, calls 0099C1B0(1), and return… |
| `009aae30` | DeletingDtor_009aae30 | likely | Scalar deleting destructor: calls FUN_00c43440 (the destructor of the class this wrapper belongs to), and when bit 0 of param_2 is set frees the object through… |
| `009aae50` | AdjustThis_2bea8 | likely | 7-byte accessor: lea eax,[ecx+0x2bea8]; ret (pseudocode returns param_1 + 0x2bea8) - returns the address of the field/sub-object at this+0x2bea8, the same larg… |
| `009aae80` | SetField54_2000 | likely | 8-byte setter: mov dword ptr [ecx+0x54], 0x7d0 (2000); ret - stores the constant 2000 into the dword field at this+0x54. |
| `009aaf30` | SetField2bea0_3 | likely | 11-byte setter: mov dword ptr [ecx+0x2bea0], 3; ret - stores the constant 3 into the dword field at this+0x2bea0 (adjacent to the +0x2bea4 pointer used by the … |
| `009ac1c0` | DeletingDtor_009ac1c0 | likely | Deleting-destructor form: invokes FUN_0103d010 (the destructor of the reward-bundle class family, which references RewardBundleBase::vftable and SoeUtil list m… |
| `009ad230` | LookupIdThenReadField5CIfFlagClear | likely | Calls 009ad100 with param_1. If the lookup returns a state and byte state+0x2c1 is clear, returns state+0x5c; otherwise returns zero. |
| `009ad260` | LookupIdThenReadField58 | likely | Calls 009ad100 twice with param_1; if the first lookup is nonzero, returns field +0x58 from the second result without another null check. Returns zero when the… |
| `009ad290` | LookupIdThenReadFirstDword | likely | Calls 009ad100 with param_1; if that lookup returns nonzero, calls it again with the same ID and returns the second result's first dword without another null c… |
| `009ad2c0` | MatchStateKindCodesAndMode | likely | Starts with in_EAX masked by 0xffffff00; for state kinds 4 and 0x11 it returns that value with low byte one, and for kind 0x3e9 it tests mode +0x178, returning… |
| `009ad440` | ResolveRecordAndPassField50 | likely | When this+0x18 is nonzero, resolves param_2 with 00c42400 and calls 00c33910 with the resolved pointer+0x50, or with null on a miss. |
| `009ad7a0` | ResolveStateAndDispatchSlot4 | likely | Uses current state for IDs below one or searches ten ID buckets by fields +0x918/+0x91c; then calls 0103b110, 0103a1e0(param_3), 00c33910(state+0x200), and the… |
| `009ad890` | WriteStateFields32CAnd328 | likely | Zeros *param_2 and *param_3. If this+0x18 is nonzero, writes state+0x32c to *param_2 and state+0x328 to *param_3. |
| `009af430` | DtorThreeUtf16StringsAndEmbeddedObject | likely | Restores IString<unsigned_short> vtables at +4, +0x14, and +0x34; for each positive-length string it decrements the owned buffer refcount and calls the string … |
| `009aff60` | CleanupWrapper_00703350 | likely | Installs a local SEH frame, calls 00703350 with the incoming ECX object as receiver, restores the previous exception-list head, and returns. |
| `009affb0` | SetBit20AtOffset0D | likely | Sets bit 0x20 in byte this+0x0d when param_2 is nonzero, and clears that bit otherwise. |
| `009affd0` | InvokeEmbeddedSlot1ThenSetFlag80 | likely | Calls virtual slot +4 on the embedded object at this+0x58 with ECX=this+0x58 and stack arguments (param_2, param_3, 1), then sets bit 0x80 in byte this+0x0d. |
| `009b04e0` | ScalarDeletingDtor_00703350 | likely | Calls 00703350 with the incoming object in ECX, frees that object only when deletion-flag bit zero is set, and otherwise returns it. |
| `009b05e0` | ScalarDeletingDtor_009af350 | likely | Calls 009af350 on the object, frees it only when deletion-flag bit zero is set, and otherwise returns it. |
| `009b0600` | ScalarDeletingDtor_009af430 | likely | Calls 009af430 on the object, frees it only when deletion-flag bit zero is set, and otherwise returns it. |
| `009b0b00` | ScalarDeletingDtor_StringMemberAt10 | likely | Calls 009afe40 with this in ECX; that helper releases the IString member at object offset +0x10. It then frees the object only when deletion-flag bit zero is s… |
| `009b0c50` | ScalarDeletingDtor_009afeb0 | likely | Calls 009afeb0 on the object, frees it only when deletion-flag bit zero is set, and otherwise returns it. |
| `009c1bc0` | DestroyStringAtOffset18AndLoggingHeader | likely | Sets ECX to this+0x18 and calls 00709b80, restores ECX to this and calls 00d0a510, then restores the exception state. |
| `009c2490` | Wrapper_009c21b0 | likely | Calls 009c21b0 with this in ECX, then returns the supplied first argument unchanged. |
| `009ce2f0` | DrainFlagSelectedListsAndCopyDefaults | likely | Tests this+0x8c1. When it equals 1, calls FUN_008d3f40 once and zeros bytes +0x8c4 and +0x8c3. Otherwise calls the helper twice, zeros dword +0x8c8 and the sam… |
| `009ce3f0` | InitializeListsAndStoreSquaredLookupValue | likely | Stores the input at +4, calls FUN_009cc1c0 with receivers this+0x40 and this+0x474, initializes state bytes and fields, then walks an object-referenced tree fo… |
| `009d1500` | DestroyLocalStringAndOptionallyFree | likely | Destroys a stack string object through FUN_009d1320 and frees the supplied outer object when the deleting flag is set. |
| `009d1600` | CopyStringRecordTrailingFields | likely | Copies an IString-like value into the supplied object and copies three trailing dwords from source offsets +0x24, +0x28, and +0x2c. |
| `009d1930` | FormatAndAppendLabelRecord | likely | Formats a label as '%s [%d]', copies three supplied dwords and one extra value into a temporary record, then appends that record to a linked list. |
| `009d2280` | DestroyThenOptionallyFreeObject | likely | Calls FUN_009d1f20 to destroy the contained object and frees the supplied outer object when the deleting flag is set. |
| `009d2690` | UpdateObjectAtOffsetC | likely | When this+0xc is non-null and this+4 is clear, obtains the global context, calls FUN_00fdbb30 with the supplied argument, and sets this+4. Every active call th… |
| `009eb100` | SetFlagByteB8 | likely | Eight-byte thiscall setter: bytes c6 81 b8 00 00 00 01 c3 = mov byte ptr [ecx+0xb8],1 ; ret; writes 1 to the byte at this+0xb8 and returns. |
| `009eb110` | SetFlagByteB9 | likely | Eight-byte thiscall setter: bytes c6 81 b9 00 00 00 01 c3 = mov byte ptr [ecx+0xb9],1 ; ret; writes 1 to the byte at this+0xb9 and returns. |
| `009ee230` | BaseDestructorWrapper_009ee230 | likely | SEH-framed cleanup wrapper calls base destructor 0070a790 with the incoming ECX receiver, restores the prior exception chain, and returns; the cookie push is f… |
| `009f7de0` | WaitForKeyedEntry | likely | Fastcall on this. Gets the keyed entry at this+0x3c through 00709a70. If found, increments its counter at +8 only when the previous value is below 1, calls vta… |
| `00a05780` | InitInvalidPairAndIconData | likely | Writes 0xffffffff to this+0 and this+4, then calls 00720bf0 to initialize an IconData subobject at this+8 with zero arguments. |
| `00a063f0` | ScalarDeletingDtor_EmbeddedAt4 | likely | Calls FUN_01061530 with the embedded subobject at this+4, then calls _free(this) when delete-flag bit 0 is set. |
| `00a10660` | ScalarDeletingDtor_00c74710 | likely | Scalar deleting destructor: calls FUN_00c74710 (the object's real destructor body, outside this shard) and frees the object with _free when bit 0 of param_2 is… |
| `00a10680` | LookupField35520_ThenRead14 | likely | Looks up *(this+0x35520) through FUN_00726e30 and returns the dword at (+0x14) of the returned object, or -1 (0xffffffff) when the lookup yields nothing. |
| `00a106b0` | WriteSelectedFieldOrParam | likely | Writes *(this+0x474) or *(this+0x478) through param_2 for selectors 1 or 2; otherwise zeroes the out value through FUN_0101cd40 and stores param_3 there. |
| `00a10890` | GetFieldByMode_0x1bed0_0x1bed4 | likely | Returns *(this+0x1bed4) when *(this+0x464) is non-zero, otherwise *(this+0x1bed0). |
| `00a108f0` | CallAndReturnFirstArg_00c7e230 | likely | Calls FUN_00c7e230(param_1,param_2), then returns param_1 unchanged. |
| `00a22590` | CallSlot34WithField350 | likely | Loads the pointer at *(this+0x44)+0x6f4; if non-null, calls that object?s virtual slot +0x34 with its dword field at +0x350. |
| `00a225c0` | CallSlot38WithField350 | likely | Loads the pointer at *(this+0x44)+0x6f4; if non-null, calls its virtual slot +0x38 with the dword at object offset +0x350. |
| `00a225e0` | CallSlot40WithField1A4 | likely | Loads the pointer at *(this+0x44)+0x6f4; if non-null, calls its virtual slot +0x40 with the dword at object offset +0x1a4. |
| `00a22600` | HasReferencedObject | likely | Returns whether the pointer at *(this+0x44)+0x6f4 is non-null. |
| `00a22610` | Field350OrZeroWhenRecordFlagSet | likely | Reads the pointer at *(this+0x44)+0x6f4 and returns 0 when absent; otherwise reads its dword at +0x350, looks up that value through 01049a70, and returns 0 if … |
| `00a22640` | MapField1A4ToConstant | likely | When the pointer at *(this+0x44)+0x6f4 exists, maps its dword at +0x1a4 values 1, 2, 3, 4 and 5 to 0xbec, 0xbed, 0xc8c, 0xc8b and 0xc8a; returns 0xbee for othe… |
| `00a226a0` | RecordField100OrDefaultBee | likely | Uses the supplied key, or the referenced object?s dword at +0x350 when the key is zero, to look up a record through 01049a70; returns that record?s dword at +0… |
| `00a22720` | RecordFieldFCOrZero | likely | Uses the supplied key, or the referenced object?s dword at +0x350 when the key is zero, to look up a record through 01049a70; returns that record?s dword at +0… |
| `00a22760` | CallSlot10OnPair | likely | Calls virtual slot +0x10 on the object at *param_1, then calls the same slot on the object at param_1[1]. |
| `00a22f30` | ListNodeAtMappedIndex | likely | If the flag at this+0x258 is clear, calls virtual slot +0x38 on this; obtains the object through this+0x254 -> +0x44 -> +0x6f4, maps the supplied key to an ind… |
| `00a23120` | CopyRecordFields50And54OrDefaultPair | likely | Looks up the dword at referenced-object offset +0x350 through 01049a70; when a record exists, copies its dwords at +0x50 and +0x54 to the output pair, otherwis… |
| `00a2e900` | ScalarDeletingDtor_010647c0 | likely | Calls 010647c0 with the incoming this receiver, then frees param_1 through _free when flag bit 0 is set. |
| `00a3ae50` | LazySingletonInit | likely | One-shot global initialiser guarded by DAT_01be69c4 == 0: allocates 0x10 bytes through FUN_00d833a6, and when the allocation succeeded constructs the object wi… |
| `00a3b370` | TailCallMember24bc_a3b370 | likely | 11-byte tail-call thunk: loads the member pointer at this+0x24bc into ECX and jumps to FUN_00c96da0, so the callee runs with that member as its this pointer. |
| `00a3b380` | TailCallMember24c0_a3b380 | likely | 11-byte tail-call thunk: loads the member pointer at this+0x24c0 into ECX and jumps to FUN_00c96de0, running the callee with that member as this. |
| `00a3b7b0` | NullAndCopyDword | likely | Two-store initialiser running inside an SEH frame: nulls the first dword of the object and copies one dword from the source into the object slot +0xc. |
| `00a43f80` | ReturnMinusOneIfDefinitionHolderNull | likely | Checks *(this+0x20). If null, places -1 in EAX, restores the 0x0c-byte stack frame, and returns with ret 8. Otherwise its short conditional branch transfers ex… |
| `00a43f97` | SelectListEntryByOrder | likely | Walks nodes starting at *(in_EAX+0x9c) through node[10]. For nodes whose field +4 equals the requested category, calls FUN_00a43f00 on node[0], tracks the smal… |
| `00a4dc90` | ReleaseThreeRefsAndListNodes | likely | Clears the byte at +0x34. For each member slot +0xc, +0x1c and +0x2c it reads through virtual slot +0x28, passes the result and zero to FUN_0081ad80, decrement… |
| `00a4f670` | CallMemberAt4c | likely | If the member pointer at +0x4c of the object is non-null, loads it into ecx and calls 00fed630 on it (bytes 8b 49 4c 85 c9 74 05 e8 .. c2 04 00: one argument o… |
| `00a4f700` | PairSlots_3b0_3b8Ready | likely | Predicate: returns 1 only when both handle slots are usable - +0x3b0 is non-null and its target's virtual slot 0xc returns true, and +0x3b8 is non-null and its… |
| `00a4f740` | SetFloatOnPair_3b4_3bc | likely | Converts the integer argument to a float (cvtsi2ss) and passes it to FUN_0078fcc0 for the handle at +0x3b4 and again for the handle at +0x3bc, when each is non… |
| `00a4fb20` | SetByte368To1 | likely | Trivial setter: writes 1 into the byte at +0x368 and returns (bytes c6 81 68 03 00 00 01 c3 = mov byte ptr [ecx+0x368],1; ret). |
| `00a4fb30` | SetByte369To1 | likely | Trivial setter: writes 1 into the byte at +0x369 and returns (bytes c6 81 69 03 00 00 01 c3 = mov byte ptr [ecx+0x369],1; ret). |
| `00a4fb40` | SetDword370 | likely | Trivial setter: stores the dword argument into the field at +0x370 and returns (bytes 8b 44 24 04 89 81 70 03 00 00 c2 04 00 = mov eax,[esp+4]; mov [ecx+0x370]… |
| `00a4fb50` | Or0x800AtObject14Plus0c | likely | Trivial flag setter: loads the object at +0x14 and ORs the bit 0x800 into its dword at +0xc (bytes 8b 41 14 81 48 0c 00 08 00 00 c3 = mov eax,[ecx+0x14]; or dw… |
| `00a4fb60` | SetByte398To1 | likely | Trivial setter: writes 1 into the byte at +0x398 and returns (bytes c6 81 98 03 00 00 01 c3 = mov byte ptr [ecx+0x398],1; ret); caller 00ca6134 uses it from th… |
| `00a51ce0` | MatchKindLowNibble1 | likely | Predicate returning 1 when the low four bits of the first dword of the object equal 1, otherwise 0; the compiled body also loads the second dword but the +4 te… |
| `00a553e0` | IndexOfNode_Link358 | likely | Walks the chain beginning at object offset +0x2c via each node's +0x358 link and returns the zero-based position of param_2 or zero if absent. |
| `00a57b60` | CompareNumericKeyThenStringInitial | likely | Compares the numeric key at offset +0x14; when equal, obtains both strings and compares their first characters, then releases the temporary string references. |
| `00a67750` | ScalarDeletingDtor_00a66780 | likely | MSVC scalar deleting destructor: bytes 6a ff 68 f8 8e 5b 01 ... c7 44 24 14 ff ff ff ff e8 fb ef ff ff f6 44 24 1c 01 74 09 56 e8 d1 ac 31 00 83 c4 04 8b c6 ..… |
| `00a67c40` | ScalarDeletingDtor_00a66780 | likely | Second, byte-identical MSVC scalar deleting destructor for the same class as 00a67750 (the only differences in its 87 bytes are the SEH handler address 015b905… |
| `00a67ca0` | CopyRecordFields_00a67ca0 | likely | Copy constructor of the 0x360-byte ship-combat player node: copies the nine leading dwords, then three 0x54-byte members at +0x24/+0x78/+0xcc through FUN_00a58… |
| `00a68d40` | Wrapper_00a67ca0 | likely | Copy-construction helper: keeps this (the destination node) in esi, forwards the stack argument to the copy constructor FUN_00a67ca0 with this still in ecx und… |
| `00a7aaa0` | RecomputeChildSums | likely | Fetches a node through the table at this+0x278 and index list at +0x288, then sums child fields +0x18 and +0x1c for children whose field +8 is nonnegative. It … |
| `00ab6540` | FreeAlignedPairStorage | likely | Frees two aligned suballocations and their owner block, then clears the stored pointers. |
| `00adeb40` | ScalarDeletingDtor_00adeb40 | likely | Deleting-destructor shape: calls the complex destructor body 00d32f60 on this and then _free(this) when bit 0 of the flag argument is set. |
| `00b20000` | CheckThreeFlagsAndDispatch | likely | Requires bytes at param_1+0x3e828, +0x3b45d and +0x3e829 all to be nonzero; then calls virtual slot +4 on the object at +0x3b460, dispatches through 00cc4be0 w… |
| `00b20070` | AdjustThis8TailJump_0099a830 | likely | mov ecx,[ecx+8], then jumps to 0099a830; forwards this through the adjusted subobject. |
| `00b20080` | AdjustThis8TailJump_0099d310 | likely | mov ecx,[ecx+8], then jumps to 0099d310; forwards this through the adjusted subobject. |
| `00b20090` | AdjustThis8TailJump_0099d350 | likely | mov ecx,[ecx+8], then jumps to 0099d350; forwards this through the adjusted subobject. |
| `00b200a0` | AdjustThis8TailJump_0099d390 | likely | mov ecx,[ecx+8], then jumps to 0099d390; forwards this through the adjusted subobject. |
| `00b20100` | AdjustThis8TailJump_009a09f0 | likely | Raw x86 loads ECX from [ECX+8], then tail-jumps to 009a09f0. |
| `00b20120` | AdjustThis8TailJump_009a3520 | likely | mov ecx,[ecx+8], then jumps to 009a3520; forwards this through the adjusted subobject. |
| `00b23c20` | HashLookup1024BucketsThen0x2a8 | likely | Looks up the uint key param_2 in the table at this+0x578 with 1024 buckets (param_2 & 0x3ff) and chain pointers at node+0x2a8; returns 1 (nonzero) when a node … |
| `00b23c70` | LookupKeyD9CompareAndTest3b3d4 | likely | Loads the string pointer at this+0x3b430, calls FUN_00d04bf0 with key 0xd9, then calls FUN_00401940 with that result, the loaded string pointer and limit 0x10.… |
| `00b23cb0` | ForwardArgToField8_0099d2d0 | likely | Copies its one stack argument into the outgoing call slot, loads ECX from the pointer field at this+8, calls FUN_0099d2d0, then returns while popping its 4-byt… |
| `00b23d30` | IsField2d0Zero | likely | 12-byte predicate (xor eax,eax; cmp dword [ecx+0x2d0],eax; sete al; ret): returns 1 when the dword at this+0x2d0 is zero, else 0. |
| `00b23d40` | GateThenForwardField8Plus4To009a7730 | likely | If the dword at this+0x2d0 is nonzero, returns 0. Otherwise loads ECX from the pointer at this+8, pushes param_3 then param_2, and calls thunk 009a82d0; that t… |
| `00b23d70` | GateThenForwardField8Plus4To009a6ba0 | likely | If the dword at this+0x2d0 is nonzero, returns 0. Otherwise loads ECX from the pointer at this+8, pushes param_3 then param_2, and calls thunk 009a7720; that t… |
| `00b23da0` | CallField8NoArgsIfGateClear_009a82e0 | likely | 23-byte gate wrapper: returns 0 when dword [ecx+0x2d0] != 0, else loads ECX = [ecx+8], calls FUN_009a82e0 with no arguments and returns 1. |
| `00b23dc0` | CallField8NoArgsIfGateClear_0099d3d0 | likely | Same shape as 00b23da0 with a different callee: returns 0 when this+0x2d0 is nonzero, otherwise calls FUN_0099d3d0 on this->[+8] and returns 1. |
| `00b616c0` | SetFlagByte1A_1 | likely | Five-byte body (c6 41 1a 01 c3): mov byte ptr [ecx+0x1a],1 then ret; a bare this-relative flag setter with no callees and no recorded callers. |
| `00b616d0` | SetFlagByte1A_0 | likely | Five-byte body (c6 41 1a 00 c3): mov byte ptr [ecx+0x1a],0 then ret; the clearing counterpart of FUN_00b616c0, with no callees and no recorded callers. |
| `00b71cc8` | ApplyTwoKeyRecordIfAbsent | likely | Uses incoming stack/register values to look up a two-key record with FUN_00b70720. When absent, appends a node through FUN_00b702b0, initializes its two key wo… |
| `00b86d80` | SetByte0x1c | likely | Single five-byte setter: mov byte [ecx+0x1c], 1; ret, i.e. it sets the byte flag at this+0x1c and returns no value. |
| `00b87c10` | ZeroInit_0x50Struct | likely | Initializes a 0x50-byte record: zeroes the dwords at +0, +4, +8 and +0xc and memsets the 0x40 bytes from +0x10 to +0x4f via _memset, returning the object point… |
| `00bd8930` | DeletingDtorFreeIfFlag1 | likely | Deleting-destructor shape: sets SEH frame LAB_015eea48, calls 008ccc60 with the cookie XORed with a local address, then calls _free (thunk 00d82463) on its fir… |
| `00c33f80` | SubobjectScalarDeletingDtor | likely | Calls FUN_01078020 for nested-member cleanup, then frees this object if flag bit 0 is set. |
| `00c37da0` | FindListEntryByKey | likely | Scans the linked list at object offset 0xc094 for a matching key at entry offset 0x2806c, then returns that entry's field at 0x44 or zero. |
| `00c54380` | ConditionalVirtualDispatch | likely | Calls param_1's vtable slot +0x24; if the first result is nonzero, calls the same slot again and tail-jumps to 0076c590 with the second result in ECX; otherwis… |
| `00c543d0` | ScalarDeletingDtor | likely | Calls cleanup function 00cf3eb0, frees param_1 when flag bit 0 is set, and otherwise returns param_1. |
| `00c543f0` | ScalarDeletingDtor | likely | Calls cleanup function 00cf4380, frees param_1 when flag bit 0 is set, and otherwise returns param_1. |
| `00c54410` | ScalarDeletingDtor | likely | Calls cleanup function 00cf4c00, frees param_1 when flag bit 0 is set, and otherwise returns param_1. |
| `00c54430` | ScalarDeletingDtor | likely | Calls cleanup function 00cf51b0, frees param_1 when flag bit 0 is set, and otherwise returns param_1. |
| `00c54540` | StoreValueAndRequestIfMissing | likely | When +0x1ac0 is nonnull and its virtual slot +0x24 returns nonzero, stores param_2 at +0x1a9c; if helper 00770090 reports the value absent, builds a local requ… |
| `00c54650` | Field_10B0_IsNonzero | likely | Returns whether the dword at this+0x10b0 is nonzero. |
| `00c54680` | CallHelperWhenField_10B0_Nonzero | likely | Tail-jumps to 00fe3b40 only when this+0x10b0 is nonzero; otherwise returns. |
| `00c546a0` | GetSelectedPointerOrDefault | likely | If this+0x10b0 is nonzero, returns the pointer from 00fe3b50; otherwise returns the address DAT_01bee430. |
| `00c546c0` | CallHelperWhenField_10B0_Nonzero | likely | Tail-jumps to 00fe3b60 only when this+0x10b0 is nonzero; otherwise returns. |
| `00c546e0` | GetSelectedPointerOrDefault | likely | If this+0x10b0 is nonzero, returns the pointer from 00fe3b70; otherwise returns the address DAT_01bee430. |
| `00c54700` | CallHelperWhenField_10B0_Nonzero | likely | Tail-jumps to 00fe3b80 only when this+0x10b0 is nonzero; otherwise returns. |
| `00c54800` | CallWhenField_1ACC_Nonzero | likely | Calls 00cf4ff0 with param_2 only when this+0x1acc is nonzero; otherwise returns. |
| `00c548e0` | AllPresentPointersPassGate | likely | Checks pointers at +0x14ac and +0x1514..+0x1530 plus pointer groups based at +0x14b0, +0x14dc, and +0x14f8; for each nonnull pointer it requires 0076c590 to re… |
| `00c550d0` | StoreFlagAndMaybeStopObject | likely | Stores param_2 at this+0x15c8; if param_2 is nonzero, this+0x10b0 is nonnull, its byte +0x143 is zero, this+0x15c1 is zero, and this+0x1ac4 is nonnull, calls 0… |
| `00c8dbe0` | ParseDelimitedRecords | likely | Iterates a text buffer, skips # lines, parses delimited fields and numbers, updates existing tutorial entries or allocates and inserts new entries. |
| `00c90f50` | SelectTemporaryListValueByIndex | likely | Builds a temporary list of IDs from a lobby configuration value, walks that list to the index at this+0x268, and returns that selected ID or zero. |
| `00c91060` | SelectMappedListValueByIndex | likely | Gets the selected ID through FUN_00c90f50, maps it into another temporary list, and returns the indexed mapped value or zero. |
| `00c91260` | ReadMappedElementField10 | likely | Selects the configured option at the current index and returns its field at offset 0x10, or zero if no option exists. |
| `00c91290` | ReadMappedElementField0C | likely | Selects the configured option at the current index and returns its field at offset 0x0c, or zero if no option exists. |
| `00c912c0` | UpdateRecordsByLinkedKey | likely | Walks the linked records, finds matching entries by key, selects an indexed value from a mapped list, and writes that value through a helper. |
| `00cc4c10` | CallWithScopedFrameContext | likely | Creates a scoped exception-frame context, calls 006faaf0 with param_1, calls 006faac0 to clean up, and returns the byte result; the raw call setup passes the l… |
| `00cd70e0` | AdvanceNinePositiveCounters | likely | For each of nine counters, adds the supplied delta when the counter is positive, resets it above 10000, and calls a helper with the index. |
| `00cf10b0` | ThisAdjustThenJump_00a6daa0 | likely | Loads ECX from this+8 and jumps directly to 00a6daa0; it is a this-adjusting thunk with no independent call body. |
| `00cf10f0` | ThisAdjustThenJump_00cff0b0 | likely | Loads ECX from this+0x18 and jumps directly to 00cff0b0; it is a this-adjusting thunk with no independent call body. |
| `00d37930` | FormatNameOddAndLoad | likely | Thiscall with one stack argument (name): formats '%s.odd' with the name through 00402fd0, assigns it to a temporary string through 00402ce0, calls slot 0xc of … |
| `00f94990` | StdStringPairTidy | likely | Fastcall on an object holding two std::string members: the first has its buffer word at +4 and capacity at +0x18, the second has its buffer at +0x20 and capaci… |
| `00f94bd0` | VectorArrayTidy5 | likely | Frees the heap buffers of five std::vector-layout members whose begin words are at +8, +0x1c, +0x30, +0x44 and +0x58 (0x14 bytes apart) through _free and zeroe… |
| `0105c1b0` | ResolvePairToStateOrNodeField8 | likely | Thiscall with two words (param_2, param_3) and a flag byte (param_4). If the byte at [this+0x79] is zero: returns 3 when ([this+0x30], [this+0x34]) equals (par… |
| `0105cc30` | RecordDtorBodyThreeIStrings | likely | Fastcall (ECX=this), EH-framed. Zeroes [this+0xd0]; stores the SoeUtil::IString<char> vftable at +0xf4 and, if [this+0x100] is positive, atomically decrements … |
| `0105cf20` | RecordCtorThreeIStringsB | likely | Fastcall (ECX=this), EH-framed. Zeroes dwords +0 through +0x2c, +0x30 through +0x48 and the byte at +0x4c; calls 00762770 (the receiver is not shown in the pse… |
| `0105d5f0` | RecordCopyCtorThreeIStrings | likely | Thiscall with one stack argument (source record). Zeroes dwords +0..+0x1c; stores the SoeUtil::IString<char> vftable at +0x60 (storage &DAT_01cb70a0 at +0x64) … |
| `0105d800` | RecordAssignFromSource | likely | Thiscall with one stack argument (source record). Copies source dwords +0x20..+0x34, +0x40..+0x5c, +0xc0..+0xcc, +0xd8, +0xdc, +0x74..+0x7c, +0xa0, +0xf0, the … |
| `0105e6a0` | RecordCtorFixtureActionsAndStrings | likely | Fastcall (ECX=this), EH-framed. Zeroes dwords +0x08, +0x0c, +0, +4, +0x18, +0x1c, +0x10, +0x14, +0x24, +0x28, +0x2c, +0x30, +0x34 and +0x38; calls 00402a10 thr… |
| `010611b0` | CtorWordsThenIconDataAt0xc | likely | Thiscall with two stack arguments (ret 8). Stores the first argument at +0, the second at +4 and zero at +8, then constructs an IconData sub-object at this+0xc… |
| `010612a0` | CtorFromRefsStoreWordAt0x20 | likely | Thiscall with three pointer arguments. Calls 010611b0 with two values read through the third and fourth arguments (*param_3, *param_4), which runs on this obje… |
| `01061ee0` | ResetIStringAndClearArrays | likely | Fastcall with the object in ECX (no stack arguments). If the word at +0xc is positive, decrements the header word at buffer-4 (buffer at +4) under LOCK and, wh… |
| `010621f0` | EmbeddedArraysDtorBody_34_98 | likely | Fastcall with the holder in ECX. Runs the Array<PromotionalAward::Feature,4,1> destructor body 01061f70 on the sub-object at +0x98, the Array<PromotionalAward:… |
| `010623e0` | EmbeddedArraysCtorBody_34_98 | likely | Thiscall with one stack argument (ret 4). Calls 0070bcc0 with that argument (the IStringFixed<char,32> initialiser, accepted label), constructs the ItemBundle … |
| `01062450` | EmbeddedArraysDefaultCtor_34_98 | likely | Fastcall with the object in ECX (no stack arguments). Calls 00703540 (fixed-string constructor, accepted label) on the object, constructs the ItemBundle array … |
| `01062770` | ZeroWords4To14 | likely | Fastcall with the object in ECX (no stack arguments). Zeroes the five words at +4, +8, +0xc, +0x10 and +0x14. Plain ret. |
| `010627f0` | ZeroInit_0x60Bytes | likely | Fastcall with the object in ECX (no stack arguments). Zeroes the words at +4, +8 and +0xc, zeroes the 0x50 bytes from +0x10 with _memset, zeroes the word at +0… |
| `01062c40` | ReadTextThenParseInt | likely | Three arguments. Fills a 2048-byte stack buffer with its first character cleared, through 006fba40 with the first argument, the buffer, size 0x800, the third a… |
| `010640e0` | ReadTextThenParseLong | likely | Three arguments (cookie-framed). Clears the first character of a 2048-byte stack buffer, fills it through 006fb560 with the first argument, the buffer, size 0x… |
| `01064180` | ReadTextThenParseLong | likely | Three arguments (cookie-framed). Same body as 010640e0: fills a 2048-byte stack buffer through 006fb560 (first argument, buffer, 0x800, third argument, flag 1 … |
| `01064920` | ReadTextThenParseLong | likely | Three arguments (cookie-framed). Same body as 010640e0 and 01064180: fills a 2048-byte stack buffer through 006fb560 (first argument, buffer, 0x800, third argu… |
| `0106e6f0` | RecordWithTwoMapsCtor | likely | Constructs a guild record: zeroes the leading dwords, installs IString<char> members at +0x28, +0x38, +0x6c and +0x270 (empty static buffer DAT_01cb7600), stor… |
| `01070a40` | CopyFieldsFromSourceRecord | likely | Copy constructor for a large record (source param_2, null gives zero values): copies dwords source+0x24, +0x30, +0x28 and +0x38 to +0x8, +0xc, +0x10 and +0x14;… |
| `010712f0` | ReleaseOwnedPtrSlot2 | likely | If the pointer stored at *ECX is nonnull, calls its virtual slot +8 with no arguments, then zeroes the dword at this+0xc. |
| `0107c4f0` | InitZeroTable4000 | likely | Fastcall (this in ECX, returns this): zeroes the dwords at +4, +8 and +0xc, clears the 4000 bytes starting at +0x10 with _memset, then zeroes the dword at +0. |
| `0107c540` | InitZeroRecord3C | likely | Fastcall (this in ECX, no return value): zeroes a 0x3c-byte record: dwords at +0x0 through +0x20, +0x28, +0x2c, +0x30, +0x34 and +0x38, and the bytes at +0x24 … |
| `0107cf50` | InitZeroTable128 | likely | Fastcall (this in ECX, returns this): zeroes the dword at +4 and the 0x80 bytes at +8 with _memset, then zeroes the dword at +0. |
| `0108b3e0` | EmptyClassScalarDeletingDtor | likely | Thiscall with a flag byte (ret 4 not shown): SEH framed, calls the stub 0108c4c0 (an empty body) and calls _free on this when bit 0 of the flag is set. Returns… |
| `01098b60` | ScalarDeletingDtorWrapper_010fdad0 | likely | Thiscall with one stack arg (flags byte); calls FUN_010fdad0 with ECX=this, then when flags bit 0 is set calls _free(this); returns this in EAX (27 bytes; the … |
| `012b7fc0` | ScalarDeletingDtor_TreeMembers | likely | thiscall scalar deleting destructor: calls the member teardown 012b7f30, then frees the object with _free only when bit 0 of the second argument is set; return… |
| `012e1bf0` | KindStringOrIndexed | likely | Writes text for the value object at ECX into the out string: when the int at +0x20 is below 3 it copies the string at +0xc; otherwise it calls 012323e0(out, ki… |
| `01333870` | StringbufStrForward | likely | Calls 0126dec0 (basic_stringbuf::str) with the first argument, which is a string result pointer, and returns that argument. |
| `01357800` | CleanupMemberAt0C | likely | Installs an SEH frame and calls 01357060 (ResetBufferPointerAt8) with ECX set to this+0xc, so the buffer at this+0x14 is freed and its words cleared; the cooki… |
| `0135eea0` | DeletingDtorBody0135cbb0 | likely | Scalar deleting destructor shape (thiscall, ECX = object, one stack argument = flags, ret 4): calls the body 0135cbb0 with ECX = object, then frees the object … |
| `0135eec0` | DeletingDtorBody0135cc00 | likely | Scalar deleting destructor shape (thiscall, ECX = object, one stack argument = flags, ret 4): calls the body 0135cc00 with ECX = object, then frees the object … |
| `01367c60` | Factory_0x28_CtorThreeDwordsA | likely | Allocates 0x28 bytes with operator new and, when non-null, constructs the object through 01367000 with the five stack arguments; returns the pointer (null when… |
| `01367fe0` | Factory_0x28_CtorThreeDwordsB | likely | Allocates 0x28 bytes with operator new and, when non-null, constructs the object through 01367080 with the five stack arguments; returns the pointer (null when… |
| `01368d10` | Factory_0x20_CtorThreeDwordsCopySub | likely | Allocates 0x20 bytes with operator new and, when non-null, constructs the object through 01368180 with the five stack arguments; returns the pointer (null on f… |
| `013f7ca0` | CopyAssignPairVec38 | likely | Stdcall-style copy-assignment (ret 4; ECX = destination): copies two dwords (+4 and +8) from the source, then copy-assigns the 56-byte-element vector at source… |

### `unknown.array`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f6c00` | CopyDword256ToVector | likely | Reserves 0x100 dwords in the vector at param_2 (012f6b00), then appends the 256 dwords stored at this+0x208 of the object in ECX: each element goes to the vect… |

### `unknown.datetime`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fc90c0` | FormatIso8601Utc | confirmed | Plain function: four arguments (two forwarded to 007026b0, which fills six date/time fields from them, the output string object param_3 and a value param_4). Z… |

### `unknown.deck`

**Deck** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ce3d0` | Deck::TraceDrawDeck | confirmed | thiscall(ECX = deck; one stack arg = trace sink): calls virtual slot +0x48 of the deck twice to get two ranges, requires the range pointers to be ordered, then… |

### `unknown.dispatch`

**CWMainController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01314350` | CWMainController::CWMainController | confirmed | Thiscall constructor. Calls base constructor 0133e060 on its argument, installs CWMainController::vftable at +0 and +0x10, then initializes many fields: dword … |
| `013144e0` | CWMainController::ThunkSub10ToScalarDeletingDtor | confirmed | Adjustor thunk in the secondary CWMainController vtable (+0x10, slot [0]): subtracts 0x10 from ECX and jumps to 01315ab0 (CWMainController::ScalarDeletingDtor). |

### `unknown.dtor`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7340` | VectorDeletingDtor0x48 | likely | Vector deleting destructor for an array of 0x48-byte elements: when bit 1 of the flag is set, runs the element destructor loop (_eh_vector_destructor_iterator_… |
| `01415970` | DtorSubobjectAt4 | likely | Cleanup routine that is an EH unwind target. Its prologue installs an SEH frame and saves ECX; it calls FUN_01300cd0 with ECX = this+4 (add ecx,4 before the ca… |
| `01415df0` | ScalarDeletingDtorSubobjectAt4 | likely | Scalar deleting destructor shape: saves ECX as this, calls FUN_01300cd0 with ECX = this+4, tests bit 0 of the stack flag byte and calls free(this) when it is s… |

### `unknown.unclassified`

**CWDeck** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01408c30` | CWDeck::GetClassName | confirmed | Virtual slot 11 (+0x2c) of CWDeck::vftable. Returns the address of the literal 'CWDeck' (mov eax, imm32; ret). |
| `01408fb0` | CWDeck::Write | confirmed | Virtual slot 10 (+0x28) of CWDeck::vftable. If the argument is zero it calls vtable slot 1 (+0x4) to obtain the stream; otherwise it calls vtable slot 2 (+0x8)… |
| `01409090` | CWDeck::~CWDeck | confirmed | Scalar deleting destructor (vtable slot 0 of CWDeck::vftable): stores CWDeck::vftable, calls 013ce5b0 on this, then calls _free(this) when bit 0 of the second … |
| `01408c40` | CWDeck::GetSharedValue | likely | Virtual slot 12 (+0x30) of CWDeck::vftable. Returns the dword at the global DAT_01cbe23c (mov eax,[DAT_01cbe23c]; ret). |
| `01408c50` | CWDeck::SetSharedValue | likely | Virtual slot 13 (+0x34) of CWDeck::vftable. Stores its stack argument into the global dword DAT_01cbe23c and returns. |
| `01408c70` | CWDeck::DtorBody | likely | Stores CWDeck::vftable into this and calls 013ce5b0 (unlabeled base step) on this with the EH frame set up; returns. |
| `01408ce0` | CWDeck::CreateCopyFields | likely | Virtual slot 29 (+0x74) of CWDeck::vftable. Allocates 0x94 bytes with operator new (00d833a6); when non-null it constructs a CWDeck in the block through 01408b… |
| `01408f70` | CWDeck::Read | likely | Virtual slot 9 (+0x24) of CWDeck::vftable. Calls vtable slot 4 (+0x10) with the stream (header read; returns false on failure), reads one value through 013cef2… |
| `01409000` | CWDeck::Create | likely | Allocates 0x94 bytes with operator new (00d833a6). Returns 0 when the allocation fails; otherwise constructs a CWDeck in the block through 01408bb0 and returns… |

**(no class)** (102)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d6c750` | CopyTwoStringFixed8FieldsAndTail | confirmed | Copies two fixed-8 string members through FUN_00b2a940, writes their vtables, then copies dwords at source offsets +0x38,+0x3c,+0x40 and one byte at +0x44. |
| `00d6c800` | ConstructHeaderAndString8String48 | confirmed | Stores two supplied dwords at offsets 0 and 4, constructs a fixed-8 field at +8 from param_4, and constructs a fixed-48 field at +0x24 from param_5. |
| `00d6e630` | CopyStringFixed8And48Record | confirmed | Copies two leading dwords, copy-constructs a fixed-8 field at +8 and a fixed-48 field at +0x24 from the source, and returns the destination. |
| `00912450` | DtorMemberCleanupE8E30 | likely | thiscall with no arguments (ECX = this, saved). Under an SEH frame with a cookie check: calls thiscall FUN_00905ca0 with ECX = this+0xe8, then thiscall FUN_010… |
| `0096e490` | UpdateMotionAndActorState | likely | Scales the frame delta, applies status-bit and related-object checks, scales stored components by a capped delta, may invoke a movement helper, writes a transf… |
| `0096f6a0` | IndexGuardThenCall_006f9e60 | likely | Compares count at this+0x78 with the index and wrapped index+1; calls 0096b1d0 only when count<=index and count<index+1 under signed comparisons, then calls 00… |
| `009708f0` | BuildListArgumentAndCall_0078b390 | likely | When param_2 is nonzero, builds a temporary IString list and argument record, calls 0078b390, conditionally updates local state fields from its result, then de… |
| `00970b60` | AccumulateTimersAndDispatchMotion | likely | Requires a valid object context and matching identifier, accumulates timer/state fields, tests status conditions, computes normalized vectors for selected para… |
| `009722d0` | TransitionIndexedActorState | likely | Checks two object contexts, validates a stored index against a model count, updates a mode field, dispatches helper calls, computes scale/transform data, and a… |
| `00972c90` | DispatchLookupValueWithCounter12 | likely | Gets a nonzero context through a virtual call, ensures helper state, looks up a value with key 0xc, adds the global counter, increments that counter modulo 12,… |
| `00972cf0` | DispatchLookupValueForKey13 | likely | Gets a nonzero context through a virtual call, ensures helper state, looks up the value for key 0x13, and passes the lookup result plus fixed sentinel/zero arg… |
| `0097b960` | RoleTemplateTextBuild | likely | Builds the text of a role object record (fields at +4 text, +0x14, +0x20, +0x24, +0x2c, +0x30) through a template family selected by the kind at +0x38: 0 calls… |
| `0097c790` | AllocAndStoreObjectAt208 | likely | Allocates a 600-byte block with FUN_00d833a6. When the allocation is non-null it calls FUN_00c29750 (argument this, as Ghidra shows) and stores the result at t… |
| `0097c810` | AllocAndConstructObject620 | likely | Allocates a 0x26c (620)-byte block with FUN_00d833a6. When non-null it calls FUN_00c29bb0 with its three stack arguments and stores the result in *param_1; oth… |
| `0098dce0` | ClearByte25C | likely | Stores zero in the byte at this+0x25c and returns. |
| `0098faa0` | LoadField37CThenJump00a6daa0 | likely | Loads the dword at this+0x37c into ECX, then jumps to 00a6daa0. |
| `0098fab0` | LoadField378ThenJump00a6dad0 | likely | Loads the dword at this+0x378 into ECX, then jumps to 00a6dad0. |
| `0098fac0` | SetNestedField25CFromArgument | likely | When the supplied value is nonzero, stores it at nested object offset +0x25c beneath this+0x390 and invokes helper 00c2c100. |
| `0098faf0` | LoadField37CThenJump00a6e7b0 | likely | Loads the dword at this+0x37c into ECX, then jumps to 00a6e7b0. |
| `00a79d20` | ListContainsPair | likely | Walks the list that starts at 00984350 with 009849e0 as next, and returns 1 as soon as a node whose first field equals argument 1 and whose second field equals… |
| `00a8ffe0` | FindTwoEntriesAndPlaceRecord | likely | Searches two keyed lists for fixed key values, passes two looked-up values and reads a field from the supplied record, and forwards the resulting arguments to … |
| `00a91800` | RegisterAndPlaceIndicatorFromStrings | likely | Builds a temporary string from manager-provided values, resolves two registry entries, and forwards those values to 00a8fc80 for placement. |
| `00ad11b0` | GetHelperByteIfFieldSet | likely | Tests this+0x5ef48; when nonzero tail-jumps to 00f9b060 with that pointer, otherwise returns zero. |
| `00ad12f0` | DestroyObjectAndMaybeFree | likely | Calls 00f9af70, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
| `00ae9800` | DispatchSlot28OrZero | likely | If the pointer at this is nonnull, tail-jumps to that object's vtable slot +0x28; otherwise returns floating-point zero. |
| `00ae9820` | SetByte284 | likely | Stores the supplied byte at this+0x284 and returns. |
| `00b58830` | CopyRecordAndLinkedEntries | likely | Copies many scalar fields from param_2 to param_1, clears and clones the linked chain at field 0x3c, and copies two fixed groups of fields. |
| `00c9a0f0` | MatchOneOfShipStateIds | likely | Returns one for five explicit values 0x1bc9, 0x1bcb, 0x1bd1, 0x1bcf, and 0x1bd0, or for values accepted by 00c9a0a0. |
| `00cdfab0` | CopyRecordFields_4_to_64 | likely | Copies dwords from param_2 offsets +4 through +0x54 and +0x60/+0x64 to this, and copies one byte from +0x58. |
| `00d694d0` | AllocateAndConstruct_00d683d0 | likely | Pushes allocation size 0x188 to FUN_00d833a6, then calls FUN_00d683d0 with the returned pointer when non-null; returns that result or 0. |
| `00d69550` | AllocateAndConstruct_00d684a0 | likely | Pushes allocation size 0xb0 to FUN_00d833a6, then calls FUN_00d684a0 with the returned pointer when non-null; returns that result or 0. |
| `00d695d0` | AllocateAndConstruct_00d68a50 | likely | Pushes allocation size 0x90 to FUN_00d833a6, then calls FUN_00d68a50 with the returned pointer when non-null; returns that result or 0. |
| `00d69650` | AllocateAndConstruct_00d68570 | likely | Pushes allocation size 0xd0 to FUN_00d833a6, then calls FUN_00d68570 with the returned pointer when non-null; returns that result or 0. |
| `00d696d0` | AllocateAndConstruct_00d68640 | likely | Pushes allocation size 0x110 to FUN_00d833a6, then calls FUN_00d68640 with the returned pointer when non-null; returns that result or 0. |
| `00d69750` | AllocateAndConstruct_00d68710 | likely | Pushes allocation size 0x110 to FUN_00d833a6, then calls FUN_00d68710 with the returned pointer when non-null; returns that result or 0. |
| `00d697d0` | AllocateAndConstruct_00d687e0 | likely | Pushes allocation size 0xf8 to FUN_00d833a6, then calls FUN_00d687e0 with the returned pointer when non-null; returns that result or 0. |
| `00d69850` | AllocateAndConstruct_00d688b0 | likely | Pushes allocation size 0xc8 to FUN_00d833a6, then calls FUN_00d688b0 with the returned pointer when non-null; returns that result or 0. |
| `00d698d0` | AllocateAndConstruct_00d68980 | likely | Pushes allocation size 0xc8 to FUN_00d833a6, then calls FUN_00d68980 with the returned pointer when non-null; returns that result or 0. |
| `00d69950` | AllocateAndConstruct_00d68b20 | likely | Pushes allocation size 0x98 to FUN_00d833a6, then calls FUN_00d68b20 with the returned pointer when non-null; returns that result or 0. |
| `00d699d0` | AllocateAndConstruct_00d68bf0 | likely | Pushes allocation size 0xb0 to FUN_00d833a6, then calls FUN_00d68bf0 with the returned pointer when non-null; returns that result or 0. |
| `00d69a50` | PruneTimedLinkedEntries | likely | Walks from a list head, compares each head timestamp with current time and 60000 ticks, then stops at the first non-expired entry; expired nodes trigger a vtab… |
| `00d69b50` | PruneTimedLinkedEntriesAndReleaseRefs | likely | Walks a linked list from param_2+4, stops when the head is younger than 60000 ticks, otherwise calls its vtable callback, cleans a field, removes the node thro… |
| `00d6a010` | CleanTwoMemberListsAt30And3C | likely | Calls FUN_00d69a50 with param_1+0x30 and FUN_00d69b50 with param_1+0x3c, then returns without a value. |
| `00d6a920` | CheckTimerAndPruneTwoLists | likely | Calls FUN_00f60dc0 with 100, initializes timer globals once, samples time, and when elapsed ticks exceed 299 calls both list-pruning helpers and stores the new… |
| `00d6b7c0` | AppendLinkedNodeAndHashBucket | likely | Links a node after the current tail or sets the head, updates the tail, stores its key and prior bucket head, inserts it into bucket key&7, and increments the … |
| `00d6b800` | AllocateAndConstruct_00d6b540 | likely | Pushes allocation size 0x1d8 to FUN_00d833a6, then calls FUN_00d6b540 with the returned pointer when non-null; returns that result or 0. |
| `0103a350` | DtorBody_IString2c_Loggable14 | likely | Destructor body without a vtable store at +0: calls the IString<char> destructor 00722bf0 on this+0x2c, then stores Loggable::vftable into the dword at this+0x… |
| `01045d50` | LazyGlobal38_GetOrCreate | likely | No args (plain ret). If the global DAT_01cb6910 is null, allocates 0x38 bytes with 00d833a6 (operator new); on success zeroes 12 dwords at offsets +0, +4, +8, … |
| `01083010` | FieldCopyAssign_0x6cList | likely | Thiscall, one stack arg (source record), returns this (ret 4). Copies dwords +0x4..+0x2c, +0x34, +0x38, +0x1c and byte fields +0x30/+0x31/+0x3c from the source… |
| `01083180` | SetCalendarFieldsToEpoch40 | likely | Thiscall with one pointer arg to seven dwords. Copies those seven dwords into this+0x1c..+0x34, calls 00702760 (calendar-to-time conversion) on that block with… |
| `010831c0` | SetEpochAt38 | likely | Thiscall with two dword args (low and high halves of a 64-bit time). Stores them at this+0x38/+0x3c, then calls 007026b0 with the 64-bit value, output pointer … |
| `010831f0` | SetEpochAt40 | likely | Thiscall with two dword args (low and high halves of a 64-bit time). Stores them at this+0x40/+0x44, then calls 007026b0 with the 64-bit value, output pointer … |
| `010833e0` | CopyAssignRecordFields_0x64 | likely | Thiscall with one stack arg (source record), ret 4. Copies dwords +0x8, +0xc, +0x10, seven dwords at +0x18..+0x30, seven dwords at +0x34..+0x4c, dwords +0x50, … |
| `01083450` | FormatEnableDisableDateTimes | likely | Thiscall with one stack arg (output string). Builds two YYYY-MM-DDTHH:MM:SSZ strings with 00402fd0 from six dwords at +0x0..+0x14 and from six dwords at +0x1c.… |
| `01083590` | FormatIdActivityGmtTime | likely | Thiscall with one stack arg (output string). Calls 01083450 into a local string, then formats 'Id=%d, activityId=%d, gmtTime=(%s), useServerTime=%d, featuredRe… |
| `01083c40` | InitTwoIStringMembers | likely | Fastcall, ECX = this, no stack args, no return value. Zeroes +0x0..+0x1c, +0x38, +0x3c, +0x50..+0x64. Stores SoeUtil::IString<char>::vftable at +0x30 and +0x48… |
| `01083d30` | IStringPairRelease_0x30_0x48 | likely | Fastcall, ECX = this, no stack args, no return value. Stores SoeUtil::IString<char>::vftable at +0x48; if the count at +0x54 is positive it atomically decremen… |
| `01083f20` | CtorFromSourceRecord88 | likely | Thiscall with three stack args (ret 0xc). Zeroes +0x0..+0xc, stores the first two args at +0x10 and +0x14, copies dwords from the optional source (param_4; zer… |
| `01086c20` | CtorWithItemRecordAt20 | likely | Fastcall, ECX = the owning object, no stack args, no return value. Zeroes +0x0, +0x24, +0x28, +0x30, +0x34, sets +0x8, +0xc, +0x10, +0x18, +0x1c to shared DAT … |
| `01086d80` | ScalarDeletingDtor_ItemRecordAt20 | likely | Thiscall with one byte flag arg. Stores ItemRecord::vftable at this+0x20 (an embedded ItemRecord sub-object of the owner), and if bit 0 of the flag is set call… |
| `010886e0` | IStringMemberRelease_0c | likely | Fastcall, ECX = this, no stack args, no return value. Stores SoeUtil::IString<char>::vftable at +0xc; if the count at +0x18 is positive, atomically decrements … |
| `012cf1e0` | Vector38_RemoveByKey | likely | Builds a key from param_2 and param_3 through 012cca70, locates an element of the 0x38-element vector at param_1 (begin at +4, end at +8) through 012cd1b0, and… |
| `012d00f0` | AddPairAt10 | likely | Thiscall forwarding wrapper: calls 012d0050 with ECX=this+0x10 and the two stack arguments; returns nothing. |
| `012d0110` | AddPairAt20 | likely | Thiscall forwarding wrapper: calls 012d0050 with ECX=this+0x20 and the two stack arguments; returns nothing. |
| `012d0810` | SendCommandToConnection | likely | Reads this+8 (the connection) into ECX and tail-jumps to 012f0a00 when it is non-null; otherwise it returns without framing anything (ret 4). |
| `01409070` | RegisterFactory_Id0x222e2 | likely | Static initialiser: calls 013ce130(0x222e2, 01409000) and 013ce230(0x222e2, 01409000), registering the factory 01409000 under key 0x222e2. |
| `01416380` | TeardownSubobjectAt4CdeclArg | likely | Cleanup routine that takes its object pointer from the first stack argument (read at [esp+0x14] after its own SEH frame setup). It calls FUN_01300cd0 with ECX … |
| `014164a0` | CtorSubobjectAt4FromParam3 | likely | Thiscall with two stack arguments (ret 8). Stores the dword at *param_2 into this[0], then calls FUN_0127a3e0 with ECX = this+4 and the third argument (param_3… |
| `01416570` | CopyCtorDwordAndSub4 | likely | Thiscall with one stack argument (src, ret 4). Copies the dword at *src into this[0], then calls FUN_0127a3e0 with ECX = this+4 and the argument src+4. Saves a… |
| `014165d0` | CopyCtorDwordAndSub4 | likely | Same body as 01416570: copies the dword at *src into this[0], calls FUN_0127a3e0 with ECX = this+4 and the argument src+4, and returns this in EAX (thiscall, o… |
| `014167a0` | TeardownSubobjectAt4StdcallArg | likely | Stdcall-style (ret 4) teardown: takes the object pointer from its first stack argument, calls FUN_01300cd0 with ECX = object+4, restores ExceptionList and retu… |
| `014167f0` | CopyCtorIfNonNull | likely | Cdecl-style, two stack arguments. If the first argument is nonzero it calls FUN_01416570 with ECX = the first argument and the second argument as its stack arg… |
| `01416a30` | DestroyRange16Sub4 | likely | Cdecl, two stack arguments (first, last). Loops over 16-byte elements from first to last and for each calls FUN_01300cd0 with ECX = element+4 (the +4 sub-objec… |
| `01416a90` | WrapperCopyCtorIfNonNull | likely | Stdcall forwarder (ret 8). Pushes its two stack arguments in order and calls FUN_014167f0 (the conditional copy constructor), then pops 8 bytes. |
| `01416ab0` | CopyCtorRange16 | likely | Cdecl, three stack arguments (first, last, dst). For each 16-byte element from first to last it calls WrapperCopyCtorIfNonNull (01416a90) with (dst, element) a… |
| `01416b50` | CopyCtorRange16 | likely | Same body as 01416ab0: for each 16-byte element from first to last calls WrapperCopyCtorIfNonNull (01416a90) with (dst, element) and advances dst by 0x10; retu… |
| `01416c50` | LowerBoundIterKey16 | likely | Thiscall with two stack arguments (out, key). Reads the list header at this: first = [this+4], last = [this+8], and checks last >= first (calls 00d83c2d otherw… |
| `01416cc0` | LowerBoundIterKey16 | likely | Same body as 01416c50 except that it calls FUN_01416700 (the duplicate lower-bound search); writes proxy = this and the search result into the out buffer. |
| `01416db0` | FillCopyCtor16 | likely | Cdecl, three stack arguments (dst, count, src). Runs count times: calls WrapperCopyCtorIfNonNull (01416a90) with (dst, src) and advances dst by 0x10. The sourc… |
| `01416f70` | WrapperDestroyRange16 | likely | Thiscall with two stack arguments (first, last), ret 8. Forwards first and last to DestroyRange16Sub4 (01416a30) as its cdecl arguments and returns; the this v… |
| `01417310` | WrapperFillCopyCtor16Ret | likely | Thiscall with three stack arguments (dst, count, src), ret 0xc. Calls FUN_01416db0 (FillCopyCtor16) with those arguments and returns dst + count * 0x10 in EAX,… |
| `01418110` | AllocAndConstruct0x18 | likely | Allocates 0x18 bytes through FUN_00d833a6 and, when the allocation is nonnull, runs FUN_01417e60 on the block and returns the constructor result in EAX. Return… |
| `01418170` | RegisterFactory0x179 | likely | Pushes 0x179 and the address of FUN_01418110 and calls FUN_013ce130 with them (stdcall, the callee pops both). It passes the returned value in ECX to FUN_013ce… |
| `01418bc0` | AllocAndConstruct0x64 | likely | Allocates 100 bytes through FUN_00d833a6 and, when the allocation is nonnull, calls FUN_01418b20 on the block and returns its result in EAX. Returns 0 when the… |
| `01418c20` | RegisterFactory0x222e5 | likely | Pushes the address of FUN_01418bc0 and the id 0x222e5, calls FUN_013ce130 (stdcall, the callee pops both), passes its return value in ECX to FUN_013ce230 with … |
| `01418e00` | HandleAt48ResolvedIfFlag44 | likely | Fastcall with ECX = object. Resolves the handle at object+0x48 through FUN_01357a00 and returns the resolved element when its byte at +0x44 is nonzero, otherwi… |
| `01418e20` | ResetFlags68To69 | likely | Fastcall with ECX = object. Stores 0 into the byte at +0x69 and the byte at +0x68, stores -1 into the dword at +0x64, and returns nothing (bytes: xor al,al; mo… |
| `01418e50` | PredicateByte69AndDword64 | likely | Fastcall with ECX = object. Returns 1 in AL when the byte at +0x69 is nonzero and the dword at +0x64 is nonzero, otherwise 0 (cmp byte [ecx+0x69],0; je; cmp dw… |
| `01418ea0` | CopySub58Into | likely | Fastcall with ECX = object and one stack argument (the destination), ret 4. Pushes object+0x58 and calls FUN_013010e0 with ECX = the destination and that pushe… |
| `014192e0` | AllocAndConstruct0x108PlusSub8 | likely | Allocates 0x108 bytes through FUN_00d833a6 and, when the allocation is nonnull, calls FUN_01418c40 on the block. Returns that result plus 8 in EAX when nonzero… |
| `01419360` | RegisterFactory0xa9 | likely | Pushes the address of FUN_014192e0 and the id 0xa9, calls FUN_013ce130 (stdcall), passes its return value in ECX to FUN_013ce230, then returns with a plain ret… |
| `01419f30` | InitRecord12 | likely | Thiscall with ECX = record; two stack arguments (ret 8). Stores the first argument at +0, the second at +4 and zero at +8, and returns the record in EAX. |
| `01419f50` | DtorClearWord0 | likely | Fastcall with ECX = object. Stores 0 into the first dword of the object (c7 01 00 00 00 00; ret) and returns nothing. |
| `0141a370` | AllocAndConstruct0x24 | likely | Allocates 0x24 bytes through FUN_00d833a6 and, when the allocation is nonnull, calls FUN_0141a060 (GameCommand_ReadyForStartOfGame constructor) on the block an… |
| `0141a3d0` | RegisterFactory0x75 | likely | Pushes the address of FUN_0141a370 and the id 0x75, calls FUN_013ce130 (stdcall), passes its return value in ECX to FUN_013ce230, then returns with a plain ret… |
| `0141a630` | SetSub30FromArg | likely | Thiscall with one stack argument. When the argument is zero it calls vtable slot 0 of the object at this+0x30 with 1 (a scalar deleting destructor), clears thi… |
| `0141a6e0` | SetSub34FromArg | likely | Same body as 0141a630 on the sub-object at this+0x34: releases it through vtable slot 0 with 1 when the argument is zero, otherwise allocates 0xc bytes when +0… |
| `0141a790` | SetSub30DwordTag2 | likely | Thiscall with one stack argument (a dword). Allocates the 12-byte sub-object at this+0x30 through FUN_012fa910 when that word is zero, calls FUN_01300680 with … |
| `0141a810` | SetSub34DwordTag2 | likely | Thiscall with one stack argument (a dword). Same body as 0141a790 on the sub-object at this+0x34: allocates it when zero, calls FUN_01300680 with 2, then store… |
| `0141ab30` | AllocAndConstruct0x38 | likely | Allocates 0x38 bytes through FUN_00d833a6 and, when the allocation is nonnull, calls FUN_0141a480 (GameCommand_StateSpecificMessage constructor) on the block a… |
| `0141ab90` | RegisterFactory0x3f | likely | Pushes the address of FUN_0141ab30 and the id 0x3f, calls FUN_013ce130 (stdcall), passes its return value in ECX to FUN_013ce230, then returns with a plain ret… |
| `01437df0` | ResetOwnedVectorAndString | likely | Fastcall (ECX = object). Calls the vtable slot 0 with flag 1 (scalar deleting destructor) on each non-null element of the vector at +0x3c..+0x40, clears the ve… |

## Source shards

[[S0007 0041e58b-00422a66|S0007]] (1), [[S0008 00422dcc-00426a69|S0008]] (1), [[S0040|S0040]] (3), [[S0041|S0041]] (63), [[S0048|S0048]] (8), [[S0069|S0069]] (7), [[S0099|S0099]] (2), [[S0113|S0113]] (77), [[S0116 0065f895-00664515|S0116]] (5), [[S0130|S0130]] (1), [[S0141|S0141]] (3), [[S0144 007079d0-0070c9f0|S0144]] (102), [[S0145 0070ca40-00711ff0|S0145]] (5), [[S0149 00719de0-0071efe0|S0149]] (7), [[S0152|S0152]] (39), [[S0156|S0156]] (7), [[S0162|S0162]] (17), [[S0163|S0163]] (71), [[S0164|S0164]] (37), [[S0166|S0166]] (11), [[S0167|S0167]] (27), [[S0168|S0168]] (10), [[S0170|S0170]] (60), [[S0171|S0171]] (11), [[S0173|S0173]] (1), [[S0176|S0176]] (5), [[S0178 007bd5b0-007c2fe0|S0178]] (37), [[S0192 00805a50-0080a790|S0192]] (21), [[S0195|S0195]] (3), [[S0196 0081b7d0-00821370|S0196]] (1), [[S0197 00821940-00827640|S0197]] (4), [[S0202 0083ea10-00844c00|S0202]] (4), [[S0217|S0217]] (12), [[S0219 0089d990-008a3c80|S0219]] (3), [[S0220 008a3ce0-008a8500|S0220]] (124), [[S0221 008a86b0-008b1d80|S0221]] (60), [[S0222 008b1e00-008b7730|S0222]] (5), [[S0223 008b7790-008bcf40|S0223]] (26), [[S0225 008c52a0-008ca980|S0225]] (6), [[S0227 008d0920-008d5c60|S0227]] (4), [[S0231|S0231]] (3), [[S0232 008e82e0-008ec1d0|S0232]] (3), [[S0233 008ec2a0-008f0cc0|S0233]] (8), [[S0236|S0236]] (7), [[S0237 008ff3d0-00904e70|S0237]] (5), [[S0238|S0238]] (10), [[S0239 0090a890-009105b0|S0239]] (10), [[S0240 009105e0-009161d0|S0240]] (5), [[S0242 0091af50-009200d0|S0242]] (2), [[S0243 00920130-00925ec0|S0243]] (1), [[S0244 00925fe0-0092b110|S0244]] (6), [[S0252|S0252]] (7), [[S0253|S0253]] (7), [[S0254 00954ec0-0095a710|S0254]] (42), [[S0255 0095a840-00960520|S0255]] (13), [[S0256 009606e0-00965340|S0256]] (12), [[S0257|S0257]] (8), [[S0258 00969a10-0096de80|S0258]] (22), [[S0259 0096e490-00972d30|S0259]] (7), [[S0261 00976740-0097c910|S0261]] (19), [[S0263 00982f10-009897a0|S0263]] (15), [[S0264 00989810-0098eae0|S0264]] (2), [[S0265 0098ec00-00995660|S0265]] (7), [[S0266 00995720-0099c9a0|S0266]] (49), [[S0267 0099ca10-009a1c60|S0267]] (1), [[S0269 009a6cf0-009aceb0|S0269]] (5), [[S0270 009acf20-009b2140|S0270]] (18), [[S0273 009bd650-009c2c50|S0273]] (9), [[S0274 009c2cc0-009c8d80|S0274]] (3), [[S0275 009c8e90-009cdee0|S0275]] (5), [[S0276 009cdf80-009d3fe0|S0276]] (16), [[S0281 009ea8a0-009f0100|S0281]] (3), [[S0283 009f39b0-009f8370|S0283]] (17), [[S0285 009fef30-00a059e0|S0285]] (1), [[S0286 00a05a50-00a0c390|S0286]] (1), [[S0287 00a0c3a0-00a13140|S0287]] (12), [[S0290 00a1de60-00a243e0|S0290]] (16), [[S0292 00a29620-00a2fe20|S0292]] (2), [[S0294 00a35150-00a39a80|S0294]] (2), [[S0295 00a39b00-00a3f350|S0295]] (5), [[S0296 00a3f560-00a45650|S0296]] (2), [[S0297|S0297]] (3), [[S0298 00a4b480-00a53380|S0298]] (10), [[S0299 00a533a0-00a59140|S0299]] (44), [[S0300 00a59220-00a5da20|S0300]] (6), [[S0301 00a5da90-00a62140|S0301]] (2), [[S0303 00a674c0-00a6c3b0|S0303]] (5), [[S0305 00a707f0-00a75b20|S0305]] (4), [[S0306 00a75b70-00a7b4d0|S0306]] (8), [[S0309 00a87ba0-00a8cdc0|S0309]] (8), [[S0310 00a8ce60-00a91b00|S0310]] (3), [[S0312 00a97450-00a9d360|S0312]] (11), [[S0314 00aa2560-00aa90c0|S0314]] (24), [[S0317 00ab2c80-00ab7380|S0317]] (24), [[S0318 00ab73e0-00abcb30|S0318]] (2), [[S0322 00acd6d0-00ad3250|S0322]] (2), [[S0323 00ad3270-00ad83b0|S0323]] (14), [[S0324 00ad8880-00adee10|S0324]] (1), [[S0326 00ae4fa0-00ae9b00|S0326]] (2), [[S0327|S0327]] (1), [[S0333 00b052f0-00b0a290|S0333]] (5), [[S0334 00b0a370-00b10840|S0334]] (9), [[S0337 00b1b5a0-00b21f20|S0337]] (7), [[S0338 00b21fb0-00b29620|S0338]] (10), [[S0339 00b29890-00b2eaf0|S0339]] (24), [[S0340 00b2ec20-00b33a50|S0340]] (51), [[S0345 00b4b3e0-00b50190|S0345]] (1), [[S0347 00b56090-00b5b410|S0347]] (1), [[S0349 00b5ffe0-00b66a80|S0349]] (2), [[S0353 00b71490-00b77eb0|S0353]] (2), [[S0354|S0354]] (1), [[S0356 00b80ac0-00b84ce0|S0356]] (1), [[S0357 00b84e90-00b8a5f0|S0357]] (3), [[S0358 00b8a640-00b906a0|S0358]] (110), [[S0359 00b90750-00b94a30|S0359]] (1), [[S0364 00ba74d0-00bac070|S0364]] (6), [[S0365 00bac0d0-00bb1110|S0365]] (25), [[S0373 00bd5e90-00bd9850|S0373]] (2), [[S0375 00be0120-00be58f0|S0375]] (5), [[S0379|S0379]] (11), [[S0380 00bf5ff0-00bf9410|S0380]] (14), [[S0382 00bfcfe0-00c056a0|S0382]] (6), [[S0386 00c1f190-00c26aa0|S0386]] (2), [[S0388 00c2df40-00c34100|S0388]] (2), [[S0389 00c34110-00c39a40|S0389]] (1), [[S0391 00c3df10-00c423a0|S0391]] (6), [[S0394|S0394]] (17), [[S0395|S0395]] (2), [[S0398|S0398]] (5), [[S0400 00c6df10-00c73b60|S0400]] (10), [[S0401 00c73bb0-00c7a5b0|S0401]] (2), [[S0402 00c7a610-00c7f590|S0402]] (6), [[S0403 00c7f620-00c84e90|S0403]] (8), [[S0405 00c89e70-00c8f7c0|S0405]] (3), [[S0406 00c8f830-00c95560|S0406]] (18), [[S0407 00c955b0-00c9baf0|S0407]] (54), [[S0410 00ca6401-00cab4e0|S0410]] (4), [[S0411 00cab620-00cb0bb0|S0411]] (1), [[S0412 00cb0bf0-00cb6b80|S0412]] (4), [[S0413 00cb6d20-00cbc7f0|S0413]] (29), [[S0414 00cbc8e0-00cc18f0|S0414]] (1), [[S0415 00cc1a00-00cc6bc0|S0415]] (1), [[S0418 00cd1210-00cd5dd0|S0418]] (4), [[S0419 00cd5e90-00cdb1a0|S0419]] (4), [[S0420 00cdb370-00ce0140|S0420]] (3), [[S0421 00ce0340-00ce4c30|S0421]] (8), [[S0422 00ce4e30-00ceb920|S0422]] (33), [[S0423 00ceb9b0-00cf13e0|S0423]] (4), [[S0424 00cf1430-00cf68d0|S0424]] (32), [[S0425 00cf69c0-00cfc480|S0425]] (32), [[S0426 00cfc4e0-00d01c70|S0426]] (1), [[S0437 00d297c0-00d2f870|S0437]] (11), [[S0439|S0439]] (20), [[S0442|S0442]] (1), [[S0445 00d54b90-00d59cb0|S0445]] (16), [[S0446|S0446]] (5), [[S0447|S0447]] (4), [[S0449 00d694d0-00d6f0a0|S0449]] (21), [[S0466|S0466]] (2), [[S0494 00e701f0-00e760b0|S0494]] (9), [[S0497|S0497]] (59), [[S0509|S0509]] (1), [[S0511|S0511]] (50), [[S0512|S0512]] (5), [[S0513|S0513]] (20), [[S0517|S0517]] (6), [[S0529|S0529]] (2), [[S0538|S0538]] (1), [[S0541|S0541]] (10), [[S0542|S0542]] (13), [[S0544|S0544]] (11), [[S0549|S0549]] (36), [[S0553|S0553]] (9), [[S0556|S0556]] (1), [[S0558|S0558]] (42), [[S0563|S0563]] (10), [[S0565 01025bf0-0102b850|S0565]] (2), [[S0567|S0567]] (10), [[S0568|S0568]] (16), [[S0569|S0569]] (7), [[S0571 01044480-0104a2b0|S0571]] (9), [[S0573|S0573]] (16), [[S0574|S0574]] (18), [[S0575|S0575]] (17), [[S0577|S0577]] (17), [[S0579|S0579]] (5), [[S0580 0107d0c0-010829c0|S0580]] (4), [[S0581|S0581]] (23), [[S0582|S0582]] (2), [[S0584 01093ee0-01099b80|S0584]] (1), [[S0587|S0587]] (5), [[S0588|S0588]] (3), [[S0590|S0590]] (8), [[S0594|S0594]] (2), [[S0595|S0595]] (48), [[S0599|S0599]] (1), [[S0602|S0602]] (10), [[S0656|S0656]] (1), [[S0658 0124dfe0-01253ee0|S0658]] (3), [[S0664|S0664]] (5), [[S0668|S0668]] (6), [[S0674|S0674]] (2), [[S0675|S0675]] (2), [[S0676|S0676]] (5), [[S0678|S0678]] (4), [[S0679|S0679]] (4), [[S0680|S0680]] (2), [[S0681|S0681]] (1), [[S0682|S0682]] (62), [[S0683|S0683]] (1), [[S0685|S0685]] (4), [[S0688|S0688]] (23), [[S0689|S0689]] (5), [[S0690|S0690]] (11), [[S0691|S0691]] (11), [[S0693|S0693]] (4), [[S0694|S0694]] (15), [[S0696|S0696]] (1), [[S0697|S0697]] (6), [[S0700|S0700]] (13), [[S0702|S0702]] (2), [[S0703|S0703]] (15), [[S0704|S0704]] (92), [[S0705|S0705]] (2), [[S0706|S0706]] (39), [[S0708|S0708]] (32), [[S0710|S0710]] (17), [[S0713|S0713]] (3), [[S0715|S0715]] (34), [[S0718|S0718]] (8), [[S0719|S0719]] (3), [[S0721|S0721]] (17), [[S0722|S0722]] (2), [[S0724|S0724]] (7), [[S0727 013be8e0-013c3b90|S0727]] (17), [[S0729|S0729]] (12), [[S0730|S0730]] (30), [[S0731|S0731]] (17), [[S0735|S0735]] (35), [[S0737|S0737]] (37), [[S0738|S0738]] (3), [[S0739|S0739]] (2), [[S0740|S0740]] (32), [[S0743|S0743]] (44), [[S0745|S0745]] (8), [[S0748|S0748]] (1), [[S0749|S0749]] (22), [[S0750|S0750]] (14), [[S0751|S0751]] (10), [[S0752|S0752]] (6), [[S0762|S0762]] (1), [[S0770|S0770]] (3), [[S0771|S0771]] (2), [[S0772|S0772]] (3), [[S0774|S0774]] (2), [[S0775|S0775]] (2), [[S0785|S0785]] (6), [[S0790|S0790]] (2), [[S0791|S0791]] (5), [[S0793|S0793]] (9)
