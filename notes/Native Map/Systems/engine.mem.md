# System: engine.mem

128 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0071b7f0` | `FreeTextureArrayStorage` | SoeUtil::Array<DeepAssetHelper::Texture… | Storage deallocator for the texture-pointer array. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `009208c0` | `HashListRailDefinition_Slot1Thunk` | SoeUtil::HashList<RailDefinition,64,32> | Virtual slot [1] of the HashList<RailDefinition,64,32> base (shared with HashListMap<int,RailDefinition,64,32>); forwards to the chunk-pool allocate path. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `009208d0` | `HashListRailDefinition_Slot2Thunk` | SoeUtil::HashList<RailDefinition,64,32> | Virtual slot [2] of the HashList<RailDefinition,64,32> base (shared with HashListMap<int,RailDefinition,64,32>); forwards to the chunk-pool free path. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00a8caa0` | `ListGuid20DtorBody` | SoeUtil::List<SoeUtil::StrongType<unsig… | Destroys a character-GUID list and releases its pool chunk tree on unwind or destruction. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8cc70` | `ListGuid20ScalarDeletingDtor` | SoeUtil::List<SoeUtil::StrongType<unsig… | Destroys a character-GUID list object and frees its storage when its owner drops it. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a9aa40` | `SoeUtil_ListEncounterDetailsCommon_NodeFree` | SoeUtil::List<EncounterDetailsCommon,-1> | Slot [3] of the SoeUtil::List<EncounterDetailsCommon,-1> vtable: node deallocator for the matchmaking queue list. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00c8f5b0` | `ProfileSortingListAllocateNode` | SoeUtil::List<ProfileSortingData,-1> | Allocates one profile sorting list node. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `01047920` | `HashMap128_128::AdjustorThunk_020c_Alloc32` | SoeUtil::HashMap<int,ContentPack,128,12… | This-adjusting thunk for virtual slot [1] of HashMap<int,ContentPack,128,128>: forwards to the block pool allocator embedded at +0x20c. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047930` | `HashMap128_128::AdjustorThunk_020c_Free32` | SoeUtil::HashMap<int,ContentPack,128,12… | This-adjusting thunk for virtual slot [2] of HashMap<int,ContentPack,128,128>: forwards to the block pool free routine embedded at +0x20c. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (82)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00401bb0` | `AllocateMemory` |  | Shared client allocation entry used by 254+ callers across the binary. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00401c00` | `FreeMemory` |  | Shared client free entry matching 00401bb0. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `004438dd` | `AllocateStampedBlock_8_c` |  | Stamped engine allocation used by the GFC class factory so that objects carry the engine tag pair; the same emission appears elsewhere as AllocateStampedBlock_8. | [[Native Map/Shards/S0014 00443617-00447db4|S0014]] |
| `004507c0` | `AllocateStampedBlock_2C` |  | Allocation helper of the engine memory/class registry: the tag pair 0x56471e89/0x9fe1234a is written by the GFC instance allocators throughout the binary (e.g. 00440330, 004410cf, 00442154). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004509c0` | `AllocateTaggedFontObjectBlock` |  | Allocates and stamps a tagged block; callers 00453f42 and 0045471b use this allocator in font-object construction paths, while the allocator body itself is generic. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00451136` | `AllocateStampedBlock_8_b` |  | Allocation helper with the GFC heap tag pair, emitted for a further translation unit. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004593b9` | `AllocateStampedBlock` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00459daa` | `AllocateStampedBlock_2` |  | Stamped allocation helper for heap diagnostics; caller 00479860 passes its EAX result as ECX to the frame-buffer constructor 00477c01. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a5ed` | `AllocateStampedBlock_3` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a6b5` | `AllocateStampedBlock_4` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045a8b9` | `AllocateStampedBlock_5` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045b74b` | `AllocateStampedBlock_6` |  | Stamped allocation helper for heap diagnostics; caller 00479d94 passes its EAX result as ECX to the draw-device constructor 00474642. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045ba4e` | `AllocateStampedBlock_7` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045bd1e` | `AllocateStampedBlock_8` |  | Stamped allocation helper for heap diagnostics; caller 00476c26 passes its EAX result as ECX to a subobject constructor. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045bde5` | `AllocateStampedBlock_9` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045beed` | `AllocateStampedBlock_10` |  | Pooled heap allocation wrapper; the helper and sentinel pair are documented for the accepted AllocHeapCanaryBlockA (0047bc69, sentinels at +8/+0xC) and the sibling AllocateStampedBlock_2C (004507c0, +0x2C/+0x30). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0047bc69` | `AllocateStampedBlock` |  | Allocation wrapper that stamps two fixed header values; whether they are guards or ordinary metadata is unresolved. | [[Native Map/Shards/S0025 00477947-0047ca0c|S0025]] |
| `0047bf41` | `AllocateStampedBlock` |  | Allocation wrapper that stamps two fixed header values; whether they are guards or ordinary metadata is unresolved. | [[Native Map/Shards/S0025 00477947-0047ca0c|S0025]] |
| `0047c0c4` | `AllocateStampedBlock` |  | Allocation wrapper that stamps two fixed header values; whether they are guards or ordinary metadata is unresolved. | [[Native Map/Shards/S0025 00477947-0047ca0c|S0025]] |
| `0047c18c` | `AllocateStampedBlock` |  | Allocation wrapper that stamps two fixed header values; whether they are guards or ordinary metadata is unresolved. | [[Native Map/Shards/S0025 00477947-0047ca0c|S0025]] |
| `00721da0` | `OperatorNew0x30_A` |  | Class-specific operator new for a 0x30-byte node type of the definition containers. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721e00` | `OperatorNew0x18` |  | Class-specific operator new for a 0x18-byte node type. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721e60` | `OperatorNew0x30_B → SoeUtil::List<CCNamesDefinition,-1>::AllocateNode` |  | Class-specific operator new for a second 0x30-byte node type. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721ea0` | `OperatorDelete4_C` |  | Class-specific operator delete for a third node type. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722660` | `OperatorNew` |  | Global operator new of the client, used by definition parsers and other callers. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007226b0` | `OperatorDelete` |  | Global operator delete of the client. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `008d4830` | `PoolAllocBlocks56` |  | Slab refill for a 56-byte-element pool: the free list is threaded through the first dword of each block, and the slab bookkeeping is kept in an ordered tree; callers 00944fe0 and 00a1b510 allocate from it. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `0091f0d0` | `ChunkPoolGrow` |  | Backing allocator of the hash container destroyed by 0091e8a0; reached through the container's virtual allocate slot 0091f330. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091f1f0` | `ChunkPoolFreeBlock` |  | Block release path of the same pool, invoked when every chunk of a block is free again. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091f250` | `ChunkPoolAlloc` |  | Chunk allocation behind the virtual slot 0091f330 of the hash container. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091f2c0` | `ChunkPoolFree` |  | Chunk release behind the virtual slot 0091f340 of the same hash container. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091fb10` | `ChunkPoolGrow_2` |  | Backing allocator of the second pool instance in this shard, the one whose alloc/free slots are 0091fd40/0091fd50 for SoeUtil::List<...DeliverableAsset,256>. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091fc30` | `ChunkPoolFreeBlock_2` |  | Block release path of the second chunk pool. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091fc90` | `ChunkPoolAlloc_2` |  | Chunk allocation behind the virtual slot 0091fd40 of the DeliverableAsset list pool. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `0091fce0` | `ChunkPoolFree_2` |  | Chunk release behind the virtual slot 0091fd50 of the DeliverableAsset list pool. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `00920660` | `ChunkPoolGrow_3` |  | Backing growth routine of the third chunk pool, called by ChunkPoolAllocate_3 (009207e0) when the free list is empty. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00920780` | `ChunkPoolFreeBlock_3` |  | Block release path of the third chunk pool, called by ChunkPoolFree_3 (00920850) when a block's free count reaches its size. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `009207e0` | `ChunkPoolAllocate_3` |  | Allocate path of the third chunk pool, reached through the HashList slot-1 thunk 009208c0. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `00920850` | `ChunkPoolFree_3` |  | Free path of the third chunk pool, reached through the HashList slot-2 thunk 009208d0. | [[Native Map/Shards/S0243 00920130-00925ec0|S0243]] |
| `009e49d0` | `FreeMemoryAligned8` |  | Two-argument deallocation stub, most likely an operator delete variant with fixed alignment 8 for objects in this module. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00a12c20` | `OperatorNew_0x18_Align4` |  | Class operator new for a 0x18-byte object allocated through the engine heap. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12c60` | `OperatorDelete_0x18_Align4` |  | Class operator delete for the 0x18-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12c80` | `OperatorDelete_SizeGuard_Align8` |  | Operator delete used by a container type with a small inline buffer (deletes only heap spills). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12ce0` | `ClientGuildManager_PendingGuildNameRequest_HashMap_OperatorNew` |  | Allocator function associated with the explicitly named PendingGuildNameRequest hash-map vtable; requests 0x20 bytes aligned to 8. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12d20` | `OperatorDelete_0x20_Align8 → ClientGuildManager_PendingGuildNameRequest_HashMap…` |  | Class operator delete for a 0x20-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12dd0` | `OperatorNew_0x38_Align8` |  | Class operator new for a 0x38-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12e30` | `OperatorNew_0xe0_Align8` |  | Class operator new for a 0xe0-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12f10` | `OperatorNew_0x88_Align8` |  | Class operator new for a 0x88-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12f50` | `OperatorDelete_0x88_Align8` |  | Class operator delete for a 0x88-byte object. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a13060` | `OperatorNew_0x38_Align8_B` |  | Separate operator new for another 0x38-byte class (the compiler emits one per class). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a130a0` | `OperatorDelete_0x38_Align8_B` |  | Second operator delete for a 0x38-byte class. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a8c870` | `PoolChunkGrow` |  | Grows a pool of 16-byte entries by one chunk and records the chunk in its tree so it can be released later. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8c990` | `PoolChunkRelease` |  | Frees a pool chunk: removes its entries from the free list, erases its tree record, and returns the memory. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8c9f0` | `PoolEntryAlloc` | SoeUtil::List<SoeUtil::StrongType<unsig… | Allocates one 16-byte entry from the pool (the list's allocate slot), growing the pool by a chunk when empty. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8ca40` | `PoolEntryFree` | SoeUtil::List<SoeUtil::StrongType<unsig… | Returns one 16-byte entry to the pool (the list's free slot) and releases a chunk when all its entries are back. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a9a7b0` | `SoeUtil_ArrayIStringU16Ptr_FreeBuffer` | SoeUtil::Array<SoeUtil::IString<unsigne… | Slot [2] of the SoeUtil::Array<SoeUtil::IString<unsigned_short>*,0,1> vtable: releases the heap element buffer; this instantiation has no inline storage, so the pointer is always freed. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9a840` | `SoeUtil_ArrayIStringU16Ptr4_FreeBuffer` | SoeUtil::Array<SoeUtil::IString<unsigne… | Slot [2] of the SoeUtil::Array<SoeUtil::IString<unsigned_short>*,4,1> vtable: releases the element buffer only when it is not the four-element inline buffer. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9a930` | `SoeUtil_ArrayInteractionData_FreeBuffer` | SoeUtil::Array<InteractionData,0,1> | Slot [2] of the SoeUtil::Array<InteractionData,0,1> vtable: releases the heap element buffer of an InteractionData array with no inline storage. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9a9c0` | `SoeUtil_ArrayInteractionData16_FreeBuffer` | SoeUtil::Array<InteractionData,16,1> | Slot [2] of the SoeUtil::Array<InteractionData,16,1> vtable: releases the element buffer only when it is not the 16-element inline buffer. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9aa00` | `SoeUtil_ListEncounterDetailsCommon_NodeAlloc` | SoeUtil::List<EncounterDetailsCommon,-1> | Slot [2] of the SoeUtil::List<EncounterDetailsCommon,-1> vtable: node allocator for the matchmaking queue list, paired with the node free 00a9aa40 which releases through 00401c00 with the same alignment 8. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9bb70` | `SoeUtil_DequeIStringU16_256_AllocBlock` | SoeUtil::Deque<SoeUtil::IString<unsigne… | Slot [1] of the SoeUtil::Deque<SoeUtil::IString<unsigned_short>,256> vtable: block allocator used by the deque of 16-byte UTF-16 strings, paired with the block free 00a9bbc0. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a9bbc0` | `SoeUtil_DequeIStringU16_256_FreeBlock` | SoeUtil::Deque<SoeUtil::IString<unsigne… | Slot [2] of the SoeUtil::Deque<SoeUtil::IString<unsigned_short>,256> vtable: releases a deque storage block; the deque code calls it as (block, 0x1000) when whole blocks are freed. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00b3ef90` | `SumSpanTreeCountsPlusOne` |  | Provides the current tracked span capacity used when the 0x58-byte block pool chooses a new span size. | [[Native Map/Shards/S0342 00b39c10-00b3f310|S0342]] |
| `00b3eff0` | `FindGreatestTreeKeyAtOrBelow` |  | Finds the owning span for pool allocation and release operations. | [[Native Map/Shards/S0342 00b39c10-00b3f310|S0342]] |
| `00c8aaa0` | `ThreadMemoryManagerAllocate` |  | Allocates memory through the thread-specific or global memory manager. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8ada0` | `FreeTreePostorder` |  | Recursively frees both child subtrees and their root node. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8ae80` | `FreeTreeAndClearPointers` |  | Frees an entire rooted tree and clears its owner fields. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8af80` | `ClearTreeAndPointers` |  | Cleanup helper reached from exception unwind funclets. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8c9f0` | `PoolAllocateElement → SoeUtil::HashList<RecipeData,64,16>::RemoveFirst` |  | Allocates one element from the pool free list. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8ca60` | `PoolFreeElement → SoeUtil::HashList<RecipeData,64,16>::AddFirst` |  | Returns an element to the pool free list. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8f5f0` | `FreeMemoryAligned4` |  | Frees one profile sorting list node with alignment 4. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00d32c80` | `GrowFixedObjectPool` |  | Expands storage for the fixed-size object pool. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32db0` | `PoolPopSlot` |  | Allocates one slot from the fixed-size object pool. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `00d32e20` | `PoolReturnSlot` |  | Returns one object slot to its fixed-size pool. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |
| `01044ef0` | `ZeroInitRecord1A0` |  | Zero-initialiser for a 0x1a0-byte record; the owning type is not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044f40` | `ZeroInitRecord60` |  | Zero-initialiser for a 0x60-byte record; the owning type is not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01045020` | `ZeroInitRecord28` |  | Zero-initialiser for a 0x28-byte record; the owning type is not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01045040` | `ZeroInitRecord18` |  | Zero-initialiser for a 0x18-byte record; the owning type is not established here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047870` | `BlockPool_Alloc32` |  | Allocates one 0x20-byte entry from a block pool, refilling when the free list is empty and keeping per-block used counts when the flag is set. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010482a0` | `ZeroInitRecord60_Dup` |  | Same body as ZeroInitRecord60 (01044f40): zero-initialiser for a 0x60-byte record; owning type not established. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01048360` | `ZeroInitRecord2C` |  | Zero-initialiser for a 0x2c-byte record; owning type not established. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049870` | `ZeroInitRecord1A0_Dup` |  | Duplicate zero-initialiser for a 0x1a0-byte record; owning type not established. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟡 speculative (24)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00401d30` | `FreeMemoryPtr ?` |  | Convenience free used where no alignment hint is available (single-argument delete-style call). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `0071b6c0` | `FreeArrayStorage ?` |  | Calls FreeMemory with alignment four on the supplied pointer. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00721de0` | `OperatorDelete4_A ? ⚠rename` |  | Class-specific operator delete matching the 4-aligned node allocations. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00721e40` | `OperatorDelete4_B ? ⚠rename` |  | Class-specific operator delete for a second node type. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `0079cc50` | `PoolAllocEntry ?` |  | Fastcall (ecx = pool holder). When the free-list head at +0x1048 is NULL, obtains a 0x1E0790-byte block through the manager routing used by 00799120 (006fb860, DAT_01bdf280 slot [1], fallback 006fb950 with alignment 0x1… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d0d0` | `PoolReleaseSweepA ?` |  | Fastcall (ecx = pool holder with a flag byte at +0x10). When the flag is clear: finds the first node of the tree at +0, walks the tree in order (+8/+0xc links, +4 parent), clears the word at +0x18 of each node, then for… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d1f0` | `PoolReleaseSweepB ?` |  | Same structure as 0079d0d0 (PoolReleaseSweepA) with element size 0x24: owning-block lookup through 00785630 and release through 00785ca0 in place of 00781330 and 007819a0. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d860` | `PoolFlagSetSweepA ?` |  | Thiscall (ecx = pool holder, one byte argument). If the flag byte at +0x10 was clear and the new value is nonzero, runs 0079d0d0 (PoolReleaseSweepA). Then stores the new flag byte at +0x10. Returns nothing. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0079d890` | `PoolFlagSetSweepB ?` |  | Thiscall (ecx = pool holder, one byte argument). If the flag byte at +0x10 was clear and the new value is nonzero, runs 0079d1f0 (PoolReleaseSweepB). Then stores the new flag byte at +0x10. Returns nothing. | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `00822d30` | `AllocStampedBlock ?` |  | Pushes its argument and calls 004d9207 (pooled allocator, cdecl with one argument); when the returned block is non-NULL stores 0x56471e89 at block+8 and 0x9fe1234a at block+0xc. The block pointer remains in EAX at retur… | [[Native Map/Shards/S0197 00821940-00827640|S0197]] |
| `00a72a30` | `MemFreeAlign4_Ret4 ? → HashMap<int,int,15,-1>::FreeNode` |  | Reads its first stack argument, pushes 4 and that pointer, calls FreeMemory 00401c00, pops 8, returns with ret 4. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a72a90` | `MemFreeAlign4_Ret4 ? → HashMap<int,IString<unsigned short>,15,-1>::FreeNode` |  | Byte-identical to 00a72a30: reads its first stack argument, pushes 4 and the pointer, calls FreeMemory 00401c00, pops 8, returns with ret 4. | [[Native Map/Shards/S0305 00a707f0-00a75b20|S0305]] |
| `00a8cb60` | `ListGuid20VSlot3Free ?` | SoeUtil::List<SoeUtil::StrongType<unsig… | Virtual slot 3 (+0xc) of List<StrongType<uint64,SoeUtilSignatureCharacterGuid,0>,20>: bytes are add ecx,0x10 followed by a rel32 jump to 00a8ca40 (PoolEntryFree); it is a this-adjusting jump thunk. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00ad3c90` | `FreeMemory ?` |  | Calls 00401c00 with the supplied pointer and alignment four. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00c8aae0` | `FreeMemoryAligned4 ?` |  | Frees a pointer with alignment 4. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8ab00` | `InitFixedNodePool ?` |  | Zeros four fields, initializes four intrusive free-list sentinels spaced by 0x50 bytes, and stores capacity 0x10. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c8c870` | `AllocateTreeBlock ? → PoolAllocateBackingBlock` |  | Computes bounded block capacity from aggregate state, allocates a tree node, inserts it, links its elements, and updates the count. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00d0b6a0` | `FreeOwnedBlock_00f8e9b0 ?` |  | If stored pointer is non-null, calls 00f8e9b0 with it and then frees the pointer. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0b6c0` | `FreeOwnedBlock_00f8e9b0 ?` |  | If stored pointer is non-null, calls 00f8e9b0, frees the allocation, and clears the stored pointer field. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0b770` | `FreeOwnedBlock_00fc6e40 ?` |  | If stored pointer is non-null, calls 00fc6e40 on it and then frees allocation. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0b790` | `FreeAndClearOwnedBlock_00fc6e40 ?` |  | Calls 00fc6e40 and frees stored allocation when non-null, then clears stored pointer field. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0ba60` | `ReplaceOwnedBlock_00f8e9b0 ?` |  | Calls 00f8e9b0 and frees previous pointer when non-null, then stores new pointer. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d0baa0` | `ReplaceOwnedBlock_00fc6e40 ?` |  | Calls 00fc6e40 and frees previous pointer when non-null, then stores new pointer. | [[Native Map/Shards/S0428 00d07500-00d0bf70|S0428]] |
| `00d32d50` | `ReleaseEmptyPoolChunk ?` |  | Removes free-list entries within a fully returned chunk, subtracts its slot count, erases its tree range, and frees the chunk block. | [[Native Map/Shards/S0438 00d2f8c0-00d34430|S0438]] |

## ⚪ unknown (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00440330` | `— ⚠reject` |  | Stamped allocation wrapper; the meaning of the two marker dwords and the allocation purpose are caller-specific. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440aa5` | `— ⚠reject` |  | Stamped allocation wrapper; the meaning of the marker pair and the allocation purpose are caller-specific. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00440ea1` | `— ⚠reject` |  | Stamped allocation wrapper; caller 00445D5B uses the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004410cf` | `— ⚠reject` |  | Stamped allocation wrapper; caller 0045018A uses the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00442154` | `— ⚠reject` |  | Stamped allocation wrapper; callers 00450237 and 004502C8 use the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00442290` | `— ⚠reject` |  | Stamped allocation wrapper; caller 0044F06C uses the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `004425bb` | `— ⚠reject` |  | Stamped allocation wrapper; callers 0044F7BC, 0044FC80, 0044FD43 and 004503B4 use the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00443427` | `— ⚠reject` |  | Stamped allocation wrapper; caller 0044F9B8 uses the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `00443572` | `— ⚠reject` |  | Stamped allocation wrapper; caller 0044A33E uses the returned block. | [[Native Map/Shards/S0013 0043de42-004435e4|S0013]] |
| `008c6d90` | `— ⚠reject` |  | Small member cleanup routine called from EH unwind funclets; owning class and meanings of fields +4 and +8 are unknown. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00c8c990` | `— ⚠reject` |  | Returns an element to its block pool and frees an empty block. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `010476f0` | `— ⚠reject` |  | Refills a block pool with a new block of 0x20-byte entries and registers the block in the pool's tree. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010478c0` | `— ⚠reject` |  | Returns one 0x20-byte entry to a block pool and releases the owning block once all of its entries are back, when the flag is set. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

