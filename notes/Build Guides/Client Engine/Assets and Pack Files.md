---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Assets and Pack Files

Part of [[Build Guides]] › Client Engine. Related: [[Rendering]], [[Zones and Terrain]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

1828 accepted functions: 602 confirmed, 653 likely, 312 speculative, 261 unknown. Network-linked (`wire`): 8.

Subsystems: `engine.gds` (513), `asset.definition` (293), `asset.loader` (255), `asset.sensory_effect` (245), `asset.delivery` (227), `engine.asset` (114), `engine.dbm` (101), `lib.soegems` (52), `asset.mannequin` (14), `lib.soegems.refarray` (8), `asset` (6)

_Large or library guide: only confirmed and likely functions are listed (1255 of 1828)._

### `asset`

**SoeUtil::HashListMap** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa530` | HashListMap_AssetReadyHandlerEntry_Construct | confirmed | Constructor of the asset-ready handler map: installs the SoeUtil::HashListMap<int,AssetManagement::AssetRequestInternal::AssetReadyHandler::HashEntry,1024,-1> … |
| `008aa5d0` | HashListMap_AssetPtr_8192_Construct | confirmed | Constructor of the asset-pointer map with 8192 buckets: installs the SoeUtil::HashListMap<int,AssetManagement::Asset*,8192,-1> vtable, zeroes +4/+8/+0xc/+0x10 … |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae2ba0` | InitializeFixedAssetPools | likely | Allocates and links 100 nodes containing 50-byte strings, then allocates and links 400 nodes containing 128-byte fixed strings; updates each pool's head/tail/c… |
| `00c8abb0` | CopyIconDataFields | likely | Copies two leading dwords, constructs embedded IconData from source +8, then copies four scalar dwords from source +0x14 through +0x20. |
| `00c8ae30` | CopyEmbeddedIconAndFields | likely | Initializes embedded icon data from the supplied source and copies the node's remaining scalar fields. |
| `00cd9480` | CreateReferencedAssetObject | likely | Increments a reference count, allocates a 0x58-byte object, constructs its asset-backed state, and stores the resulting handle. |

### `asset.definition`

**Actor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007849d0` | Actor::ReportFailedPendingTextureAliases | likely | Walks the pending texture-alias list at this+0x4cc (head at +0x4d0, next at node+0xac). For each node whose flag byte at +0xa8 is zero and where 007b8ec0(node+… |

**ActorCompositeEffectDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00914a10` | DtorBody | confirmed | __fastcall (this=ECX). Under an SEH frame: stores ActorCompositeEffectDefinition::vftable (01804f24) at [this]; stores SoeUtil::List<ActorCompositeEffectEntry,… |
| `00914ad0` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01804f24 (thiscall, one byte argument). Calls FUN_00914a10 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); r… |

**ActorDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078da30` | ActorDefinition::Ctor | confirmed | Stores ActorDefinition::vftable, runs the StringFixed<128> base constructor 00402950, builds a 5-element array of 0x10-byte elements with _eh_vector_constructo… |
| `007b9dc0` | ActorDefinition_DtorBody | confirmed | Destructor body (ECX = this, SEH framed): stores ActorDefinition::vftable at +0, then tears down members in reverse construction order - 007b95b0 (alias tree a… |
| `0078dd00` | ActorDefinition::ScalarDeletingDtor | likely | Scalar deleting destructor (ret 4): calls 007b9dc0 (ActorDefinition teardown body) with this in ECX, then _free(this) only when flag bit 0 is set. Returns this… |
| `007b80f0` | StringTripleList_DestroyAllNodes | likely | Fastcall whose ECX is the list header (head at +0, tail at +4, count at +8): walks the nodes through the next pointer at node+4, unlinks each node by patching … |
| `007b8ec0` | AliasTree_ApplyByName | likely | Hashes param_2 with 00705b00 and walks the tree at this+0x298 (level 1 via +0x34, level 2 via +0x6c) looking for a level-3 record whose key at +0x2c equals tha… |
| `007b9170` | AliasTree_FindByNameAssignString | likely | Walks the alias tree at this+0x298 in three nested chains (level1 via +0x34, level2 head at level1+4 via +0x6c, level3 head at level2+4 via +0x34); for every i… |
| `007b95b0` | AliasTree_DestroyAll | likely | Destroys the whole alias tree at this+0x298: for each level-1 node (chain via +0x34) it walks the level-2 chain (head at level1+4, next +0x6c) and inside it tw… |

**ActorDefinitionAsset** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078dd20` | ActorDefinitionAsset::Ctor | confirmed | Calls 0071bb80 (AssetConstructor) on this, stores ActorDefinitionAsset::vftable, then calls 0078da30 (ActorDefinition::Ctor) on this. Ret with no value. |
| `0078dd80` | ActorDefinitionAsset::ScalarDeletingDtor | likely | Scalar deleting destructor (ret 4): calls 007c6850 (ActorDefinitionAsset teardown body) with this in ECX, then _free(this) only when flag bit 0 is set. Returns… |
| `007c6850` | ActorDefinitionAsset::DtorBody | likely | Fastcall, ECX = this (103 bytes, SEH frame). Stores the ActorDefinitionAsset vtable at +0, clears the byte at +0x1c, calls 007b9dc0, then calls 0071bc20 (Asset… |

**ActorEffectDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00914670` | Ctor | confirmed | __fastcall (this=ECX, returns this). Under an SEH frame calls the base constructor FUN_009145b0 (SensoryEffectDefinition constructor), then stores ActorEffectD… |

**ActorGroupDefinitionAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0079b510` | ActorGroupDefinitionAssetCtor | confirmed | Fastcall (ecx = this), returns this. Runs the base AssetConstructor 0071bb80, stores ActorGroupDefinitionAsset::vftable at +0, zeroes +0x28, calls 0079aa00 (Ac… |
| `0079ba50` | ActorGroupDefinitionAsset::ParseActorSet | confirmed | Virtual slot [3] of ActorGroupDefinitionAsset, thiscall with two stack arguments (reader object and element) and a stack-protected 0x8000+ frame. Reads named c… |
| `0079c180` | ActorGroupDefinitionAssetDtorBody | confirmed | Fastcall (ecx = this). Stores ActorGroupDefinitionAsset::vftable at +0, calls 0079a730 (the CustomTintEntry capacity-1 list destructor body) and 0079b420 (the … |
| `0079c1f0` | ActorGroupDefinitionAssetScalarDtor | confirmed | Thiscall (ecx = this, one flag argument). Calls 0079c180 on this, frees this through _free when bit 0 of the flag is set, and returns this. |

**ActorMeshEmitterDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00914e30` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01804fa0 (thiscall, one byte argument). Calls FUN_010b5460 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); r… |
| `008c05b0` | ActorMeshEmitterDefinition::ScalarDeletingDestructor | likely | Calls 0x010b5460 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00914e50` | ThunkThisMinus90ToDeletingDtor | likely | Adjustor thunk (bytes 81 e9 90 00 00 00 e9 d5 ff ff ff): subtracts 0x90 from ECX and jumps to FUN_00914e30. It is vftable[0] of the secondary vtable 01804f8c. |

**ActorMeshEmitterDefinitionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00914b60` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01804f6c (thiscall, one byte argument). Calls FUN_010b6110 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); r… |
| `00914af0` | CtorWrap0090ef40 | likely | thiscall (returns this in EAX). Under an SEH frame calls FUN_0090ef40 with the two stack arguments passed through, after storing the frame; the class vtables A… |
| `00914bc0` | ThunkThisMinus4ToDeletingDtorB | likely | Adjustor thunk (bytes 83 e9 04 e9 98 ff ff ff): subtracts 4 from ECX and jumps to FUN_00914b60. It is vftable[0] of the second ActorMeshEmitterDefinitionManage… |

**ActorModelMaterialDefinition** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008fda20` | ActorModelMaterialDefinition::Dtor | confirmed | fastcall (ECX=this, EH frame with /GS cookie). Stores ActorModelMaterialDefinition::vftable at this+0 and at this+0x28. Stores SoeUtil::List<ModelMaterialParam… |
| `008fdb60` | ActorModelMaterialDefinition::ScalarDeletingDtor | confirmed | thiscall with one byte argument (delete flags, ret 4). Saves this in ESI, calls 008fda20 (the destructor body), then if bit 0 of the flag argument is set calls… |
| `0093cee0` | ActorModelMaterialDefinition::Ctor | confirmed | Constructs the asset-request base, installs definition vtables, initializes its IString and parameter list, and clears definition fields. |
| `008fdb50` | ActorModelMaterialDefinition::SecondaryDtorAdjustor | likely | Eight bytes: sub ecx, 0x28; jmp 008fdb60. An adjustor thunk: it moves this back by 0x28 (from the secondary vtable at this+0x28 to the primary object) and jump… |
| `0093cfb0` | ActorModelMaterialDefinition::InitFromArgs | likely | Calls the definition constructor using the third argument's first dword, stores the first argument's dword at offset 0x68, and returns the destination. |
| `010b49b0` | ActorModelMaterialDefinition::Slot8_WriteKeyValues | likely | Virtual slot 8 of ActorModelMaterialDefinition: passes the key "material" with the value at this+0x34 to 00d518f0, passes "transitionTime" with the float at th… |
| `010b5190` | ActorModelMaterialDefinition::Slot7_ReadKeyValues | likely | Virtual slot [7] of ActorModelMaterialDefinition, the reader mirror of slot 8 (010b49b0): reads the "material" value through 00d517f0 and 00402ce0; builds a st… |

**ActorParticleEmitterDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00914710` | Ctor | confirmed | thiscall (this=arg1, arg2 on stack; returns this). Under an SEH frame: calls FUN_00914670() (ActorEffectDefinition constructor); stores AssetRequestInterface::… |
| `008c04f0` | ActorParticleEmitterDefinition::ScalarDeletingDestructor | likely | Calls 0x007e90b0 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |

**ActorParticleEmitterDefinitionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009149f0` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01804f04 (thiscall, one byte argument). Calls thiscall FUN_009149a0 (the destructor body) with ECX=this; then if (arg & 1) != 0 calls … |
| `00914370` | CtorWrap0090ea70 | likely | thiscall (returns this in EAX). Calls FUN_0090ea70(arg2, arg3) with the two stack arguments passed through, then returns param_1. The class vtables ActorPartic… |
| `00914440` | ThunkThisMinus4ToDeletingDtor | likely | Adjustor thunk (bytes 83 e9 04 e9 a8 05 00 00): subtracts 4 from ECX (this) and jumps to FUN_009149f0. It is vftable[0] of the second ActorParticleEmitterDefin… |

**ActorProjectileDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00915230` | Ctor | confirmed | thiscall (this=arg1, arg2 on stack; returns this). Calls the base constructor FUN_00914670 first; stores AssetRequestInterface::AssetRequester vftable (017e7d1… |
| `009152e0` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 0180501c (thiscall, one byte argument). Calls FUN_010b6220 (the destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); r… |
| `008c0610` | ActorProjectileDefinition::ScalarDeletingDestructor | likely | Calls 0x010b6220 with this in ECX and no stack arguments, then conditionally frees this when flag bit 0 is set. |
| `00915300` | ThunkThisMinus90ToDeletingDtor | likely | Adjustor thunk (bytes 81 e9 90 00 00 00 e9 d5 ff ff ff): subtracts 0x90 from ECX and jumps to FUN_009152e0. It is vftable[0] of the secondary vtable 01805008. |

**ActorProjectileDefinitionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00915050` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01804fe8 (thiscall, one byte argument; bytes 56 8b f1 e8 .. ). Calls FUN_010b6340 (the destructor body) with ECX=this; if (arg & 1) !=… |
| `00914fe0` | CtorWrap0090f420 | likely | thiscall (returns this). Calls FUN_0090f420(arg2, arg3) with the two stack arguments passed through, after its frame setup; the class vtables ActorProjectileDe… |
| `009150b0` | ThunkThisMinus4ToDeletingDtorC | likely | Adjustor thunk (bytes 83 e9 04 e9 98 ff ff ff): subtracts 4 from ECX and jumps to FUN_00915050. It is vftable[0] of the second ActorProjectileDefinitionManager… |

**ClientAccountInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071bfd0` | ClientAccountInterface::CreateParticleDefinitionAsset | confirmed | Allocates 0x68 bytes, gets asset context from manager, and constructs ParticleDefinitionAsset with requester pointer and enabled flag. |

**ClientAreaCompositeEffectProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cebe50` | ClientAreaCompositeEffectProperty::WriteFields | confirmed | Writes the base property values, effect definition ID, location offset, rotation, scale, floor tolerance, and broadcast flag to definition fields. |
| `00cebf90` | ClientAreaCompositeEffectProperty::ReadFields | confirmed | Reads effect definition ID, parses three location offsets and three rotations with atof, reads scale and floor tolerance, and parses Broadcast as 1/t/T. |
| `00cec1a0` | ClientAreaCompositeEffectProperty::AcquireEffectReferences | confirmed | Looks up the effect definition, increments an area-local counter, walks typed references, resolves each referenced object, and increments each resolved object'… |
| `00cecaf0` | ClientAreaCompositeEffectProperty::Release | confirmed | Releases typed effect references, visits remaining registered objects and invokes their virtual cleanup when gates allow, then clears the reference handle and … |

**ClientAreaMinimapProperty** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ced8d0` | ClientAreaMinimapProperty::WriteFields | confirmed | Writes base property fields and the enterMinimapName and exitMinimapName values. |
| `00ced990` | ClientAreaMinimapProperty::ReadFields | confirmed | Reads enterMinimapName and exitMinimapName strings and assigns each to its matching property field. |

**ClientAreaProperty** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cee0d0` | ClientAreaProperty::BaseCtor | confirmed | Stores the ClientAreaProperty vtable pointer at the beginning of the supplied object. |

**ClientAreaRandomEffectProperty** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ced210` | ClientAreaRandomEffectProperty::UpdateTransform | confirmed | When the area is active, copies four transform words from the owner into fields 0x70-0x7c and calls the effect-node motion/transform update routine. |
| `00ced330` | ClientAreaRandomEffectProperty::WriteFields | confirmed | Writes the base property values, effect ID, distance and frequency bounds, camera/ground/random-Y/area-relative flags, nondefault rotations, and nondefault sca… |
| `00ced510` | ClientAreaRandomEffectProperty::ReadFields | confirmed | Reads effect ID, minimum/maximum distance and frequency, placement flags and rotations; reads scale and passes it to a virtual setter, then invokes another pro… |
| `00ced260` | StoreDwordAt54 | likely | Stores the supplied 32-bit value at object offset 0x54. |
| `00ced270` | StoreDwordAt58 | likely | Stores the supplied 32-bit value at object offset 0x58. |
| `00ced280` | StoreDwordAt5C | likely | Stores the supplied 32-bit value at object offset 0x5c. |
| `00ced290` | StoreDwordAt60 | likely | Stores the supplied 32-bit value at object offset 0x60. |
| `00ced2a0` | StoreByteAt64 | likely | Stores the supplied byte at object offset 0x64. |
| `00ced2b0` | StoreByteAt65 | likely | Stores the supplied byte at object offset 0x65. |
| `00ced2c0` | StoreByteAt66 | likely | Stores the supplied byte at object offset 0x66. |
| `00ced2d0` | StoreInvertedByteAt67 | likely | Stores the logical inverse of the supplied byte as a boolean at object offset 0x67. |

**ClientAreaSkyTransitionProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceda00` | ClientAreaSkyTransitionProperty::WriteFields | confirmed | Writes the base client-area property fields through the property serializer. |
| `00ceda60` | ClientAreaSkyTransitionProperty::Ctor | confirmed | Initializes base property fields, stores constructor arguments, sets default values and installs the ClientAreaSkyTransitionProperty vtable. |
| `00cee0c0` | ClientAreaSkyTransitionProperty::GetTypeName | confirmed | Returns a pointer to the constant string SkyTransition. |
| `00cee120` | ClientAreaSkyTransitionProperty::ScalarDeletingDtor | confirmed | Resets the base ClientAreaProperty vtable and frees the object when the deleting flag bit is set; otherwise returns the object pointer. |

**ClientItemDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090b720` | ClientItemDefinition::ClientItemDefinition | confirmed | Constructs BaseItemDefinition, initializes its ability list, two shared strings, and several scalar and pointer fields, then calls 00905c20. |
| `0090b820` | ClientItemDefinition::~ClientItemDefinition | confirmed | Destroys embedded strings, removes all ability-list nodes, calls the BaseItemDefinition destructor, and restores ClientItemDefinition's vtable during cleanup. |
| `0090b910` | ClientItemDefinition::ScalarDeletingDtor | confirmed | Calls ClientItemDefinition destruction, frees the object when flag bit 0 is set, and returns the original object pointer. |

**Deep::ModelDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a8ed0` | DeepModelDefinitionDtorBody | confirmed | Destructor body (ECX = model definition): stores the Deep::ModelDefinition vftable at +0, releases the refcounted object at +0x74 (LOCK decrement, vtable +4 at… |
| `010a91e0` | ModelDefinitionScalarDeletingDtor | confirmed | Virtual slot 0 of the Deep::ModelDefinition vftable (thiscall, one stack byte flag, ret 4): calls the destructor body 010a8ed0 on this, calls _free on this whe… |
| `010a9c00` | ModelDefinition::ModelDefinition | confirmed | Constructor for Deep::ModelDefinition from a DMOD block: stores the vtable, calls 0071b950 for the RefCounted base, initialises the member arrays, checks the 4… |

**DeepAssetHelper::AnimationAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d35850` | AnimationAsset::Ctor | confirmed | Thiscall constructor with one stack argument: calls AssetConstructor 0071bb80 on the new object, installs the AnimationAsset vtable, stores the argument at +0x… |
| `00d358b0` | AnimationAsset::DtorBody | confirmed | Destructor body (no free): stores the AnimationAsset vtable, releases the ref-counted object at +0x2c (calls its slot 1 when the count reaches zero, then clear… |
| `00d35a80` | AnimationAsset::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the AnimationAsset destructor body 00d358b0 and frees the block through _free when bit 0 of the argument is… |
| `00d35930` | AnimationAsset::LoadSource | likely | Virtual slot [3] of AnimationAsset with args (source, listener). Builds an IString from the source through slot 2 (+8) of the argument object, looks it up with… |

**DeepAssetHelper::MaterialPaletteAsset** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00786270` | MaterialPaletteAssetCtor | confirmed | Thiscall (ECX = object; stack arguments param_2, param_3 and a byte param_4). Calls the base constructor 0071bb80 (AssetConstructor) first, then stores AssetRe… |
| `00786320` | MaterialPaletteAssetDeletingDtor | confirmed | Virtual slot 0 of DeepAssetHelper::MaterialPaletteAsset::vftable (secondary table 017ec738). Calls 00d35400 with ECX = this (base teardown), then calls _free(t… |
| `00786340` | MaterialPaletteAssetDtorThunk | confirmed | Adjustor thunk for the +0x28 secondary vtable: subtracts 0x28 from ECX (sub ecx, 0x28) and jumps to 00786320 (the MaterialPaletteAsset scalar deleting destruct… |
| `00d35400` | MaterialPaletteAsset::DtorBody | confirmed | Destructor body (no free): stores the primary and secondary MaterialPaletteAsset vtables, walks the list at this+0x40 releasing each ref-counted entry (calling… |
| `00d348b0` | MaterialPaletteAsset::CreateTextureAsset | likely | Secondary-vtable slot [3] of MaterialPaletteAsset with one stack arg. Allocates 0x34 bytes via 00d833a6; if non-null it calls TextureAsset ctor 00d355b0 on tha… |

**DeepAssetHelper::ModelDefinitionAsset** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d35b30` | ModelDefinitionAsset::Ctor | confirmed | Thiscall constructor with stack args (param_2 at +0x2c, param_3 at +0x30, param_4 at +0x34, param_5 at +0x3c) and byte param_6 at +0x50: calls AssetConstructor… |
| `00d35bf0` | ModelDefinitionAsset::AdjustedDeletingDtor | confirmed | Adjustor thunk for the secondary ModelDefinitionAsset vtable: subtracts 0x28 from ECX and jumps to the scalar deleting destructor 00d35d30. |
| `00d35c00` | ModelDefinitionAsset::DtorBody | confirmed | Destructor body (no free): stores both ModelDefinitionAsset vtables, releases the refcounted members at +0x3c, +0x34 (cleared), +0x38, +0x40 and +0x44 (each re… |
| `00d35d30` | ModelDefinitionAsset::ScalarDeletingDtor | confirmed | Scalar deleting destructor (primary vtable slot [0]): calls the destructor body 00d35c00 and frees the block through _free when bit 0 of the argument is set; r… |
| `00d36130` | ModelDefinitionAsset::CreateMaterialPalette | confirmed | Secondary-vtable slot [3] of ModelDefinitionAsset (ECX = secondary at primary+0x28). Allocates 0x80 bytes via 00d833a6; if non-null it constructs a MaterialPal… |

**DeepAssetHelper::ParticleDefinitionAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071bed0` | ParticleDefinitionAsset::Constructor | confirmed | Constructs Asset base and AssetRequester member, installs particle-asset vtables, initializes texture and string arrays and flags, and stores inputs. |
| `0071bfa0` | ParticleDefinitionAsset::ScalarDeletingDtor | confirmed | Calls 00d33ff0 particle-asset teardown and frees object when deleting flag bit is set. |
| `0071bfc0` | ParticleDefinitionAssetAdjustedDeletingDtor | confirmed | Subtracts 0x28 from ECX and jumps to 0071bfa0 scalar deleting destructor. |
| `00d344a0` | ParticleDefinitionAsset::ParseSource | confirmed | Virtual slot [3] of ParticleDefinitionAsset. Stores listener param_3 at this+0x58, copies the source IString (param_2) into a NUL-terminated buffer, parses it … |

**DeepAssetHelper::SkeletonDefinitionAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00775cf0` | SkeletonDefinitionAssetCtor | confirmed | Fastcall constructor (ECX = object). Calls 0071bb80 on the object (the Asset base constructor, thiscall with ECX unchanged), stores DeepAssetHelper::SkeletonDe… |
| `00775d50` | SkeletonDefinitionAsset::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag, ret 4). Calls the destructor body 00d34830 on the object, frees the object through _free when fla… |
| `00d34830` | SkeletonDefinitionAsset::DtorBody | confirmed | Stores the SkeletonDefinitionAsset vtable at this+0, releases the ref-counted object at this+0x28 (calls its slot 1 when the count reaches zero), then calls ba… |
| `00d347a0` | SkeletonDefinitionAsset::ParseSource | likely | Virtual slot [3] of SkeletonDefinitionAsset with args (source IString, listener). Allocates 0xaa8 bytes via operator new 00d833a6; if non-null it calls ctor 01… |

**DeepAssetHelper::TextureAsset** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d35520` | TextureAsset::DtorBody | confirmed | Destructor body (no free): stores the TextureAsset vtable at this+0, releases the ref-counted owner at this+0x2c (calls its slot 1 when the count reaches zero)… |
| `00d35590` | TextureAsset::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the TextureAsset destructor body 00d35520 and frees the block through _free when bit 0 of the argument is s… |
| `00d355b0` | TextureAsset::Ctor | confirmed | Thiscall constructor with args (name, owner param_3, flag param_4, byte param_5): calls base init 007cc650 on the name, installs the TextureAsset vtable, store… |
| `00d35640` | TextureAsset::CtorWithOwner | confirmed | Thiscall constructor with args (name, owner param_3, value param_4): calls base init 007cc650 on the name, installs the TextureAsset vtable, stores param_4 at … |
| `00d356c0` | TextureAsset::LoadSource | confirmed | Virtual slot [3] of TextureAsset with args (name IString, listener). Hashes the name with 00705b00 and looks it up through vtable +0xc4 of the manager at this+… |

**ItemCategoryManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01045520` | ItemCategoryManager::Ctor | confirmed | Thiscall, one stack arg (param_2, stored at this+0x260). Stores BaseDefinitionManager::vftable, then ItemCategoryManager::vftable at *this. Zeroes this+8, +0xc… |
| `010455f0` | ItemCategoryManager::ReleaseTables | confirmed | Thiscall with no args. Bytes: push esi; mov esi,ecx; lea ecx,[esi+4]; call 008dbc70; lea ecx,[esi+0x204]; call 008abd80; lea ecx,[esi+0x1a4]; pop esi; jmp 008d… |
| `01045620` | ItemCategoryManager::DtorBody | confirmed | Fastcall, ECX = this. Stores ItemCategoryManager::vftable at *this, calls 008dbc70 (table at this+4), 008abd80 and 008dbda0 (table at this+0x1a4), stores SoeUt… |
| `01045ba0` | ItemCategoryManager::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Bytes: push esi; mov esi,ecx; call 01045620 (dtor body); test byte [esp+8],1; je; push 0; push esi; call 00401c00 (free with… |

**ItemClassDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010483f0` | ItemClassDefinitionManager::Ctor | confirmed | Fastcall, ECX = this. Stores BaseDefinitionManager::vftable, then ItemClassDefinitionManager::vftable (0187f5e4) at *this, zeroes this+8, +0xc, +0x10 and +4, m… |
| `01048500` | ItemClassDefinitionManager::ReleaseTable | confirmed | Bytes 83 c1 04 e9 48 36 89 ff = add ecx,4; jmp 008dbb50. Thunk: advances this by 4 and tail-jumps to 008dbb50 (the 20-bucket table destroy on the table at this… |
| `01048510` | ItemClassDefinitionManager::DtorBody | confirmed | Fastcall, ECX = this. Stores ItemClassDefinitionManager::vftable at *this, calls 008dbb50 (destroys the 20-bucket table), then stores BaseDefinitionManager::vf… |
| `01048790` | ItemClassDefinitionManager::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Bytes: push esi; mov esi,ecx; call 01048510 (dtor body); test byte [esp+8],1; je; push 0; push esi; call 00401c00 (free with… |

**LoadingScreenData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca99e0` | LoadingScreenData::VirtualLoadOrRelease | likely | Virtual slot 3 of LoadingScreenData: when the mode argument is 0 it calls 00ca97e0 on the argument; when it is 1 it decrements the counter at this+0x24 and, wh… |

**LoadingScreenText** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ca91e0` | LoadingScreenText::Ctor | confirmed | Constructs a LoadingScreenText entry: calls the RefCounted base initialiser 0073b300, installs LoadingScreenText::vftable at +0, installs the IString vtable an… |
| `00ca9630` | LoadingScreenText::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of LoadingScreenText): calls the body 00ca9250 on this, then frees this through _free when bit 0 of the flag argument… |
| `00ca9250` | LoadingScreenText::DtorBody | likely | Destructor body of LoadingScreenText: installs the IString vtable, releases the embedded string buffer when its count is positive (decrement, IString slot 2 at… |

**LocaleDefinitionManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01050640` | LocaleDefinitionManager::DrainMap | confirmed | While the head at this+0xc is non-null, calls 01050560 (RemoveEntry) with ECX=this+4 and the head as argument; returns void. |
| `01050660` | LocaleDefinitionManager::LoadLanguages | confirmed | Drains the existing locale entries through 01050560, opens languages.txt through the text reader (0x5e separator), and for each non-comment row constructs a si… |
| `01050930` | LocaleDefinitionManager::DtorBody | confirmed | Restores LocaleDefinitionManager::vftable, runs the locale map teardown 010508c0 after draining through 01050560, then stores BaseDefinitionManager::vftable; n… |
| `01050a50` | LocaleDefinitionManager::Ctor | confirmed | Installs BaseDefinitionManager::vftable then LocaleDefinitionManager::vftable, runs the locale container constructor 010509b0 (receiver not recovered by the ps… |
| `01050ab0` | LocaleDefinitionManager::ScalarDeletingDtor | confirmed | Runs the manager teardown 01050930, then calls FreeMemory (00401c00) with alignment 0 on this only when bit 0 of the flag argument is set; returns this. |

**ModelDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00728180` | ModelDefinitionManager::ClearContainers | confirmed | Virtual slot [2] of ModelDefinitionManager. Bytes: push esi; mov esi,ecx; lea ecx,[esi+4]; call 00727cf0 (on the embedded object at this+4); lea ecx,[esi+0xfb4… |
| `00729320` | ModelDefinitionManager::~ModelDefinitionManager | confirmed | fastcall (ECX = this) with an EH frame. Stores ModelDefinitionManager::vftable at this+0; calls 00727cf0 on this+4 (the embedded object) and 00727940 on the 10… |
| `007294b0` | ModelDefinitionManager::ModelDefinitionManager | confirmed | thiscall constructor (ECX = this, returns this). Stores BaseDefinitionManager::vftable, then ModelDefinitionManager::vftable at this+0; zeroes this+4, +8, +0xc… |
| `007295d0` | ModelDefinitionManager::ScalarDeletingDtor | confirmed | Virtual slot [0] of ModelDefinitionManager. Bytes: push esi; mov esi,ecx; call 00729320 (the ModelDefinitionManager destructor body); if bit 0 of the flags arg… |

**ParticleDefinitionAsset** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d33d60` | ParticleDefinitionAsset::CreateTextureAsset | confirmed | Allocates 0x34 bytes, constructs a TextureAsset in that block, and returns the constructor result. |
| `00d33ff0` | ParticleDefinitionAsset::~ParticleDefinitionAsset | confirmed | Releases the requester and ref-counted entry state, cleans the object's arrays, then calls the base Asset destructor. |
| `00d341e0` | ParticleDefinitionAsset::OnAssetUnavailable | confirmed | Sets the unavailable state, logs the missing particle asset to ParticleLoadErrors.log, and invokes the optional requester callback. |
| `00d342a0` | ParticleDefinitionAsset::CheckParseAndNotify | confirmed | Checks parse and load-completion state, logs parse failure to ParticleDefinitions.log, and notifies the requester when the asset is ready. |
| `00d343d0` | ParticleDefinitionAsset::AddDefinitionEntry | confirmed | Appends and retains a child definition object, updates the entry count, and then checks parse completion and requester notification. |

**RailDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00928c20` | RailDefinitionManager::Ctor | confirmed | Installs BaseDefinitionManager then RailDefinitionManager vtables and initializes inherited storage through 00925040. |
| `00928c80` | RailDefinitionManager::Dtor | confirmed | Destroys the rail hash/list map through 0091fe30, then restores the BaseDefinitionManager vtable. |
| `00928ce0` | RailDefinitionManager::ScalarDeletingDtor | confirmed | Runs full manager cleanup and frees through 00401c00 when the deleting flag is set. |
| `0104a6d0` | RailDefinitionManager::DrainMap | likely | Loops while this+0xc is non-null, calling 008fab60 thiscall on this+4 with the current head node as its argument; returns void. |

**SkyConvolvedTexture** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008077e0` | SkyConvolvedTexture::InitFromParent | confirmed | Thiscall ctor with two stack arguments after this. Calls 0071b950, copies param_2+0x1c8 into this+0xc, stores the AssetRequester vtable at this+8, stores SkyTe… |
| `00807950` | SkyConvolvedTexture::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: SEH-framed call to 00804800 (base teardown), then if bit 0 of the flag is set calls _free(this) (00d82463, thunk to 00d8a9d8), and… |

**SkyDefinition::Horizon** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00806890` | SkyDefinition::Horizon::InitFromSource | confirmed | Thiscall ctor. Stores SkyDefinition::TintedTexture::vftable, copies the dword at param_2+0x10 (AddRef on its +4 count) and the dwords at +0x20..+0x2c into this… |
| `008079f0` | SkyDefinition::Horizon::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: saves ECX to ESI, calls 00807420 (TintedTexture teardown body, DtorB) with this, then if bit 0 of the flag is set calls _free(this… |

**SkyDefinition::Satellite** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00806930` | SkyDefinition::Satellite::InitFromSource | confirmed | Thiscall ctor. Stores SkyDefinition::TintedTexture::vftable, copies the dword at param_2+0x10 (AddRef on its +4 count) and the dwords at +0x20..+0x2c into this… |
| `00807a10` | SkyDefinition::Satellite::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: saves ECX to ESI, calls 00807490 (TintedTexture teardown body, DtorC) with this, then if bit 0 of the flag is set calls _free(this… |

**SkyDefinition::TintedTexture** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00805fd0` | SkyDefinition::TintedTexture::DefaultCtor | confirmed | Fastcall ctor with this in ECX: stores SkyDefinition::TintedTexture::vftable at this+0 and at this+0x30, zeroes this+0x10, this+0x40 and this+0x60, and fills t… |
| `008069d0` | SkyDefinition::TintedTexture::InitFromSourceWithSub | confirmed | Thiscall ctor. Stores SkyDefinition::TintedTexture::vftable at this+0 and at this+0x30, copies param_2+0x10 into this+0x10 (AddRef on its +4 count) and param_2… |
| `00806b50` | SkyDefinition::TintedTexture::InitFromBlockAndHandle | confirmed | Thiscall ctor taking param_2 (a dword handle) and param_3 (source block). Stores the TintedTexture vtable at this+0, copies param_3+0x10 (AddRef on its +4 coun… |
| `008079b0` | SkyDefinition::TintedTexture::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: stores SkyDefinition::TintedTexture::vftable at this+0, releases the refcounted object at this+0x10 (LOCK decrement, slot +4 at ze… |
| `00806aa0` | SkyDefinition::TintedTexture::Dtor | likely | SEH-framed body: stores SkyDefinition::TintedTexture::vftable at this+0, takes the refcounted object at this+0x10, decrements its +4 count with LOCK, calls its… |
| `00807380` | SkyDefinition::TintedTexture::DtorTwoRefs | likely | Fastcall SEH-framed teardown: stores the TintedTexture vtable at this+0x30, releases the refcounted object at this+0x40 (LOCK decrement of its +4 count, slot +… |
| `00807420` | SkyDefinition::TintedTexture::DtorB | likely | Fastcall SEH-framed teardown identical in body to SkyDefinition::TintedTexture::Dtor (00806aa0): stores the TintedTexture vtable at this+0, releases the refcou… |
| `00807490` | SkyDefinition::TintedTexture::DtorC | likely | Fastcall SEH-framed teardown identical in body to SkyDefinition::TintedTexture::Dtor (00806aa0): stores the TintedTexture vtable at this+0, releases the refcou… |
| `00808f40` | SkyDefinition::TintedTexture::ScalarDeletingDtorB | likely | Thiscall with one byte flag: calls 00806aa0 (TintedTexture single-reference dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns… |

**SkyModelDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00807500` | SkyModelDefinition::InitFromParent | confirmed | Thiscall ctor, 3 stack args plus a char flag (param_4). Calls 0071b950 (RefCounted base init), copies the dword at param_2+0x1c8 into this+0xc, stores the Asse… |
| `00807720` | SkyModelDefinition::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: calls 00807660 (the SkyModelDefinition dtor body), then if bit 0 of the flag is set calls _free(this) (00d82463, thunk to 00d8a9d8… |
| `00807660` | SkyModelDefinition::DtorBody | likely | Fastcall SEH-framed teardown: stores SkyModelDefinition::vftable at this+0 and this+8; if the object at this+0x14 exists it calls slot +0x3c of the object at t… |

**SkyTexture** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00807740` | SkyTexture::InitFromParent | confirmed | Thiscall ctor with a char flag (param_4). Calls 0071b950 (RefCounted base init), copies the dword at param_2+0x1c8 into this+0xc, stores the AssetRequester vta… |

**SkyTextureAsset** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008095d0` | SkyTextureAsset::InitWithHandle | confirmed | Thiscall ctor with one stack argument (param_2): calls 0071bb80 (asset base constructor, accepted label AssetConstructor in another shard), stores SkyTextureAs… |
| `008096b0` | SkyTextureAsset::ScalarDeletingDtor | confirmed | Thiscall with one byte flag: calls 00809630 (SkyTextureAsset dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `00809630` | SkyTextureAsset::DtorBody | likely | Fastcall SEH-framed teardown: stores SkyTextureAsset::vftable at this+0, releases the refcounted object at this+0x2c (LOCK decrement, slot +4 at zero, pointer … |

**SoeUtil::Hash<ContentPack,128,128>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01047c60` | Hash_ContentPack128_128::Ctor | confirmed | Fastcall, ECX = this. Calls 01047960 (the HashMap<int,ContentPack,128,128> constructor body) and then stores SoeUtil::Hash<ContentPack,128,128>::vftable (0187f… |
| `01047cc0` | Hash_ContentPack128_128::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Calls 01046f00 (the 128/128 map destructor body), then if bit 0 of the flag is set calls _free on the object. Returns the ob… |

**SoeUtil::Hash<ItemCategoryGroupDefinition*,20,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010452d0` | Hash_ItemCategoryGroupDefinition::Ctor | confirmed | Fastcall, ECX = this. Stores the HashMap vtable (0187f1ec) at *this, zeroes this+8 and this+4, memsets 0x50 bytes at this+0xc, then stores the Hash<ItemCategor… |
| `01045350` | Hash_ItemCategoryGroupDefinition::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Stores the HashMap vtable (0187f1ec, not the Hash vtable) at *this, calls 008abd80 (20-bucket drain), then if bit 0 of the f… |
| `010450f0` | Hash_ItemCategoryGroupDefinition::FreeBlock4 | likely | One stack arg (ret 4). Bytes 8b 44 24 04 6a 04 50 e8 .. 83 c4 08 c2 04 00 = push 4; push arg; call 00401c00; ret 4. Passes the argument pointer and alignment 4… |

**SoeUtil::HashMap<int,ContentPack,128,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010462a0` | HashMap_ContentPack::Ctor | confirmed | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ContentPack,128,-1>::vftable (0187f2a4) at *this, zeroes this+8, memsets 0x200 bytes at this+0xc (128 bucket … |
| `01046d20` | HashMap_ContentPack::DtorBody | confirmed | Fastcall, ECX = this. Bytes c7 01 .. e9 .. = mov [ecx], 0187f2a4 (HashMap<int,ContentPack,128,-1> vtable), then jmp 01046b70 (tail-jump to the table clear). No… |
| `01046dd0` | HashMap_ContentPack::ScalarDeletingDtor | confirmed | Thiscall with a flag byte. Bytes: push esi; mov esi,ecx; mov [esi], 0187f2a4; call 01046b70 (clear, ECX = esi); test byte [esp+8],1; je; push esi; call _free; … |
| `01046490` | HashMap_ContentPack::FreeBlock4 | likely | One stack arg (ret 4). Bytes 8b 44 24 04 6a 04 50 e8 .. 83 c4 08 c2 04 00 = push 4; push arg; call 00401c00; ret 4. Frees the argument with alignment 4 through… |

**SoeUtil::HashMap<int,ContentPack,128,128>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01046f00` | HashMap128_128_DtorBody | confirmed | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ContentPack,128,128>::vftable (0187f450) at *this, calls 01046b70 (clear the 128-bucket nodes), reads the poi… |
| `01047940` | HashMap128_128::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Calls 01046f00 (the 128/128 destructor body), then if bit 0 of the flag is set calls _free on the object. Returns the object… |

**SoeUtil::HashMap<int,ItemCategoryGroupDefinition*,20,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01044f90` | HashMap_ItemCategoryGroupDefinition::Ctor | confirmed | Fastcall, ECX = this. Stores SoeUtil::HashMap<int,ItemCategoryGroupDefinition*,20,-1>::vftable (0187f1ec) at *this, zeroes this+8 and this+4, memsets 0x50 byte… |
| `010450a0` | HashMap_ItemCategoryGroupDefinition::DtorBody | confirmed | Fastcall, ECX = this. Bytes c7 01 .. e9 .. = mov [ecx], 0187f1ec (HashMap vftable), then jmp 008abd80. It is a tail-jump: it stores the vtable and transfers to… |
| `01045110` | HashMap_ItemCategoryGroupDefinition::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Stores the HashMap vtable (0187f1ec) at *this, calls 008abd80 (20-bucket drain), then if bit 0 of the flag is set calls _fre… |
| `01045280` | HashMap_ItemCategoryGroupDefinition::DtorCleanup | confirmed | Fastcall, ECX = this. Stores the HashMap vtable (0187f1ec) at *this, then calls 008abd80 (20-bucket drain) with this as the receiver (the cookie value Ghidra s… |

**StringHashToValueManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01049790` | StringHashToValueManager::DtorBody | confirmed | Fastcall, ECX = this. Stores StringHashToValueManager::vftable (01805cb0) at *this, calls 01049750 (ClearTables), then calls 00917e70 and 008f1610 (member clea… |

**TintGroupDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0104c4a0` | TintGroupDefinitionManager::DrainMap | confirmed | While the head at this+0xc is non-null, calls 0104c130 (RemoveEntry) with ECX=this+4 and the head as argument; returns void. |
| `0104c5a0` | TintGroupDefinitionManager::DtorBody | confirmed | Restores TintGroupDefinitionManager::vftable, runs the 10-bucket map teardown 0104c530 on this+4, then stores BaseDefinitionManager::vftable; no free. |
| `0104c6a0` | TintGroupDefinitionManager::Ctor | confirmed | Installs BaseDefinitionManager::vftable then TintGroupDefinitionManager::vftable into *this, runs the 10-bucket container constructor 0104c600, and returns thi… |
| `0104c700` | TintGroupDefinitionManager::ScalarDeletingDtor | confirmed | Runs the manager teardown 0104c5a0, then calls FreeMemory (00401c00) with alignment 0 on this only when bit 0 of the flag argument is set; returns this. |

**(no class)** (44)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989b90` | LogMissingImageMapping | confirmed | Searches the image-set and image-type tables; on a missing result it formats and logs the corresponding ImageSetId or ImageType error to MissingImages.log. |
| `00989cc0` | FindImageNameById | confirmed | Searches three integer-keyed tables for the supplied image id; on a hit assigns the stored string and returns 1, otherwise logs the miss and tries fallback id … |
| `0098c390` | ParseImageDefinitionsFile | confirmed | Reads caret-delimited image records, skips lines beginning with #, parses id, filename, and content id, logs each record, checks duplicate names, and populates… |
| `0098cc00` | LoadImageDefinitionFiles | confirmed | Builds a path to Images.txt and parses it, builds a path to ImageSetMappings.txt and parses that file, then destroys its temporary fixed string. |
| `009fe280` | LoadGodBeamDefinitions | confirmed | Clears prior nodes, reads GodBeams.txt, parses each non-comment row's key and remaining numeric/vector fields, inserts the definition into a hash-list map, and… |
| `010265f0` | LoadNotificationImagesTxt | confirmed | Reads the text of the NotificationImages.txt resource chunk by chunk through 006fc270, handles each line: skips lines whose first character is '#', passes the … |
| `01027fb0` | LoadItemSortTypes | confirmed | Empties the 32-bucket map at this+0x20 through 01026e80, reads ItemSortTypes.txt chunk by chunk, skips lines starting with '#' and lines of 2 characters or few… |
| `0102a4d0` | LoadLobbyGameTables | confirmed | Clears five hash tables (008cc3d0, 008cc480, 008cc530, 008e7780, 008cc5f0), then formats the file names %sLobbyGameOptionMap.txt, %sLobbyGameOptionValueMap.txt… |
| `0102af70` | LoadAbilityEffectTypeSettings | confirmed | Reads AbilityEffectTypeSettings.txt chunk by chunk through 00fbf4a0 and 006fc270, skips lines starting with '#' or of 2 characters or fewer, parses the first i… |
| `0104a090` | RailDefinitionCopyCtor | confirmed | Thiscall, one stack arg (source record). Copies the dwords at +0 and +4 and the bytes at +8 and +9 and the dword at +0xc from the source. Stores SoeUtil::Array… |
| `010a9a40` | MeshSlotHolderDtorBody | confirmed | Destructor body (ECX = holder): releases the refcounted object at +0 (LOCK decrement of its +4 count, its vtable +4 when it reaches zero) and zeroes +0, then s… |
| `007b7dd0` | HashList4_RemoveByStringIntIntKey | likely | Computes the case-folded hash of the key string (IString at +4 of the key record) with 00705b00, walks the bucket chain (head this+0x284+(hash&3)*4, chain next… |
| `007b7ea0` | FixedStringMember0x10_Dtor | likely | Sets up an SEH frame plus stack cookie, loads ECX with this+0x10 and calls 00703350 (fixed-string destructor body) on that member, then restores the exception … |
| `007b8210` | SixIStringRecord_CopyCtor | likely | Copy-constructs a 0x74-byte array element from the record at param_2: initialises the six embedded SoeUtil::IString<char> members at +0, +0x10, +0x24, +0x34, +… |
| `007b83a0` | SixIStringArray_GrowMove | likely | Grows the array at ECX (data pointer +4, count +8, capacity +0xc, allocator/free function pointers reached through the object's own vtable slots +4 and +8): al… |
| `007b8470` | IStringChildRecord_CopyCtor | likely | Copy-constructs a 0x24-byte list child record from the source at param_2: initialises the IString at +0 (vtable plus empty base) and copy-assigns it from the s… |
| `007b8500` | TwoIStringChildListRecord_CopyCtor | likely | Copy-constructs a 0x38-byte record: two embedded SoeUtil::IString<char> at +0 and +0x10 (vtable plus empty base, copy-assigned with 006f9e60 from the source's … |
| `007b8600` | TwoStringsAndStringList_Dtor | likely | Destructor body (ECX = object, SEH framed) that first calls 007b7f10 with ECX = this+0x24 to destroy every node of the string list there, then releases the ISt… |
| `007b8710` | TwoIStringValueRecord_Ctor | likely | Constructor (thiscall, four stack arguments) of a record that starts with a dword (param_2, usually a vtable or id), followed by a SoeUtil::IString<char> at +4… |
| `007b87b0` | HashList4_InsertUniqueStringIntInt | likely | Inserts the caller-built node into the 4-bucket hash list at this+0x274 but only when the (IString, int, int) key is not already present: hashes node+4 with 00… |
| `007b9100` | FixedString32Record_ScalarDeletingDtor | likely | Scalar deleting destructor: destroys the fixed string member at this+0x10 by calling 00703350 with ECX = this+0x10 (SEH framed), then, when the flag argument h… |
| `007b93d0` | TwoIStringListRecord_ScalarDeletingDtor | likely | Scalar deleting destructor: calls the non-deleting destructor body 007b8600 with this in ECX, then, if the flag argument has bit 0 set, frees this with _free; … |
| `007b9520` | TwoIStringValueRecord_CtorThroughPointers | likely | SEH-framed constructor wrapper: with the new record in ECX, dereferences its argument pointers (the four dword pointers passed by the caller) and forwards the … |
| `007b9870` | SixIStringArray_AppendCopyN | likely | Appends param_3 copies of consecutive 0x74-byte source records to the array at this (data +4, count +8, capacity +0xc): grows the array through 007b83a0 when c… |
| `007b9a40` | Record_FiveStringArray_CopyAssign | likely | Copy-assigns a record whose first dword is copied raw, followed by five SoeUtil::IString<char> members at +4, +0x14, +0x24, +0x34, +0x44 (all copy-assigned wit… |
| `0098bba0` | FindOrCreateNameId | likely | Hashes the supplied name and searches three ID/name tables; on a miss it increments the next-id field, stores the string and both map directions, then returns … |
| `0098ca00` | ParseImageSetMappingsFile | likely | Reads caret-delimited mapping records, skips comment lines, parses set id, type, and image id, creates a nested map for an unseen set, then inserts or replaces… |
| `00cec340` | ReleaseEffectReferences | likely | Walks the selected effect definition's typed reference list, resolves each referenced object, and decrements its reference count when positive. |
| `00ced500` | StoreDwordAt50 | likely | Stores the supplied 32-bit value at object offset 0x50 and returns. |
| `010296d0` | LoadTextTableRecords10 | likely | Reads the text named by param_1 through 00fbf4a0 and walks it line by line (chunks via 006fc270); lines starting with '#' or of 2 characters or fewer are skipp… |
| `010298c0` | LoadTextTableRecords10Copy | likely | Same body as 010296d0 (0x1c-byte records, three separator-0x5e tokens, skip '#' and short lines), except that each record is inserted through 008da080 (the sec… |
| `01029ab0` | LoadTextTableRecords10Key1c | likely | Reads the text named by param_1 line by line (same skip rules as 010296d0). Each line allocates a 0x28-byte record, constructs an embedded IString at +8 throug… |
| `01029ca0` | LoadTextTableRecords10Key18 | likely | Reads the text named by param_1 line by line (same skip rules). Each line allocates a 0x24-byte record, zeroes its dwords at +0x14..+0x20, parses the line with… |
| `01029e70` | LoadTextTableRecords100 | likely | Reads the text named by param_1 line by line (same skip rules). Each line allocates a 0x20-byte record, zeroes its dwords at +0..+0x10, parses the line with 01… |
| `0104a170` | RailNodeCtor | likely | Thiscall, two stack args (param_2 = pointer to a dword key, param_3 = source record). Calls 0104a090 (RailDefinitionCopyCtor) with the source, then stores the … |
| `0104d220` | Record_IconSub_CopyInitKey | likely | Copy-constructs this from param_3 through 0104d020, then stores *param_2 at this+0x30 as the key; returns this. |
| `0104f120` | Record_IconSub_CopyCtor_B | likely | Copies the dwords at +0, +4, +8 from param_2, runs the IconData copy constructor 008b2bd0 on param_2+0xc, then copies param_2's dwords at +0x18, +0x1c, +0x20, … |
| `0104f2c0` | Record_IconSub_CopyInitKey_B | likely | Copy-constructs this from param_3 through 0104f120, then stores *param_2 at this+0x30 as the key; returns this. |
| `01052000` | ParseRowDwordsAndToken | likely | Thiscall (this in ECX; param_2 = source cursor, param_3 = context). Reads a value through 0072d900 into this+0, a token through 006fc080 into this+4, then valu… |
| `010a7fe0` | SkeletonResolveBoneMap | likely | Thiscall (ECX = skeleton definition; one stack arg = lookup table pointer): for each bone i below the count at +0x50, sets map[i] (array at +0x60) to i when th… |
| `010a8040` | SkeletonBoneMapVerify | likely | Fastcall (ECX = skeleton definition, no stack args): returns 0 when any bone-map entry at +0x60 is negative or differs from 010aae40 applied to the bone key at… |
| `010a99e0` | MeshSlotHolderCtor | likely | Constructor (ECX = holder): zeroes the dword at +0, stores the Array<Deep::MeshData::Slot,0,1> vftable at +4 and zeroes its buffer, count and capacity at +8, +… |
| `010a9bc0` | MeshSlotHolderAssignRef | likely | Thiscall (ECX = holder; one stack arg = source object whose +0x10 is a count): adds one reference to the source (LOCK increment of its +4), stores the source p… |
| `010baf30` | NodeListSaveFile | likely | Thiscall, ECX=owner, two stack arguments (path and mode/flags). Calls 010ba7c0 on the third argument, opens the file with 00703e10 (mode 3), and on success wal… |

### `asset.delivery`

**AssetManagement::AssetDeliveryDirect** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0109f730` | AssetDeliveryDirect::AssetDeliveryDirect | confirmed | Thiscall with three stack arguments. Runs the AssetDeliveryInterface base ctor 0109c820, installs the AssetDeliveryDirect vftable, installs the IString vftable… |
| `0109f950` | AssetDeliveryDirect::~AssetDeliveryDirect | confirmed | Destructor body. Installs the AssetDeliveryDirect vftable, calls 0109e360 (ReleaseLoadersAndQueue), calls 0109f6a0 (ReleaseBuffer9344), installs the IString vf… |
| `0109fa00` | AssetDeliveryDirect::ScalarDeletingDestructor | confirmed | Virtual slot 0. Calls the destructor body 0109f950 on this, then calls _free on the object when bit 0 of the flag argument is set; returns the object pointer. |
| `0109e3d0` | AssetDeliveryDirect::InitFileLoaders | likely | Virtual slot [1] of AssetDeliveryDirect. Calls 0109e360 to release any previous queue and loaders, allocates 0xb0 bytes and constructs a FileRequestQueue (0109… |
| `0109ee50` | AssetDeliveryDirect::GetAssetData | likely | Thiscall with four stack dwords (ret 0x10). Builds the name IString of the request at arg1+4 via 01093aa0, logs to DirectAssets.txt through 00704db0, looks the… |
| `0109f6a0` | AssetDeliveryDirect::ReleaseBuffer9344 | likely | Fastcall with this in ECX, no stack arguments. Sets the member at +0x9340 to the SoeUtil::IString vftable. If the count at +0x934c is positive, decrements the … |

**AssetManagement::AssetDeliveryDirect::FileLoader** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0109dfa0` | AssetDeliveryDirect::FileLoader::FileLoader | confirmed | Constructor: calls 00fc4cc0, installs the FileLoader vtable, zeroes the list links at +0xc4 and +0xc8, stores the owner queue pointer param_2 at +0xcc and incr… |
| `0109e030` | AssetDeliveryDirect::FileLoader::~FileLoader | confirmed | Destructor body of FileLoader: installs the FileLoader vtable and runs the base teardown 00fc4c30. |
| `0109e080` | AssetDeliveryDirect::FileLoader::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: installs the FileLoader vtable, runs the base teardown 00fc4c30, and calls _free on the object when bit 0 of the stack argume… |
| `0109ecd0` | AssetDeliveryDirect::FileLoader::ProcessQueue | likely | Loops while 00fc4ac0 returns 0. Each pass: under lock 00fc45f0 takes the head node of the list at owner+0x8 (count at owner+0x10), pushes it onto the front of … |

**AssetManagement::AssetDeliveryDirect::FileRequestQueue** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0109e900` | AssetDeliveryDirect::FileRequestQueue::ScalarDeletingDestru… | confirmed | Scalar deleting destructor shape: calls the FileRequestQueue body 0109e890, then calls _free when bit 0 of the stack argument is set; returns the object pointe… |
| `0109e590` | FileRequestQueue::FindItemByNameAndRef | likely | Virtual slot [3] of FileRequestQueue, two stack arguments: under the lock 00fc45f0 scans the lists at +8 and +0x14 for a node whose word at +0x28 equals the th… |

**AssetManagement::AssetDeliveryHandlerInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010935e0` | AssetManagement::AssetDeliveryHandlerInterface::DtorBody | confirmed | Destructor body: stores AssetManagement::AssetDeliveryHandlerInterface::vftable into *this and, when the pointer at this+4 is non-null, calls its vtable slot +… |

**AssetManagement::AssetDeliveryIndirect** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0109a390` | AssetDeliveryIndirect::InitPackStorage | confirmed | Virtual slot [1] of AssetDeliveryIndirect. When the flag at +0x624 is clear: creates the cache directory (00fc72c0); creates a transport object chosen by URL p… |
| `0109afc0` | AssetDeliveryIndirect::GetAssetData | confirmed | Virtual slot [3] of AssetDeliveryIndirect. Converts the request name at param_2+4 to an IString, logs to IndirectAssets.txt, looks the name up with 0109a810 (l… |
| `0109b2a0` | AssetDeliveryIndirect::RetryRequest | confirmed | Increments the retry counter at +4 of the pending request, logs the retry attempt and name, looks the name up with 0109a810, then re-issues the manifest record… |
| `0109b400` | AssetDeliveryIndirect::AssetReceived | confirmed | Virtual slot [0] of the secondary AssetDeliveryIndirect vtable 018854e0. Converts the received name, strips a trailing .z marker (sets the compressed flag), lo… |
| `0109bb00` | AssetDeliveryIndirect::AssetErrorReceived | confirmed | Virtual slot [1] of the secondary AssetDeliveryIndirect vtable 018854e0. Converts the name, strips a trailing .z marker, walks the pending entries for that nam… |
| `0109bd00` | AssetDeliveryIndirect::OnManifestCrcReceived | confirmed | Virtual slot [3] of the secondary AssetDeliveryIndirect vtable 018854e0. With no CRC body it logs a failure. Otherwise it builds the local manifest path (%s/%s… |
| `0109bea0` | AssetDeliveryIndirect::ManifestReceived | confirmed | Virtual slot [2] of the secondary AssetDeliveryIndirect vtable 018854e0. A null body logs a corrupted-download failure. Otherwise it computes the CRC of the bo… |
| `0109c000` | AssetDeliveryIndirect::OnFinished | confirmed | Virtual slot [1] of AssetDeliveryIndirect (vtable 018854c8). Reads the job status through the job vtable slot at +0x18 and dispatches: the read-status branch d… |
| `0109d960` | AssetDeliveryIndirect::AssetDeliveryIndirect | confirmed | Constructor: runs the AssetDeliveryInterface base constructor 0109c820, installs the RemoteDeliveryHandler and JobQueueHandler vtables at +0x618 and +0x61c and… |
| `0109db20` | AssetDeliveryIndirect::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the AssetDeliveryIndirect body 0109d640, then calls _free on the object when bit 0 of the stack argument is set; return… |
| `0109a810` | AssetDeliveryIndirect::FindManifestEntry | likely | Hashes the name argument with 00705b00 and scans the 10000-bucket table at this+0xcf38 for a record with the same hash (key at +0x64) and equal name (006fc5b0)… |

**AssetManagement::AssetDeliveryIndirect::DecompressJob** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099fb0` | DecompressJob::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the DecompressJob body 010998b0, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
| `01099fd0` | DecompressJob::Decompress | confirmed | Virtual slot [2] of DecompressJob. Reads the input buffer at this+0xb0; 010ebd70 returns the output length from the envelope header, the output RefArray is gro… |

**AssetManagement::AssetDeliveryIndirect::PendingRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099be0` | PendingRequest::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the PendingRequest body 01099b80 unconditionally, then calls _free on the object when bit 0 of the stack argument is se… |

**AssetManagement::AssetDeliveryIndirect::ReadJob** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099da0` | ReadJob::ReadJob | confirmed | Constructor: calls 010995e0 (PackJob base constructor, which installs the PackJob vtable), installs the ReadJob vtable, constructs a StringFixed<64> at offset … |
| `01099e30` | ReadJob::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the ReadJob body 01099730, then calls _free on the object when bit 0 of the stack argument is set; returns the object p… |

**AssetManagement::AssetDeliveryIndirect::WriteJob** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099f90` | WriteJob::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the WriteJob body 010997f0, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |

**AssetManagement::AssetDeliveryInterface** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010937d0` | AssetManagement::AssetDeliveryInterface::ClearPendingRef | confirmed | Virtual slot [19]: walks the list headed at this+0x40 and the list headed at this+0x2e8 (node next pointer at +0x20) and sets the node's +4 field to 0 wherever… |
| `01093820` | AssetManagement::AssetDeliveryInterface::ContainsPendingRef | confirmed | Virtual slot [20]: returns 1 when a node in the list at this+0x40 or in the list at this+0x2e8 has its +4 field equal to the argument, otherwise 0. |
| `01093860` | AssetManagement::AssetDeliveryInterface::FindEntryByNameAnd… | confirmed | Virtual slot [21]: searches the lists at this+0x40 and this+0x2e8: for each node it compares the node name at +0xc with the string at (arg2+4) through 006fc5b0… |
| `0109c720` | AssetDeliveryInterface::~AssetDeliveryInterface | confirmed | Destructor body: installs the AssetDeliveryInterface vtable, runs the base teardown 00fc45a0, destroys the PendingEntryError list member (0109c5f0) and the Pen… |
| `0109c800` | AssetDeliveryInterface::ScalarDeletingDestructor | confirmed | Scalar deleting destructor shape: calls the AssetDeliveryInterface body 0109c720, then calls _free when bit 0 of the stack argument is set; returns the object … |
| `0109c820` | AssetDeliveryInterface::AssetDeliveryInterface | confirmed | Constructor: installs the AssetDeliveryInterface vtable, runs the base constructor 00703540, constructs the PendingEntryData list member (0109b830) and the Pen… |

**AssetManagement::AssetDeliveryInterface::PendingEntry** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01093bf0` | AssetManagement::AssetDeliveryInterface::PendingEntry::Scal… | confirmed | Virtual slot [0]: calls the PendingEntry destructor body 010938d0 on this, then frees the block with _free when bit 0 of the argument byte is set; returns this… |
| `01093d10` | AssetManagement::AssetDeliveryInterface::PendingEntry::Ctor | confirmed | Constructor: stores AssetManagement::AssetDeliveryInterface::PendingEntry::vftable at *this, stores the IString vftable at +8 with the shared empty buffer DAT_… |

**AssetManagement::AssetDeliveryInterface::PendingEntryData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01093940` | AssetManagement::AssetDeliveryInterface::PendingEntryData::… | confirmed | Destructor body: stores AssetManagement::AssetDeliveryInterface::PendingEntryData::vftable into *this, decrements the reference count at (object at +0x1c)+0x14… |
| `01093c10` | AssetManagement::AssetDeliveryInterface::PendingEntryData::… | confirmed | Virtual slot [0]: calls the PendingEntryData destructor body 01093940 on this, then frees the block with _free when bit 0 of the argument byte is set; returns … |
| `01093d90` | AssetManagement::AssetDeliveryInterface::PendingEntryData::… | confirmed | Constructor: calls the PendingEntry constructor 01093d10 on the arguments, stores param_5 at +0x1c, stores the PendingEntryData vftable at *this and increments… |

**AssetManagement::AssetDeliveryInterface::PendingEntryError** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01093e10` | AssetManagement::AssetDeliveryInterface::PendingEntryError:… | confirmed | Constructor: calls the PendingEntry constructor 01093d10 on the first three arguments, stores the PendingEntryError vftable at *this and param_5 at +0x1c. |
| `01093e80` | AssetManagement::AssetDeliveryInterface::PendingEntryError:… | confirmed | Virtual slot [0]: calls the PendingEntry destructor body 010938d0 on this, then frees the block with _free when bit 0 of the argument byte is set; returns this. |

**AssetManagement::AssetDeliveryWrapper** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0109fd10` | AssetDeliveryWrapper::Initialize | confirmed | Virtual slot 1. Allocates 0x9998 bytes and builds the direct delivery object 0109f730 into +0x618 (or null when the allocation fails), allocates 0x16eb0 bytes … |
| `010a0370` | AssetDeliveryWrapper::ScalarDeletingDestructor | confirmed | Virtual slot 0. Calls the destructor body 010a0100 on this, then calls _free on the object when bit 0 of the flag argument is set; returns the object pointer. |
| `0109fae0` | AssetDeliveryWrapper::GetAssetDataDispatch | likely | Thiscall, four stack arguments (ret 0x10). Calls slot 11 (+0x2c) of the direct delegate with the first argument; if that returns nonzero it calls slot 3 (+0xc)… |
| `0109fb40` | AssetDeliveryWrapper::GetFallbackDelegate | likely | Virtual slot 12. Returns the dword at this+0x61c (the second delegate pointer) in EAX. |
| `0109fb50` | AssetDeliveryWrapper::GetPrimaryDelegate | likely | Virtual slot 13. Returns the dword at this+0x618 (the first delegate pointer) in EAX. |
| `0109fe60` | AssetDeliveryWrapper::DescribeSources | likely | Virtual slot 10, one stack argument (ret 4). Calls slot 10 (+0x28) of the delegates at +0x618 and +0x61c into two local StringFixed<128> values. If both are no… |
| `0109ff90` | AssetDeliveryWrapper::NameAvailableEither | likely | Virtual slot 23, thiscall with one stack argument (the name) and a bool result. Assigns the name to a temporary IString through 00402ce0, calls slot 11 (+0x2c)… |

**AssetManagement::AssetRequestInterface::AssetRequester** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989810` | AssetRequester::Destructor | confirmed | Releases the IString member at +0xc when its length is positive, installs the AssetRequester vtable, then calls the nonnull member at +4 through slot +0x18 wit… |

**ImageDirectory** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098d860` | ImageDirectory::Destructor | confirmed | Installs the ImageDirectory vtable, calls shared service helpers, clears nested-map and string/integer caches, and clears pending image-load and portrait maps. |
| `0098dab0` | ImageDirectory::Constructor | confirmed | Installs the ImageDirectory vtable, clears manager state, initializes pending-load and portrait maps plus integer and string lookup maps, sets a timestamp-like… |

**ImageDirectory::ImageAsset2** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989b70` | ImageAsset2::ScalarDeletingDestructor | confirmed | Calls the ImageAsset2 destructor body; frees this only when deleting flag bit 0 is set, then otherwise returns this. |
| `0098af60` | ImageAsset2::Method2 | confirmed | Initializes a temporary IString, obtains a string-producing value through 0071c050, assigns from the returned data pointer plus five bytes, releases the tempor… |

**ImageDirectory::PendingLoad2** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989dc0` | ImageDirectory::PendingLoad2::ScalarDeletingDestructor | confirmed | Calls the AssetRequester destructor body, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
| `0098b0d0` | ImageDirectory::PendingLoad2::ReportUnavailableAsset | confirmed | Formats and logs the unavailable asset name, image id, and URL; releases and detaches its image reference, then removes the nested pending-load entry. |

**RemoteAssetDelivery::AssetDataPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a31a0` | AssetDataPacket::AssetDataPacket 🌐 | confirmed | Fastcall, no arguments. Installs the BasePacket vftable, writes the dword 4 at +0x4, installs the AssetDataPacket vftable at *this, and constructs a StringFixe… |
| `010a3210` | AssetDataPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the packet release 010a2f60, then calls _free when bit 0 of the flag is set; returns the object pointer. |

**RemoteAssetDelivery::AssetErrorPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a3230` | AssetErrorPacket::AssetErrorPacket 🌐 | confirmed | Fastcall, no arguments. Installs the BasePacket vftable, writes the dword 5 at +0x4, installs the AssetErrorPacket vftable at *this, and constructs a StringFix… |
| `010a32a0` | AssetErrorPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the packet release 010a2fe0, then calls _free when bit 0 of the flag is set; returns the object pointer. |

**RemoteAssetDelivery::AssetRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a3110` | AssetRequestPacket::AssetRequestPacket 🌐 | confirmed | Thiscall with one stack argument (the name). Installs the BasePacket vftable, writes the dword 1 at +0x4, installs the AssetRequestPacket vftable, then copies … |
| `010a3180` | AssetRequestPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the packet release 010a2f00, then calls _free when bit 0 of the flag is set; returns the object pointer. |

**RemoteAssetDelivery::AssetServerApi** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a29a0` | AssetServerApi::~AssetServerApi | confirmed | Destructor body, fastcall. Installs the AssetServerApi vftable at *this, at +0xc0 and at +0xc4, then calls 010d65d0 (base teardown, label not known) and return… |
| `010a2a00` | AssetServerApi::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the destructor body 010a29a0, then calls _free on the object when bit 0 of the flag is set; returns the object… |
| `010a2a20` | AssetServerApi::AssetServerApi | confirmed | Thiscall with one stack argument (the owner pointer). Calls 010d7650 with the string ASSET_SERVER_05 and the constants 1, 0, 0, 0, 0, 1, then stores the owner … |
| `010a3690` | AssetServerApi::DispatchPacket 🌐 | confirmed | Virtual slot 18 of AssetServerApi, thiscall with the buffer pointer and length. Reads the leading 16-bit message value; for 4, 5, 6 and 7 it constructs the mat… |
| `010a3b00` | AssetServerApi::SendHeaderOnlyPacket | likely | Thiscall with the API in ECX and the packet as the stack argument. Initialises a packet writer (008d4a40 with 0x10000000), copies the packet's 16-bit message v… |
| `010a3c10` | AssetServerApi::SendHeaderOnlyPacket | likely | Thiscall with the API in ECX and the packet as the stack argument. Same sequence as 010a3b00: initialises a packet writer, writes the packet's 16-bit message v… |
| `010a3d20` | AssetServerApi::SendRequestPacket | likely | Fastcall with the API in ECX. Initialises a packet writer (008d4a40 with 0x10000000) in an 8 KB stack array, serialises the request object through 010a3940, th… |
| `010a3df0` | AssetServerApi::SendManifestRequest | likely | Thiscall with the API in ECX. Builds a ManifestRequestPacket on the stack through 010a2790 and passes it to 010a3b00 (SendHeaderOnlyPacket). |
| `010a3e60` | AssetServerApi::SendManifestCrcRequest | likely | Thiscall with the API in ECX. Builds a ManifestCrcRequestPacket on the stack through 010a2840 and passes it to 010a3c10 (SendHeaderOnlyPacket copy). |
| `010a3ed0` | AssetServerApi::RequestAsset | likely | Thiscall, one stack argument (the name). Tests a condition on the API (the result is kept in AL and not fully decoded by the pseudocode). When the test fails i… |

**RemoteAssetDelivery::BasePacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a2770` | BasePacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Stores the BasePacket vftable, then calls _free when bit 0 of the flag is set; returns the object pointer. |
| `010a2e00` | BasePacket::ReleasePayloadString | likely | Fastcall, no arguments. Installs the SoeUtil::IString vftable at this+0x8, releases the refcounted buffer at this+0xc (refcount word at buffer-4) when the leng… |
| `010a2f00` | BasePacket::ReleaseString64 | likely | Thiscall, no arguments. Installs the BasePacket vftable at *this, then runs the IStringFixed<64> destructor 004022b0 on the member at this+0x8 inside an EH fra… |
| `010a2f60` | BasePacket::ReleaseString64 | likely | Thiscall, no arguments. Installs the BasePacket vftable at *this, then runs the IStringFixed<64> destructor 004022b0 on the member at this+0x8 inside an EH fra… |
| `010a2fe0` | BasePacket::ReleaseString64 | likely | Thiscall, no arguments. Installs the BasePacket vftable at *this, then runs the IStringFixed<64> destructor 004022b0 on the member at this+0x8 inside an EH fra… |

**RemoteAssetDelivery::FileRemoteDelivery** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a0c00` | FileRemoteDelivery::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the destructor body 010a07a0, then calls _free on the object when bit 0 of the flag is set; returns the object… |
| `010a0740` | FileRemoteDelivery::RequestManifestCrc | likely | Virtual slot 2 of FileRemoteDelivery, no arguments. Calls virtual slot 7 (+0x1c) of this object with the arguments "manifest.crc", "INTERNAL" and 0x20. |
| `010a07a0` | FileRemoteDelivery::~FileRemoteDelivery | likely | Fastcall, no stack arguments. Installs the FileRemoteDelivery vftables at *this and +0x8, calls 00fc60d0, destroys a string temporary through 00402140, calls 0… |
| `010a0840` | FileRemoteDelivery::ThunkScalarDeletingDtor | likely | Virtual slot 0 of the secondary FileRemoteDelivery vftable at +0x8 (018858c0): an 8-byte adjustor that forwards to 010a0c00, the primary scalar deleting destru… |
| `010a0d00` | FileRemoteDelivery::RequestBucketedFile | likely | Virtual slot 3, thiscall with three stack arguments. Builds the bucketed path of argument 1 through 010a09c0 (FormatHashBucketPath, with the flag at +0x114), t… |
| `010a1180` | FileRemoteDelivery::QueueReadJob | likely | Virtual slot 7 of FileRemoteDelivery, thiscall with three stack arguments and a size. Adds the size argument to the byte counter at this+0x44c, formats a path … |

**RemoteAssetDelivery::FileRemoteDelivery::ReadJob** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a0900` | FileRemoteDelivery::ReadJob::~ReadJob | confirmed | Destructor body. Installs the ReadJob vftable at *this, releases the refcounted byte buffer at this+0x2c4 (refcount at buffer+0x14, deleter vtable slot 1 at ze… |
| `010a1160` | FileRemoteDelivery::ReadJob::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the ReadJob destructor body 010a0900, then calls _free when bit 0 of the flag is set; returns the object point… |
| `010a0850` | FileRemoteDelivery::ReadJob::ReadFileIntoBuffer | likely | Virtual slot 2 of ReadJob, no arguments. Allocates a 0x18-byte RefArray buffer (00d833a6 then 0083fe90) and stores it at this+0x2c4, reads the file named by th… |
| `010a10c0` | FileRemoteDelivery::ReadJob::ReadJob | likely | Thiscall with three stack arguments and a size. Runs the job base constructor 0083f1e0, installs the ReadJob vftable, calls 007e1270 with argument 1, then 0072… |

**RemoteAssetDelivery::HttpRemoteDelivery** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a12d0` | HttpRemoteDelivery::LogStatistics | confirmed | Virtual slot 6, no arguments. Reads four dwords from the object at this+0x14b8 (+0xe0, +0xe4, +0x100, +0x104, +0x108, +0x10c), computes an average rate as (+0x… |
| `010a2360` | HttpRemoteDelivery::~HttpRemoteDelivery | confirmed | Destructor body. Installs the HttpRemoteDelivery vftables at *this and +0x8, frees the HTTP manager object at this+0x14b8 through 00d32f60 (HttpManager::Cleanu… |
| `010a2710` | HttpRemoteDelivery::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the destructor body 010a2360, then calls _free on the object when bit 0 of the flag is set; returns the object… |
| `010a12b0` | HttpRemoteDelivery::GiveTime | likely | Virtual slot 4, no arguments, no return value. Calls 00d31890 with no arguments. |
| `010a1980` | HttpRemoteDelivery::OnRequestComplete | likely | Virtual slot 1 of the secondary HttpRemoteDelivery vftable at +0x8 (01885a8c). Finds the pending request by the key in argument 4 in bucket (key & 0x3ff) of th… |
| `010a1e00` | HttpRemoteDelivery::IssueRequest | likely | Thiscall with four stack arguments. Formats the request URL with 00402fd0 from the base string at this+0x14c0 and argument 2, takes the next request id from th… |
| `010a1f70` | HttpRemoteDelivery::RequestManifestTxtZ | likely | Virtual slot 1 of HttpRemoteDelivery, thiscall with two stack arguments. Assigns 'manifest.txt.z' to a local through 00402ce0, appends the argument 3 text thro… |
| `010a2060` | HttpRemoteDelivery::RequestManifestCrc | likely | Virtual slot 2 of HttpRemoteDelivery, no arguments. Calls 010a1e00 (IssueRequest) with 'manifest.crc', 'INTERNAL', 0x20 and 0. |
| `010a2080` | HttpRemoteDelivery::RequestBucketedAsset | likely | Virtual slot 3 of HttpRemoteDelivery, thiscall with four stack arguments (ret 0x10). Builds the bucketed path of argument 2 through 010a09c0 (FormatHashBucketP… |
| `010a2420` | HttpRemoteDelivery::ThunkScalarDeletingDtor | likely | Virtual slot 0 of the secondary HttpRemoteDelivery vftable at +0x8 (01885a8c): an 8-byte forwarder to 010a2710, the primary scalar deleting destructor. |

**RemoteAssetDelivery::HttpRemoteDelivery::AssetName** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a18b0` | AssetNameCtor | likely | Thiscall with two stack arguments (key pointer, name pointer), returns this. Builds a temporary IString from the name through 00402ce0 and 010a1770, releases t… |

**RemoteAssetDelivery::ManifestCrcDataPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a2c70` | ManifestCrcDataPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Stores the BasePacket vftable at *this, then calls _free when bit 0 of the flag is set; returns the object pointer. |
| `010a28f0` | ManifestCrcDataPacket::ManifestCrcDataPacket 🌐 | likely | Fastcall, no stack arguments. Stores the BasePacket vftable, writes the dword 7 at +0x4, then stores the ManifestCrcDataPacket vftable at *this. |

**RemoteAssetDelivery::ManifestCrcRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a2c10` | ManifestCrcRequestPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Stores the BasePacket vftable at *this, then calls _free when bit 0 of the flag is set; returns the object pointer. |
| `010a2840` | ManifestCrcRequestPacket::ManifestCrcRequestPacket 🌐 | likely | Fastcall, no stack arguments. Stores the BasePacket vftable, writes the dword 3 at +0x4, then stores the ManifestCrcRequestPacket vftable at *this. |

**RemoteAssetDelivery::ManifestDataPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a2fc0` | ManifestDataPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the payload release 010a2e00, then calls _free when bit 0 of the flag is set; returns the object pointer. |
| `010a2d90` | ManifestDataPacket::ManifestDataPacket 🌐 | likely | Fastcall, no arguments. Installs the BasePacket vftable, writes the dword 6 at +0x4, installs the ManifestDataPacket vftable at *this, installs the IString vft… |

**RemoteAssetDelivery::ManifestRequestPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a2bb0` | ManifestRequestPacket::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Stores the BasePacket vftable at *this, then calls _free when bit 0 of the flag is set; returns the object pointer. |
| `010a2790` | ManifestRequestPacket::ManifestRequestPacket 🌐 | likely | Fastcall, no stack arguments. Stores the BasePacket vftable, writes the dword 2 at +0x4, then stores the ManifestRequestPacket vftable at *this. |

**RemoteAssetDelivery::RemoteDeliveryInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a03a0` | RemoteDeliveryInterface::~RemoteDeliveryInterface | confirmed | Destructor body, fastcall with this in ECX and no stack arguments. Stores the RemoteDeliveryInterface vftable into *this and returns. It has no other effect. |
| `010a03d0` | RemoteDeliveryInterface::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Stores the RemoteDeliveryInterface vftable into *this, then calls _free on the object when bit 0 of the flag is set;… |

**RemoteAssetDelivery::UdpRemoteDelivery** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a0410` | UdpRemoteDelivery::~UdpRemoteDelivery | confirmed | Destructor body, fastcall. Stores the UdpRemoteDelivery vftable at *this and at +0x8, deletes the object at +0xc through its vtable slot 0 with flag 1 when it … |
| `010a0570` | UdpRemoteDelivery::UdpRemoteDelivery | confirmed | Thiscall with two stack arguments (owner pointer and a value). Stores the owner at +0x4, installs the UdpRemoteDelivery vftables at *this and +0x8, allocates 0… |
| `010a0640` | UdpRemoteDelivery::ScalarDeletingDestructor | confirmed | Virtual slot 0, thiscall with a flag byte. Calls the destructor body 010a0410, then calls _free on the object when bit 0 of the flag is set; returns the object… |
| `010a0490` | UdpRemoteDelivery::SendManifestRequest | likely | Virtual slot 1 of UdpRemoteDelivery (ret 8, the two stack arguments are not read). Loads the AssetServerApi pointer at this+0xc into ECX and calls 010a3df0 on … |
| `010a04a0` | UdpRemoteDelivery::SendManifestCrcRequest | likely | Virtual slot 2 of UdpRemoteDelivery. Loads the AssetServerApi at this+0xc into ECX and tail-jumps (jmp 010a3e60) to it. |
| `010a04b0` | UdpRemoteDelivery::RequestAsset | likely | Virtual slot 3, four stack arguments (ret 0x10; only the first is used). Loads the AssetServerApi at this+0xc into ECX, pushes the first argument and calls 010… |
| `010a04e0` | UdpRemoteDelivery::HandlerForwardSlot1 | likely | Slot 1 of the secondary UdpRemoteDelivery vftable at this+0x8, four stack arguments (only three are forwarded). If the owner pointer at this+0x4 is non-null it… |
| `010a0510` | UdpRemoteDelivery::HandlerForwardSlot2 | likely | Slot 2 of the secondary UdpRemoteDelivery vftable at this+0x8, arguments forwarded: arg2. If the owner pointer at this+0x4 is non-null it calls the owner's vta… |
| `010a0530` | UdpRemoteDelivery::HandlerForwardSlot3 | likely | Slot 3 of the secondary UdpRemoteDelivery vftable at this+0x8, arguments forwarded: arg2 and arg3. If the owner pointer at this+0x4 is non-null it calls the ow… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0098ae70` | BuildIdi2KeyAndQueryAssetManager | confirmed | Builds an idi2:-prefixed key; queries the manager with Asset and ImageAsset2 RTTI descriptors, then passes the returned pointer to 00d8d382. |
| `0109ae60` | RemoveDeadAssets | confirmed | Walks a global name list from 010f7280 and 010f7290, builds each name as an IString, looks it up with 0109a810, and for a name without a manifest record logs '… |
| `0098b1e0` | CreatePendingImageLoadForId | likely | Looks up the image id's name; when found, allocates and initializes PendingLoad2, stores the id and supplied URL, inserts it in the pending map, then invokes 0… |
| `0109edd0` | KeyedRecordFormatPath | likely | Walks the chain in bucket (key & 0x1fff) of the table at this+0x658 (match key at record+0x90, next at +0x94). If found and record+0x18 > 0 it formats '%s/%s/%… |
| `010a3080` | ReadHeaderI16I32I32 | likely | Thiscall: out in ECX, reader at the stack argument. Reads a sign-extended 16-bit value into out+0x4 and two 32-bit values into out+0x8 and out+0xc from the rea… |
| `010a3300` | ReadI16CountedBytes | likely | Thiscall: out in ECX, reader at the stack argument. Reads a sign-extended 16-bit value into out+0x4, then a 32-bit count; a negative count or insufficient data… |
| `010a3380` | ReadI16BoundedBlob | likely | Thiscall: out in ECX, reader at the stack argument. Reads a sign-extended 16-bit value into out+0x4, then reads a bounded length-prefixed blob through 008bfc00… |
| `010a3410` | ReadI16CountedBytesThenI32 | likely | Thiscall: out in ECX, reader at the stack argument. Reads a sign-extended 16-bit value into out+0x4, a 32-bit counted byte run through 00894b10 (as in ReadI16C… |
| `010a34c0` | ReadHeaderFromBuffer | likely | Cdecl with buffer, length and output arguments. Builds a stack reader (start, length, cursor, end, error byte cleared), calls 010a3080 with the output as ECX a… |
| `010a3580` | ReadI16CountedBytesFromBuffer | likely | Cdecl with buffer, length and output arguments. Builds a stack reader over the buffer, calls 010a3300 and returns the reader's error byte in AL. |
| `010a35c0` | ReadI16BoundedBlobFromBuffer | likely | Cdecl with buffer, length and output arguments. Builds a stack reader over the buffer, calls 010a3380 and returns the reader's error byte in AL. |
| `010a3600` | ReadI16CountedBytesThenI32FromBuffer | likely | Cdecl with buffer, length and output arguments. Builds a stack reader over the buffer, calls 010a3410 and returns the reader's error byte in AL. |
| `010a3940` | WriteU16HeaderThenLenPrefixed | likely | Thiscall: the object is in ECX and the writer is the stack argument. Writes its 16-bit message value at +0x4 into the writer's buffer (bounded by the writer's … |
| `010a4010` | ResetRecordArrayKeysMinusOne | likely | Thiscall with count in the stack argument. Frees the old buffer at +0x0, allocates count*8 bytes through the allocator thunk 00d81d91, stores the buffer, sets … |
| `010a44f0` | RefCachedEntrySet | likely | Thiscall with an entry index, a sorted key and a new refcounted object (plus a flag). Looks up the entry's current key through 010a4070 in the sorted index at … |
| `010a4580` | RefCachedEntryAssign | likely | Thiscall with a count, an object and an index. When the index is within range (0 <= index < this+0x10) it calls 010a44f0 with the entry's key and object from t… |
| `010a4850` | RefCachedEntryBulkSet | likely | Thiscall with a source table. For each entry of the source (12-byte entries, count at +0x10), finds its key through 010a4070 in the sorted index at this+0x7c; … |

### `asset.loader`

**AssetManagement::Asset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071bb80` | AssetConstructor | confirmed | Constructs RefCountedImplicit base, installs Asset vtable, initializes embedded IString to empty, and clears asset fields. |
| `0071bc20` | AssetDestructor | confirmed | Restores IString and RefCounted base vtables, releases owned string storage and resets embedded reference-count fields. |
| `0071bcb0` | AssetScalarDeletingDtor | confirmed | Calls Asset destructor and frees object when deleting flag bit is set. |
| `0071c0c0` | Asset::SharedVirtualMethod2 | confirmed | Creates temporary IString, calls 006f9e60 for temporary and owner, releases any owned buffer, and returns. |

**AssetManagement::AssetRequestHandler** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00927e80` | AssetRequestHandler::Ctor | confirmed | Initializes request/delivery vtables, embedded ready handler, two 32 KiB buffers, synchronization fields, and the requester pointer. |
| `00927f90` | AssetRequestHandler::CallVirtual1cAsBool | confirmed | Calls handler virtual slot 0x1c with the argument and returns whether its result is nonzero. |
| `00927fb0` | AssetRequestHandler::SetStateByte | confirmed | Stores the input byte at object offset 0x0c. |
| `00927fc0` | AssetRequestHandler::GetStateByte | confirmed | Returns the byte stored at object offset 0x0c. |
| `00927fd0` | AssetRequestHandler::ScalarDeletingDtor | confirmed | Runs helper 01098940 and frees the handler only when the deleting flag is set. |
| `00927ff0` | AssetRequestHandler::DispatchByNonzeroValue | confirmed | For nonzero mode calls 010957d0; for zero calls 01095850 with the first argument. |
| `00928030` | AssetRequestHandler::ScalarDeletingDtor | confirmed | Applies a four-byte this adjustment and jumps to the primary deleting-destructor implementation. |
| `00928040` | AssetRequestHandler::GetBufferField359c | confirmed | Returns the 32-bit field stored at object offset 0x359c. |
| `00928050` | AssetRequestHandler::GetBufferFieldb5a4 | confirmed | Returns the 32-bit field stored at object offset 0xb5a4. |

**AssetManagement::AssetRequestInterface::AssetRequester** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0071b5b0` | AssetRequesterScalarDeletingDtor | confirmed | Calls AssetRequester cleanup body and frees object when deleting flag bit is set. |
| `0071af10` | AssetRequester::Destructor | likely | Installs AssetRequester vtable and calls member virtual slot +0x18 with this and zero when member exists. |

**AssetManagement::AssetRequestInternal::AssetReadyHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00927e10` | AssetReadyHandler::Ctor | confirmed | Installs its vtable, clears a field, initializes a deliverable-asset list, and calls helper initialization. |

**ClientStartingZoneLoader** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ce0d0` | ClientStartingZoneLoader::Ctor | confirmed | Stores the supplied pointer, installs ClientStartingZoneLoader and FileInfo-list vtables, clears fields, and initializes two helper objects. |
| `009ce180` | ClientStartingZoneLoader::Dtor | confirmed | Restores helper objects, drains the owned list at this+0x10, cleans both helper objects, then restores and drains the FileInfo list. |

**DeferredPlayListLoader** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009461a0` | DeferredPlayListLoader::DeferredPlayListLoader | confirmed | Constructor: calls the SoundDefinitionAssetLoader base constructor 008a2530 on this with its stack argument, installs DeferredPlayListLoader::vftable at +0 and… |

**DelayedAssetLoader** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae8100` | DelayedAssetLoader::OnFileCreateJobComplete | confirmed | Finds the tracked request by job ID, logs a failed save when the job flag is false, notifies the status handler with success or failure, parses a successful li… |
| `00ae8360` | DelayedAssetLoader::Ctor | confirmed | Installs the asset-delivery and job-queue base vtables, initializes flags and counters, constructs the pending deque, pool, tracked-asset list, and hash table,… |
| `00ae8450` | DelayedAssetLoader::AdjustThisMinus8ThenScalarDelete | confirmed | Subtracts 8 from ECX and jumps to the scalar deleting destructor at 00ae85b0. |
| `00ae8460` | DelayedAssetLoader::Destroy | confirmed | Restores the loader vtables, resets active state, frees the auxiliary buffer, unregisters the callback, clears the hash table and string, destroys the pending … |
| `00ae85b0` | DelayedAssetLoader::ScalarDeletingDtor | confirmed | Calls the DelayedAssetLoader destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |

**DelayedAssetLoader::FileCreateJob** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae51e0` | DelayedAssetLoader::FileCreateJob::Destroy | confirmed | Rewrites the job vtable, calls the job cleanup helper and base cleanup, then restores the Job and RefCounted base vtables. |
| `00ae56f0` | DelayedAssetLoader::FileCreateJob::Ctor | confirmed | Initializes the job base, installs the FileCreateJob vtable, constructs a StringFixed<char,512> member, stores two constructor values, clears the trailing byte… |
| `00ae5790` | DelayedAssetLoader::FileCreateJob::ScalarDeletingDtor | confirmed | Calls the FileCreateJob destructor body, conditionally frees the object when flag bit 0 is set, and returns the original pointer otherwise. |

**FlashLoaderJob** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00840b80` | FlashLoaderJobScalarDeletingDtor | confirmed | thiscall scalar deleting destructor (30 bytes): calls 00840ad0 (FlashLoaderJobDtorBody), then calls _free(this) when bit 0 of arg1 is set. Bytes after the _fre… |
| `00840ab0` | FlashLoaderJobCreateMovie | likely | Virtual slot 2 of FlashLoaderJob (26 bytes). Calls GFxWrap::CreateMovie with the movie name at this+0x114 and stores the returned movie at this+0x120. |
| `00840ad0` | FlashLoaderJobDtorBody | likely | __fastcall destructor body (no free): stores the FlashLoaderJob vtable, calls GFxWrap::ReleaseMovie on this+0x120 when it is non-null, releases the IString<cha… |

**GameRuntimeTerrain::Chunk** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d4c330` | LogUnavailableChunkAsset | confirmed | Logs the result code and asset name with the FreeRealms.log format, then sets the chunk status byte at +0x38 to 1. |
| `00d4eb30` | ChunkBuildAssetPaths | confirmed | When the alternate-format flag is set, formats a .gck2 filename and passes it with this object to provider vtable slot +0x0C; always formats and passes a .gcnk… |
| `00d4ec80` | HandleChunkAssetResult | confirmed | Requires three stored state fields, calls the chunk parser, and when parsing returns false invokes Chunk vtable slot +0x10 with status 3 and the supplied resul… |

**GameRuntimeTerrain::DecompressChunkDataJob** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d4c2c0` | PrepareChunkDecodeBuffers | confirmed | Allocates buffers for the stored input and output sizes, passes each buffer and its lengths to 010ebdc0, and clears the job success flag when that call returns… |
| `00d4d720` | DecompressChunkDataJobCtor | confirmed | Initializes the job base and its input fields, stores size and owner references, installs the job vtable, and increments the referenced object's count. |
| `00d4d7e0` | DecompressChunkDataJobDtor | confirmed | Decrements its referenced owner count, releases the owner at zero, frees one buffer, frees another allocation with alignment 0x10, then restores base vtables. |
| `00d4d8a0` | DecompressChunkDataJobScalarDeletingDtor | confirmed | Runs job cleanup, frees the job object when flag bit 0 is set, and otherwise returns the object pointer. |

**ItemClassDefinitionManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01048570` | ItemClassDefinitionManager::LoadDefinitions | confirmed | Thiscall, one stack arg (path or source IString). Releases the old table through 008dbb50, assigns the source string and appends the literal "ItemClasses.txt" … |

**LoadingScreenData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a5c5d0` | LoadingScreenDataCtor | confirmed | Initializes the delivery-handler base, installs LoadingScreenData vtables, sets default flags, initializes a string, and stores a field from a global manager. |
| `00a5c680` | LoadingScreenDataDtorAdjustorThunk | confirmed | Subtracts 8 from ECX, then jumps to 00a5c730; bytes are 83 e9 08 followed by a relative jump to that target. |
| `00a5c730` | LoadingScreenData::ScalarDeletingDtor | confirmed | Calls the LoadingScreenData cleanup body, frees the object when flag bit 0 is set, otherwise returns its pointer. |
| `00a5c690` | LoadingScreenDataBaseDtor | likely | Releases its reference-counted string when retained, runs another member cleanup, then installs the RefCounted base vtable. |

**SCEA::BinXML::ReaderInput** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007df440` | SCEA::BinXML::ReaderInput::DtorBody | confirmed | fastcall (this=ECX), no args, 7 bytes: stores SCEA::BinXML::ReaderInput::vftable at [this] and returns. Frees nothing. |
| `007df470` | SCEA::BinXML::ReaderInput::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (31 bytes). Stores ReaderInput::vftable at [this], then if flag bit 0 is set calls _free on this (through the 00d824… |
| `007df560` | SCEA::BinXML::ReaderInput::DtorBody_SEH | confirmed | fastcall (this=ECX), no args, SEH-framed (fs:[0] saved, cookie xor), stores SCEA::BinXML::ReaderInput::vftable at [this] and returns. Frees nothing. Same effec… |

**SCEA::BinXML::ReaderInput_Memory** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007df490` | SCEA::BinXML::ReaderInput_Memory::ReaderInput_Memory | confirmed | thiscall with two stack args (param_2 and param_3). Stores ReaderInput::vftable, then this+4 = param_3; then ReaderInput_Memory::vftable at [this], and this+8 … |
| `007e0420` | SCEA::BinXML::ReaderInput_Memory::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (91 bytes). Stores SCEA::BinXML::ReaderInput::vftable at [this] (the base vtable, not the Memory one), then if flag … |
| `007df500` | SCEA::BinXML::ReaderInput_Memory::VSlot1_ReadBytes | likely | thiscall, two stack args (destination pointer and byte count), ret 8; copies count bytes with memcpy from the cursor at this+8 into the destination, advances t… |
| `007df530` | SCEA::BinXML::ReaderInput_Memory::VSlot2_SkipBytes | likely | thiscall, one stack arg (count), returns 1. Adds count to the cursor at this+8 without copying (12 bytes). |

**SCEA::BinXML::Streamable** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007df690` | SCEA::BinXML::Streamable::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (30 bytes). Calls 00f9e6e0 with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |

**SCEA::BinXML::String** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007df6b0` | SCEA::BinXML::String::String | confirmed | fastcall (this=ECX), no args. Stores Streamable::vftable at [this], then clears the byte at +4 and the byte at +5, zeroes the dwords at +8, +0xc and +0x10, the… |
| `007e0480` | SCEA::BinXML::String::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (90 bytes). Calls 00f9e6e0 (the Streamable base destructor body) with this under an SEH frame, then if flag bit 0 is… |

**SoeUtil::List<ClientStartingZoneLoader::FileInfo,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009cd8e0` | FileInfoList::AllocateNode | confirmed | Obtains a block from the current or global allocator, falling back to an aligned 0x30-byte allocation. |
| `009cd920` | FileInfoList::FreeNode | confirmed | Frees the supplied node through the memory manager with alignment 4. |

**Streaming::SOEAssetLoaderHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123bbc0` | Streaming::SOEAssetLoaderHandler::Ctor | confirmed | Stores AssetManagement::AssetDeliveryHandlerInterface::vftable into the first word, stores the argument at +4, then overwrites the first word with Streaming::S… |
| `0123bd80` | Streaming::SOEAssetLoaderHandler::ScalarDeletingDtor | confirmed | Calls FUN_010935e0 (base teardown) with the stack-cookie value, then frees the object through _free when bit 0 of the flag argument is set; returns the object … |

**Streaming::StreamingRequestStreamedAssets** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123bde0` | Streaming::StreamingRequestStreamedAssets::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable and then Streaming::StreamingRequestStreamedAssets::vftable into the first word of the object, and when the … |
| `0123be80` | Streaming::StreamingRequestStreamedAssets::ScalarDeletingDt… | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object, then frees it through _free when bit 0 of the flag argument is set. |
| `0123be50` | Streaming::StreamingRequestStreamedAssets::GetActionName | likely | Virtual slot [1] of Streaming::StreamingRequestStreamedAssets::vftable: returns the address of the literal RequestStreamedAssets. |
| `0123be70` | Streaming::StreamingRequestStreamedAssets::GetTraceLabel | likely | Virtual slot [4] of Streaming::StreamingRequestStreamedAssets::vftable: returns the address of its diagnostic literal. |

**Streaming::StreamingService** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9ac50` | StreamingService::ScalarDeletingDtor | confirmed | Virtual slot 0 of Streaming::StreamingService: stores the vtable at +0, calls 01233c80 (StreamingServiceBase destructor body), then frees this through _free wh… |

**Streaming::StreamingServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01233c80` | Streaming::StreamingServiceBase::~StreamingServiceBase | confirmed | Destructor body: stores the StreamingServiceBase vtable, calls the vtable slot 0 of the object at this+8 with flag 1 when that pointer is non-null, calls 01233… |
| `01233e20` | Streaming::StreamingServiceBase::StreamingServiceBase | confirmed | Constructor body: calls 01230d10 (ServiceBase constructor), stores the StreamingServiceBase vtable at +0, zeroes +8, calls 01233d00 to set up the empty tree he… |
| `01233e80` | Streaming::StreamingServiceBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls 01233c80 (the destructor body), then calls _free on this when bit 0 of the flag argument is set, and returns … |

**VehiclePartProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d25a0` | VehiclePartProcessor::GetOrCreateShipCombatAssetProcessor | likely | If this+0xe10 is null, allocates 0x4f8 bytes, reads the owner value at the object referenced by this+8 offset +0xe8, gets the global manager, calls FUN_00c534f… |

**(no class)** (86)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00928600` | InitAssetDelivery | confirmed | Reads optional AdditionalPaths, selects direct or indirect delivery, polls for a manifest up to 600 ms, logs failures/readiness, and configures request buffers… |
| `009cdcd0` | LoadStartingZoneLoadRecords | confirmed | Opens the named StartingZoneLoad text file, reads lines into a 32 KB buffer, skips comment or short lines, splits fields at delimiters, and appends records to … |
| `00ae5a00` | CreateRequestFromNextPooledRecord | confirmed | Refills the chunked pool if needed, advances its cursor, returns zero for an empty slot, otherwise builds a temporary IString from the supplied text and calls … |
| `00ae6870` | OnDelayedAssetRequestError | confirmed | Logs the failed asset path and code; when the path is the manifest filename, logs the manifest-list failure and resets loader state to 3; otherwise notifies th… |
| `00ae6ab0` | BeginAssetFileListRequest | confirmed | Optionally allocates tracking state, sets loader status to 1, calls the delivery interface to request the configured file-list string, and on a nonzero result … |
| `00ae6b90` | PumpDelayedAssetRequests | confirmed | When active state is 2, reports completion if no work remains; otherwise submits queued asset paths while below the active limit, allocates an ID per request, … |
| `00ae7ae0` | ParseDeferredAssetsManifest | confirmed | Splits the manifest into lines, skips blank and comment entries, strips a leading plus marker, hashes the final path component, filters duplicates and unwanted… |
| `00ae7f00` | OnAssetDataOrManifestReceived | confirmed | If the response path is the configured manifest, logs empty or received status and parses a nonempty body; otherwise matches the response to a tracked request,… |
| `00caa1a0` | DateWindowCheck | confirmed | Reads the StartDate and EndDate attributes (00d517f0, copied into IStrings with 00402ce0); when either is non-empty it obtains a date value with 00702670, pars… |
| `00caa9e0` | LoadingScreenEntry_IsEligible | confirmed | Decides whether a loading-screen entry element is shown: reads ShowToMembers and ShowToNonMembers flags, compares them with the byte at this+0x2a, tests Countr… |
| `00caae20` | LoadingScreenXml_Load | confirmed | Loads LoadingScreen.xml (00fbf4a0): reads each LoadingScreens/LoadingScreen element through the filter 00caa9e0, builds LoadingScreenData records (Filename, We… |
| `00ce2320` | MiniGamePreloadLoad | confirmed | Thiscall with one pointer argument: builds an IString path by assigning CResources/ and appending the argument (FUN_00402ce0, FUN_00402770); when the flag byte… |
| `00d4a9e0` | ReadTerrainTileRecord | confirmed | Consumes fixed fields from the input cursor, grows per-tile arrays, fills flora and object records, and advances the cursor through variable-length sections. |
| `00d4e3a0` | CopyStringRecordFields | confirmed | Initializes two embedded IString objects from the source record and copies the following scalar fields into the destination record. |
| `00d4e470` | CopyStringRecordFields | confirmed | Initializes embedded IString objects from the source record and copies the remaining scalar fields into the destination layout. |
| `00d4e560` | CopyStringRecordFields | confirmed | Initializes the first embedded IString object from the source, then copies the remaining scalar fields into the destination record. |
| `00d4e610` | ConfigureRuntimeTerrainAssets | confirmed | Stores supplied runtime terrain fields, checks for RuntimeTerrainFallback and RuntimeTerrain1 assets, sets availability flags, and returns the virtual readines… |
| `00d4e6b0` | ParseChunkHeaderAndCreateDecompressionJob | confirmed | Reads the chunk magic and version, accepts GCNK versions 1 through 6 and positive GCK2 versions, creates a typed decompression job, and marks failure for unsup… |
| `00d4e9a0` | InstallFallbackTerrainResource | confirmed | When a secondary job yields a buffer, formats fallback_%s_%d_%d, calls a resource vtable method with the buffer and stored fields, sets a state flag, and frees… |
| `00d4ead0` | FormatGcnkFilename | confirmed | Formats %s_%d_%d.gcnk using the stored area name and two integer coordinates, writing into the supplied destination. |
| `01097160` | AssetLoadFailureDispatch | confirmed | Thiscall (ECX=loader; 1024-bucket table at +0x1c, count-less), two stack args (request object pointer, asset name string). When the request is non-null and its… |
| `01097c10` | AssetDumpToFile | confirmed | Thiscall (ECX=owner), one stack arg (optional name filter string, may be null or empty), no return value. Walks the 8192-bucket table at owner+0x35a0 (count at… |
| `007e04e0` | DirectionNameToVector2 | likely | thiscall (ECX = out pair), one stack arg (a C string pointer). Null string: writes (0,0). Otherwise compares the string with "front" (006fc5b0 returns 0 on mat… |
| `007e2500` | TextureAliasesRead | likely | Thiscall (ECX = owner cell, one stack arg = reader). Resets a local success flag to 1 and loops over tag-1 alias records: reads a String pair, assigns the alia… |
| `007e2940` | TintAliasesRead | likely | Thiscall (ECX = owner, one stack arg = reader). Clears the owner's string at +0x2cc and counters at +0x2d0 and +0x2d4, then loops over tag-1 tint alias records… |
| `007e6c10` | BinXmlReadElementDispatch | likely | Thiscall (ECX = owner), one stack arg (the BinXML reader). Reads a count from the reader through its vtable +0xc, then loops: 00f9f4a0 returns a pointer to the… |
| `009cdbc0` | AdvanceQueueUnlessState1Or2 | likely | Calls the queue selector only when field +0x1c is neither 1 nor 2; otherwise returns without dispatch. |
| `009cdbe0` | AppendStringPairNodeAtTail | likely | Allocates and initializes a new string-pair node, links its previous pointer to the old tail, sets its next pointer null, updates head/tail, and increments cou… |
| `009cdee0` | UnlinkAndDestroyListNode | likely | Repairs list head, tail and neighbor links, decrements count, destroys the node's two strings, then calls the list deallocator. |
| `00ae4fa0` | NextChunkedRequestRecord | likely | Computes how many 0x100-sized request slots to refill, calls the refill helper, advances the cursor, and returns the selected 0x22c-byte record. |
| `00ae5560` | PopChunkedTreeRecord | likely | Adds a block when the cursor is empty, returns the current free-list record, decrements the pool count, and advances the free-list head; when enabled, it updat… |
| `00ae5bc0` | InitializePendingRequestStorage | likely | Calls the pending-request deque block initializer and then calls a second helper with the immediate size 4000. |
| `00ae5c20` | PopPendingDequeRecord | likely | Calls a deque helper, removes and returns the next pooled record, and calls a second helper unless the object equals sentinel value -0x28020. |
| `00ae6250` | DtorWrapperFor51E0 | likely | Creates an exception frame and invokes the FileCreateJob destructor body on the supplied object. |
| `00ae62a0` | ReturnRecordToDequePool | likely | Runs a deque lock helper, returns the supplied record through 00ae61e0, then runs the unlock helper unless the object equals sentinel -0x28020. |
| `00ae6380` | ConsumePendingDequeTail | likely | Destroys the requested number of entries at the deque tail, reduces the pending count, resets the start index when empty, and when enabled frees whole 0x100-en… |
| `00ae6460` | InitializePendingRequestDeque | likely | Creates an exception frame, invokes the pending-request storage initializer, and returns the original argument. |
| `00ae6540` | ConsumePendingDequeHead | likely | Destroys the requested number of entries at the deque head, advances the start index, reduces the pending count, and when enabled frees whole 0x100-entry block… |
| `00ae65e0` | AppendTrackedHashListNode | likely | Allocates an entry through the supplied vtable, initializes its string/key fields, appends it to the doubly linked list, links it into the low-byte bucket, inc… |
| `00ae6690` | AllocateFileCreateJobNode | likely | Obtains a deque record through 00ae5c20, returns zero if none is available, otherwise constructs a FileCreateJob pool node from the supplied arguments. |
| `00ae6720` | ResetDelayedAssetLoaderState | likely | Removes tracked entries, frees the auxiliary buffer, releases the retained string buffer, resets status and counters, stores the supplied state, and notifies t… |
| `00ae67e0` | AllocateTrackedAssetRequestId | likely | Increments the request counter with a minimum of one, scans the selected low-byte bucket for the resulting ID, creates and inserts a tracked node when absent, … |
| `00ae6940` | InitializeAndReleaseJobNode | likely | Obtains a node through 00ae6690, passes it to helper 00fc6fb0, decrements the node reference count, and calls its vtable slot +4 if the count reaches zero. |
| `00ae6d00` | PumpUntilElapsedThreshold | likely | While loader state is nonzero, compares the elapsed time pair with the supplied threshold; if the threshold has not elapsed it pumps requests and sleeps for 25… |
| `00ae6d80` | PumpUntilLoaderIdle | likely | While loader state at +0x10 is nonzero, pumps delayed requests and sleeps for 25 milliseconds before checking the state again. |
| `00ae7900` | TrackedAssetMapDtorWrapper | likely | Creates an exception frame and invokes the shared tracked-asset map destructor. |
| `00ae7950` | BeginDeferredLoadMode3 | likely | When idle, stores the limit argument, assigns the supplied file-list string, clears the byte flag, sets state 3, and starts the file-list request; otherwise no… |
| `00ae79b0` | BeginDeferredLoadMode1 | likely | When idle, stores the limit argument, assigns the supplied asset-list string, clears the byte flag, sets state 1, and starts the file-list request; otherwise n… |
| `00ae7a10` | BeginDeferredLoadMode2 | likely | When idle, assigns the supplied list string, stores the byte option, sets state 2, and starts the file-list request; otherwise notifies the callback with statu… |
| `00ae7a60` | ShutdownDelayedAssetLoader | likely | Optionally waits for a positive timeout while requests remain active, resets active state to 5, frees the auxiliary buffer, unregisters its callback through vt… |
| `00d4b390` | PrepareTileResourceSlots | likely | Selects resource identifiers by the current mode, updates two resource groups through helper calls, binds per-index entries, and marks the groups initialized. |
| `00d4bc20` | DownsamplePackedPixels32 | likely | Averages neighboring packed values into the next output level, using 32-bit channel masks when param_6 is 4 and alternate packed branches for other values. |
| `00d4be60` | DownsamplePackedPixels16 | likely | Averages neighboring packed values into the next output level, using 16-bit channel masks and branches on param_6. |
| `00d4c0c0` | CopyCountedWordsAndAdvanceSource | likely | Copies param_1 times 4 bytes from the source pointer to the EAX destination and returns the source pointer advanced by that byte count. |
| `00d4c360` | ConvertTerrainTextureMipLevels | likely | Creates a destination surface, converts or copies the first level according to stored pixel size and format, then builds lower levels with the packed-pixel fil… |
| `00d4c750` | ConvertTerrainTextureMipLevelsPacked | likely | Creates a destination surface, expands packed source pixels into the first level, and builds lower levels using the texture mipmap helper. |
| `00d4cc20` | ConvertTerrainTextureMipLevelsExpanded | likely | Creates a destination surface, copies or expands rows according to source pixel size, and generates lower levels with the packed-pixel filter. |
| `00d4d020` | ConvertTerrainTextureMipLevels16 | likely | Creates a destination surface, converts or copies packed input pixels across rows, then generates lower levels with the 16-bit packed-pixel filter. |
| `00d4edc0` | CopyStringRecordFields | likely | Initializes an embedded string from source data and copies the trailing scalar fields to the destination record. |
| `00d4ee10` | CopyStringRecordFields | likely | Initializes an embedded string from source data and copies trailing scalar fields to the destination record. |
| `00f9ce10` | GetOrCreateStreamingService | likely | Fastcall on a registry object: looks up the global key DAT_01cbd8f0 through 00f9bf20; when absent it allocates 0x18 bytes, constructs the Streaming::StreamingS… |
| `01063030` | TextTableLoadIntoBuckets | likely | Fastcall with the loader in ECX; no stack arguments, cookie-framed. Reads a text file through 00fbf4a0 (its first argument is the dword at this+0x74, the secon… |
| `01063d10` | TextRowsLoadIntoOrderedHash | likely | Fastcall with the loader in ECX (cookie-framed, no stack arguments). Reads a text file through 00fbf4a0 (its first argument is the dword at this+0x64, flag 1),… |
| `01088860` | ReadTextFieldAndParseInt | likely | Sets buf[0]=0 on a 2048-byte stack buffer, calls 006fba40(param_1, buf, 0x800, param_3, 1), parses buf into *param_2 through 01088750, and returns 006fba40's r… |
| `010889b0` | ParseTextRecordFields | likely | Thiscall (ECX = record): calls 0072d900(src, this+0), 01088860(src+n, this+4), 0072d900(this+8), 0072d900(this+0xc), 00793e30(this+0x10), 0072d900(this+0x20) a… |
| `01088a60` | ReadTextRecordD | likely | Thiscall (ECX = record): calls 0072d900 four times, reading into this+0, this+4, this+8 and this+0xc in order and advancing the source pointer by each returned… |
| `01088ab0` | ReadTextRecordB | likely | Thiscall (ECX = record): 0072d900 into this+0; 006fc080(src, this+8, ctx, 1); 0072d900 into this+4; 0072d4e0 into this+0x18; 0072d4e0 into four locals; 0072d90… |
| `01088d10` | ReadTextRecordC | likely | Thiscall (ECX = record): 0072d900 into this+0; 0072d900 into this+4; 006fc080(src, this+8, ctx, 1); 0072d4e0 into this+0x18; 0072d900 into this+0x1c, +0x20 and… |
| `01088fb0` | ReadTextFieldAndParseIntB | likely | Sets buf[0]=0 on a 2048-byte stack buffer, calls 006fba40(param_1, buf, 0x800, param_3, 1), parses buf into *param_2 through 01088ea0 (same parser as 01088750)… |
| `01089060` | ReadTextRecordE | likely | Thiscall (ECX = record, size 0x3c): reads 15 fields in order through 0072d900 (this+0, +4, +0x28, +0x1c, +0x30, +0xc, +0x14, +0x18, +0x38), 01058eb0 (this+0x20… |
| `01089230` | ReadTextRecordF | likely | Thiscall (ECX = record, size 0x34): reads fields through 0072d900 (this+0, +4, +0x20, +8, +0x28, +0x2c), 01058eb0 (this+0x1c), 006fc080 (this+0xc, trailing fla… |
| `0108a1b0` | ReadTextRecordG | likely | Thiscall (ECX = record, src, ctx): reads through 0072d900 into this+0, this+4, this+8, this+0xc, a stack local (param_3), a stack local (param_2), this+0x1c, t… |
| `0108a250` | ReadTextRecordH | likely | Thiscall (ECX = record, src, ctx): reads through 0072d900 into this+0, this+4, this+8, a stack local (param_3), this+0x1c, this+0x20 and a stack local (param_2… |
| `0108a450` | ReadTextRecordI | likely | Thiscall (ECX = record, src, ctx): reads fields through 0072d900 (this+0, +4, +8, +0xc, +0x10, +0x14, +0x80), 00793e30 (this+0x18, +0x28, +0x38), 0072d550 (thi… |
| `0108a650` | CaretListFloatOrFallback | likely | Thiscall with two dword stack arguments, ret 8: if the flag byte at ECX is nonzero, the pointer at ECX+0x24 is nonzero and param_2 (as a signed int) is not neg… |
| `0108a710` | CaretListIntOrFallback | likely | Thiscall with two int stack arguments: if the flag byte at ECX is nonzero, the pointer at ECX+0x24 is nonzero and param_2 is not negative, it calls 006fbde0 on… |
| `0108b360` | ReadTextFieldAndParseIntC | likely | Sets buf[0]=0 on a 2048-byte stack buffer, calls 006fba40(param_1, buf, 0x800, param_3, 1), parses buf into *param_2 through 0108b120 (same parser as ParseDeci… |
| `0108b440` | ReadTextRecordJ | likely | Thiscall (ECX = record, src, ctx): reads this+0xc through 0072d900, this+0x10 through 0108b360 (a text field reader returning its byte count), this+0x14 and th… |
| `0108c530` | ReadTextRecordK | likely | Thiscall (ECX = record, src, ctx): reads through 0072d900 into this+0x10 and this+0xc, then into three stack locals; when this+0xc is nonzero stores locals 1 a… |
| `010b5850` | CheckDefinitionListReport | likely | Walks the list of definitions whose head is this+0x34 (next at +0xc0): for each node calls its vtable slot 1 (+4) with no arguments and, when that returns fals… |
| `010f71c0` | PackFileWriteAt | likely | Thiscall(this=param_1, offset=param_2, buffer=param_3, size=param_4). Calls 00702f50 to check the file is open; if not it logs 'PackFile Error: pack file could… |
| `0123bd00` | CreateStaticLoaderHandler | likely | Allocates 8 bytes through operator new (FUN_00d833a6(8)); when the allocation is null it stores 0 into the global at DAT_01cbcc80, otherwise it stores the resu… |
| `012f4750` | ResolveRootedPath | likely | Writes into the std::string at param_1 (zeroed to empty SSO first) the copy of the source string param_3 with backslashes replaced by slashes (012f3cc0). If th… |
| `012f4910` | ResolveRootedPathEmptyPrefix | likely | Builds an empty local std::string (SSO, capacity 0xf) as the prefix, calls 012f4750 (ResolveRootedPath) with (param_1 output, empty prefix, param_2 source), fr… |
| `012f5560` | LoadFileContents | likely | Reads a whole file named by the std::string argument into a new heap buffer: zeroes the two out parameters, builds an ifstream through 012f5410 (name, mode 0x2… |
| `012f56c0` | LoadFileAsString | likely | Loads the file named by the std::string argument param_1 into the out std::string param_2: opens it through 012f5410 (name, mode 0x25, protection 0x40, most-de… |

### `asset.mannequin`

**MannequinManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b87d70` | MannequinManager::MannequinManager | confirmed | Constructor: reads the service pointer from FUN_00769710 (+0x18) into +4, installs AssetManagement::AssetDeliveryHandlerInterface::vftable then MannequinManage… |
| `00b881c0` | MannequinManager::~MannequinManager | confirmed | Destructor body: reinstalls MannequinManager::vftable, destroys the stored entry via 00b88140 and then runs the base teardown FUN_010935e0 (asset-delivery-hand… |
| `00b885a0` | MannequinManager::ScalarDeletingDtor | confirmed | Vtable slot 0 of MannequinManager: runs the destructor body 00b881c0 and frees the manager object through _free when bit 0 of param_2 is set. |
| `00b87e00` | MannequinManager::FindEntry | likely | Hash lookup: starts from bucket (param_2 & 0xf) of the bucket table at this+0x1c, walks the chain via the next pointer at node+0x28 and returns the first node … |
| `00b87e30` | MannequinManager::RemoveEntry | likely | Unlinks an entry from both containers: finds the node in the bucket chain at this+0x10 + (key&0xf)*4 by identity, splices it out (piVar1 = next), zeroes the no… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b885c0` | LoadMannequinsTextFile | confirmed | Formats the path 'Mannequins.txt' into a SoeUtil::IString through FUN_00402fd0 (variadic IStringFormat), reads the file text via 00b88220, releases the tempora… |
| `00b87fd0` | MannequinEntry_Init | likely | Initializes a 0x38-byte mannequin record: zeroes +0, +0x10/+0x14 (and +0x20/+0x24), and installs two SoeUtil::IString<char> members, one at +8/+0xc (vtable 017… |
| `00b88050` | MannequinEntry_Destroy | likely | Destructor body of the mannequin record: reinstalls the SoeUtil::IString<char> vtable at this+0x18, and for each of the two refcounted string data pointers (th… |
| `00b88100` | MannequinEntry_ScalarDeletingDtor | likely | Scalar deleting destructor body: runs the record destructor 00b88050 and then frees the record through _free when bit 0 of param_2 is set. |
| `00b88220` | MannequinsTextRead | likely | Reads a text asset through the asset delivery handler: builds an IString copy of the passed path (SoeUtil::IStringAssign), fetches the service object (FUN_0076… |

### `asset.sensory_effect`

**ActorCameraEffectDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00927110` | ActorCameraEffectDefinition::Ctor | confirmed | Initializes requester state and pointer, copies defaults, installs definition vtables, and clears a byte field. |
| `010b9070` | ActorCameraEffectDefinition::ReadAttributes | confirmed | Virtual [7] of ActorCameraEffectDefinition, thiscall with ECX=this and one stack argument (the element). Calls 007ca120 on the element first. Reads intensity i… |
| `010b92a0` | ActorCameraEffectDefinition::WriteAttributes | confirmed | Virtual [8] of ActorCameraEffectDefinition, thiscall with ECX=this and one stack argument (the element). Returns 0 when the gate 007c9dd0 fails. Otherwise writ… |
| `00927250` | ActorCameraEffectDefinition::InitWithData | likely | Constructs the definition from supplied values and stores its key at offset 200. |

**ActorCameraEffectDefinitionManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00926ee0` | ActorCameraEffectDefinitionManager::Ctor | confirmed | Initializes base manager storage and installs the derived manager vtable in both subobjects. |
| `00926f90` | ActorCameraEffectDefinitionManager::ScalarDeletingDtor | confirmed | Adjusts this by four bytes and jumps to its secondary-base deleting-destructor path. |
| `00927350` | ActorCameraEffectDefinitionManager::AddDefinition | confirmed | Copies the supplied key and delegates definition insertion to 009272d0. |
| `00927380` | ActorCameraEffectDefinitionManager::Dtor | confirmed | Restores manager RTTI vtables and invokes inherited cleanup routine 009240b0. |
| `009273d0` | ActorCameraEffectDefinitionManager::ScalarDeletingDtor | confirmed | Runs full cleanup and frees the object only when the deleting flag is set. |
| `009272d0` | CreateAndLinkActorCameraEffectDefinition | likely | Allocates a camera-effect definition through a virtual slot, initializes it, then links it into manager list and hash storage. |

**ActorCompositeEffectDefinition** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00941210` | ActorCompositeEffectDefinitionCtor | confirmed | Constructs its base, installs the definition vtable, initializes the embedded entry list, and zeros its count and head/tail fields. |
| `010bfeb0` | ActorCompositeEffectDefinition::ReadXml | confirmed | Thiscall virtual (slot [7]) with this in ECX and one float stack argument, which the body reuses as the eventSlot read buffer. Clears +0x28 and +0x2c, then rea… |
| `009412a0` | ActorCompositeEffectDefinitionInitWithValue | likely | Calls the definition constructor, stores param_2's first word at this+0x48, and returns this. |
| `010bf570` | ActorCompositeEffectDefinition::WriteXml | likely | Fastcall virtual (slot [8]) with this in ECX. Returns whether at least one trigger in the list at +0x34 (next +0x2c) passes its checks (effect id at +0x14 nonz… |
| `010bfc40` | ActorCompositeEffectDefinition::ValidateTriggers | likely | Fastcall virtual (slot [4]) with this in ECX. Walks the trigger list at +0x34 (next +0x2c). For each trigger, switches on its type at +0x10: types 0 to 8 selec… |

**ActorCompositeEffectDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00940fb0` | ActorCompositeEffectManagerCtor | confirmed | Constructs the templated sensory-effect manager base, then installs the concrete manager vtable in both subobjects. |
| `00941070` | ActorCompositeEffectManagerScalarDtor | confirmed | Bytes subtract 4 from ECX and jump to 00941430, an adjustor thunk to shared scalar deletion. |
| `009413e0` | DestroyActorCompositeEffectManager | confirmed | Restores the concrete manager vtables and calls 0093ff40 with ECX=this to destroy the templated manager state, then returns. |
| `00941430` | ActorCompositeEffectManagerScalarDtor | confirmed | Calls 009413e0 to destroy manager state, then frees the object only when the deleting flag is set. |

**ActorEffectDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e0b10` | ActorEffectDefinition::DtorBody | confirmed | fastcall (this=ECX), no args. Stores ActorEffectDefinition::vftable at [this], then the IString<char> vtable at [this+0x28]. When the length dword at +0x34 is … |
| `007e0ba0` | ActorEffectDefinition::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (30 bytes). Calls 007e0b10 (ActorEffectDefinition::DtorBody) with this, then if flag bit 0 is set calls _free on thi… |

**ActorLightEmitterDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00926bf0` | ActorLightEmitterDefinition::Ctor | confirmed | Initializes requester state, stores its pointer, installs definition vtables, copies defaults, and clears a byte field. |
| `010b8980` | ActorLightEmitterDefinition::ReadAttributes | confirmed | Virtual [7] of ActorLightEmitterDefinition, thiscall with ECX=this and one stack argument (the element). Calls 007ca120 on the element first. Reads color (4 fl… |
| `010b8f00` | ActorLightEmitterDefinition::WriteAttributes | confirmed | Virtual [8] of ActorLightEmitterDefinition, thiscall with ECX=this and one stack argument (the element). Returns 0 when the gate 007c9dd0 fails. Otherwise writ… |
| `00926d40` | ActorLightEmitterDefinition::InitWithData | likely | Constructs an ActorLightEmitterDefinition, stores the supplied key at +0xd8, and returns this. |

**ActorLightEmitterDefinitionManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009269c0` | ActorLightEmitterDefinitionManager::Ctor | confirmed | Initializes inherited manager storage and installs the derived vtable in both manager subobjects. |
| `00926a70` | ActorLightEmitterDefinitionManager::ScalarDeletingDtor | confirmed | Applies a four-byte this adjustment and jumps through the secondary-base deleting-destructor path. |
| `00926e40` | ActorLightEmitterDefinitionManager::AddDefinition | confirmed | Copies the supplied key and forwards it with source data to 00926dc0. |
| `00926e70` | ActorLightEmitterDefinitionManager::Dtor | confirmed | Restores manager vtables and invokes inherited cleanup routine 00923bd0. |
| `00926ec0` | ActorLightEmitterDefinitionManager::ScalarDeletingDtor | confirmed | Runs the complete destructor and frees the object only when the deleting flag is set. |
| `00926dc0` | CreateAndLinkActorLightEmitterDefinition | likely | Allocates a definition through a virtual slot, initializes it on success, and links it into list and hash storage. |

**ActorLightsaberTrailDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00927b30` | ActorLightsaberTrailDefinition::Ctor | confirmed | Initializes requester state, installs definition and IString vtables, clears fields, and copies defaults. |
| `010b9a60` | ActorLightsaberTrailDefinition::WriteAttributes | confirmed | Virtual [8] of ActorLightsaberTrailDefinition, thiscall with ECX=this and one stack argument (the element). Returns 0 when the gate 007c9dd0 fails. Otherwise w… |
| `010b9ad0` | ActorLightsaberTrailDefinition::ReadAttributes | confirmed | Virtual [7] of ActorLightsaberTrailDefinition, thiscall with ECX=this and one stack argument (the element). Calls 007ca120 on the element, reads tipBoneName th… |
| `00927c70` | ActorLightsaberTrailDefinition::InitWithData | likely | Constructs the trail definition and writes its key at offset 200. |

**ActorLightsaberTrailDefinitionManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00927900` | ActorLightsaberTrailDefinitionManager::Ctor | confirmed | Initializes inherited manager storage and installs both derived manager vtables. |
| `009279b0` | ActorLightsaberTrailDefinitionManager::ScalarDeletingDtor | confirmed | Adjusts this by four bytes and jumps to the secondary-base deleting-destructor path. |
| `00927d70` | ActorLightsaberTrailDefinitionManager::AddDefinition | confirmed | Copies the key locally and forwards the source data to insertion routine 00927cf0. |
| `00927da0` | ActorLightsaberTrailDefinitionManager::Dtor | confirmed | Restores manager RTTI vtables and calls inherited cleanup routine 00924a70. |
| `00927df0` | ActorLightsaberTrailDefinitionManager::ScalarDeletingDtor | confirmed | Runs complete cleanup and frees the manager only when the deleting flag is set. |
| `00927cf0` | CreateAndLinkActorLightsaberTrailDefinition | likely | Allocates and initializes a trail definition, then links it into manager list and hash storage. |

**ActorMeshEmitterDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b5dd0` | ActorMeshEmitterDefinition::WriteAttributes | confirmed | Virtual [8] of ActorMeshEmitterDefinition. Calls 007c9dd0 on its stack argument and returns 0 when that is false. Otherwise writes float duration from this+0xa… |
| `010b5f30` | ActorMeshEmitterDefinition::ReadAttributes | confirmed | Virtual [7] of ActorMeshEmitterDefinition. Reads float duration into this+0xac, easeInTime into +0xb4 and easeOutTime into +0xb8 through 010b5520. Reads worldO… |

**ActorMeshEmitterDefinitionManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b6110` | ActorMeshEmitterDefinitionManager::Dtor | confirmed | Thiscall-style fastcall with ECX=this. Stores ActorMeshEmitterDefinitionManager::vftable into both vtable words (+0 and +4), then calls 0090f390 to tear down t… |

**ActorModelMaterialDefinitionManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093cc80` | ActorModelMaterialDefinitionManager::Ctor | confirmed | Constructs the sensory-effect manager base, then installs derived manager vtables in both subobjects. |
| `0093cd40` | ActorModelMaterialDefinitionManager::ScalarDeletingDtorThunk | confirmed | Adjusts ECX by four bytes, then jumps to 0093d160; this thunk does not contain the destructor body. |
| `0093d0e0` | ActorModelMaterialDefinitionManager::AddDefinition | confirmed | Copies the supplied identifier into a temporary key and delegates insertion to the definition-node routine. |
| `0093d110` | ActorModelMaterialDefinitionManager::DtorBody | confirmed | Restores derived manager vtables and calls the sensory-effect manager destructor body to release owned maps and loader state. |
| `0093d160` | ActorModelMaterialDefinitionManager::ScalarDeletingDtor | confirmed | Calls the derived manager destructor body and frees the object only when the deleting flag low bit is set. |

**ActorProjectileDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b61e0` | ActorProjectileDefinition::WriteAttributes | confirmed | Virtual [8] of ActorProjectileDefinition. Calls 007c9dd0 on its stack argument and returns 0 when false. Otherwise, if the char pointer at this+0x9c is non-nul… |
| `010b6300` | ActorProjectileDefinition::ReadAttributes | confirmed | Virtual [7] of ActorProjectileDefinition, thiscall with ECX=this and ret 4. Calls 007ca120 on its stack argument, looks up attribute targetBoneName on that arg… |

**ActorProjectileDefinitionManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b6340` | ActorProjectileDefinitionManager::Dtor | confirmed | Fastcall with ECX=this. Stores ActorProjectileDefinitionManager::vftable into both vtable words (+0 and +4), then calls 0090f870 for the manager base teardown … |

**ActorRumbleEffectDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b93d0` | ActorRumbleEffectDefinition::ReadAttributes | confirmed | Virtual [7] of ActorRumbleEffectDefinition, thiscall with ECX=this and one stack argument (the element). Calls 007ca120 on the element first. Reads intensity1 … |
| `010b95f0` | ActorRumbleEffectDefinition::WriteAttributes | confirmed | Virtual [8] of ActorRumbleEffectDefinition, thiscall with ECX=this and one stack argument (the element). Returns 0 when the gate 007c9dd0 fails. Otherwise writ… |
| `00927760` | ActorRumbleEffectDefinition::InitWithData | likely | Constructs the rumble-effect definition and stores its key at offset 200. |

**ActorRumbleEffectDefinitionManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009273f0` | ActorRumbleEffectDefinitionManager::Ctor | confirmed | Initializes inherited manager storage and installs the derived vtable in both subobjects. |
| `009274a0` | ActorRumbleEffectDefinitionManager::ScalarDeletingDtor | confirmed | Subtracts 4 from ECX, then jumps to 009278e0. |
| `00927860` | ActorRumbleEffectDefinitionManager::AddDefinition | confirmed | Copies the input key and delegates to the manager insertion routine 009277e0. |
| `00927890` | ActorRumbleEffectDefinitionManager::Dtor | confirmed | Restores both RTTI vtables and invokes inherited cleanup routine 00924590. |
| `009278e0` | ActorRumbleEffectDefinitionManager::ScalarDeletingDtor | confirmed | Runs the complete destructor and frees the object only when the deleting flag is set. |

**ActorSoundEmitterDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00946220` | ActorSoundEmitterDefinition::ActorSoundEmitterDefinition | confirmed | Constructor: calls base initialiser 00914670 on this, installs ActorSoundEmitterDefinition::vftable at +0, constructs an embedded DeferredPlayListLoader at thi… |
| `00946310` | ActorSoundEmitterDefinition::ConstructWithKey | likely | Runs the ActorSoundEmitterDefinition constructor 00946220 on this, with arguments copied from this function's own stack frame; stores the dword pointed to by i… |

**ActorSoundEmitterDefinitionManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00945f00` | ActorSoundEmitterDefinitionManager::ThunkSubEcx4ToScalarDtor | confirmed | Secondary-vtable thunk (018089e4 slot 0). Bytes 83 e9 04 subtract 4 from ECX, then e9 jumps to 009464a0. It is not a body. |
| `00946450` | ActorSoundEmitterDefinitionManager::~ActorSoundEmitterDefin… | confirmed | Destructor body: stores ActorSoundEmitterDefinitionManager::vftable at +0 and +4, then calls the SensoryEffectDefinitionManager destructor body 00944f30 on thi… |
| `009464a0` | ActorSoundEmitterDefinitionManager::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: calls the body 00946450 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00946420` | ActorSoundEmitterDefinitionManager::AddDefinition | likely | Virtual slot 5 of ActorSoundEmitterDefinitionManager (018089fc slot 5). Copies its stack argument into a local, then calls 00946390 on the member at this+0x2c … |

**SensoryEffectDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e0a40` | SensoryEffectDefinition::DtorBody | confirmed | fastcall (this=ECX), SEH-framed. Stores SensoryEffectDefinition::vftable at [this], then the IString<char> vtable at [this+0xc]. When the length dword at +0x18… |
| `007e0af0` | SensoryEffectDefinition::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (30 bytes). Calls 007e0a40 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463… |
| `009145b0` | Ctor | confirmed | thiscall with one stack argument (stored at [this+4]). Stores SensoryEffectDefinition::vftable at [this]; zero at [this+8]; IString<char> vftable at [this+0xc]… |

**SensoryEffectDefinitionLoader** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b80e0` | SensoryEffectDefinitionLoader::Save | confirmed | Thiscall, ECX=loader, one stack argument (output path). Only when the count at +0x1c is positive and the virtual at slot [4] returns true: creates the SensoryE… |
| `010b82d0` | SensoryEffectDefinitionLoader::Load | confirmed | Thiscall, ECX=loader, one stack argument (document path or stream). Opens the document through 00d549d0, then for each Definitions/EffectDefinition child reads… |

**SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256>** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00923c60` | SensoryEffectDefinitionManager_ActorCamera_Ctor | confirmed | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256> vtabl… |
| `00923d10` | SensoryEffectActorCamera_Slot0Thunk | confirmed | Adjustor thunk: subtracts 4 from ECX and jumps to 00925180. |
| `00923d20` | SensoryEffectActorCamera_CountPositive | confirmed | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00923d30` | SensoryEffectActorCamera_ReturnAt34 | confirmed | Returns the dword at this+0x34. |
| `00923d40` | SensoryEffectActorCamera_ReturnAt3c | confirmed | Returns the dword at this+0x3c. |
| `009240b0` | SensoryEffectDefinitionManager_ActorCamera_Dtor | confirmed | Destructor body: stores the SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440… |
| `00925180` | SensoryEffectActorCamera_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 009240b0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00923e80` | SensoryEffectActorCamera_CastGetAt0xc0 | likely | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorCameraEffectDefinition and 0 (a dynamic-cast helper), then returns t… |

**SensoryEffectDefinitionManager<ActorCompositeEffectDefinition,256>** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093faf0` | SensoryEffectManagerCtor | confirmed | Constructs the sensory-effect loader base, installs both manager vtables, calls 0093f810 with ECX=this+0x2c to initialize the embedded ActorCompositeEffectDefi… |
| `0093fba0` | SensoryEffectManagerScalarDtor | confirmed | Bytes subtract 4 from ECX and jump to 00940a80, adjusting a secondary-base pointer before shared deletion. |
| `0093fbb0` | SensoryEffectManagerHasPositiveCount | confirmed | Returns true exactly when the signed 32-bit field at this+0x3c is greater than zero. |
| `0093fbc0` | SensoryEffectManagerGetField34 | confirmed | Returns the 32-bit value at this+0x34. |
| `0093fbd0` | SensoryEffectManagerGetField3c | confirmed | Returns the 32-bit value at this+0x3c. |
| `0093fd10` | SensoryEffectManagerGetCastField40 | confirmed | Uses RTTI to cast its argument from SensoryEffectDefinition to ActorCompositeEffectDefinition, then returns the cast object dword at +0x40. |
| `0093fed0` | SensoryEffectManagerInsertEntryAndDispatch | confirmed | Calls 00705b00 on the dword at the input object+4, inserts the resulting string entry into the manager map at +0x440 through 0093fe40, stores param_2 at entry+… |
| `0093ff40` | DestroySensoryEffectManager | confirmed | Restores manager vtables, destroys the name-map subobject at +0x440 through 008e28d0, drains the ActorCompositeEffectDefinition map at +0x2c through 0093f7a0, … |
| `00940a80` | SensoryEffectManagerScalarDtor | confirmed | Calls 0093ff40 to destroy owned maps and loader, then frees the object only when the deleting flag is set. |

**SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256>** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00923830` | SensoryEffectLightEmitter_Slot0Thunk | confirmed | Adjustor thunk: subtracts 4 from ECX and jumps to 00925160. |
| `00923840` | SensoryEffectLightEmitter_CountPositive | confirmed | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00923860` | SensoryEffectLightEmitter_ReturnAt3c | confirmed | Returns the dword at this+0x3c. |
| `00923b60` | SensoryEffectLightEmitter_InsertKeyed | confirmed | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00923ad0, stores param_2 at element+0x10, calls 006f9e6… |
| `00923bd0` | SensoryEffectDefinitionManager_ActorLightEmitter_Dtor | confirmed | Destructor body: stores the SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256> vtables at this and this+4, stores the HashMap<int,...NameEntry,256… |
| `00925160` | SensoryEffectLightEmitter_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 00923bd0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `009239a0` | SensoryEffectLightEmitter_CastGetAt0xd0 | likely | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorLightEmitterDefinition and 0 (a dynamic-cast helper), then returns t… |

**SensoryEffectDefinitionManager<ActorLightsaberTrailDefinition,256>** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00924620` | SensoryEffectDefinitionManager_ActorLightsaber_Ctor | confirmed | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<ActorLightsaberTrailDefinition,256> vt… |
| `009246d0` | SensoryEffectActorLightsaber_Slot0Thunk | confirmed | Adjustor thunk: subtracts 4 from ECX and jumps to 009251c0. |
| `009246e0` | SensoryEffectActorLightsaber_CountPositive | confirmed | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `009246f0` | SensoryEffectActorLightsaber_ReturnAt34 | confirmed | Returns the dword at this+0x34. |
| `00924700` | SensoryEffectActorLightsaber_ReturnAt3c | confirmed | Returns the dword at this+0x3c. |
| `00924a00` | SensoryEffectActorLightsaber_InsertKeyed | confirmed | Virtual slot [6]: builds a source record through 00705b00 from the source at param_3+4, inserts it with 00924970, stores param_2 at element+0x10, calls 006f9e6… |
| `00924a70` | SensoryEffectDefinitionManager_ActorLightsaber_Dtor | confirmed | Destructor body: stores the SensoryEffectDefinitionManager<ActorLightsaberTrailDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x… |
| `009251c0` | SensoryEffectActorLightsaber_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 00924a70 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00924840` | SensoryEffectActorLightsaber_CastGetAt0xc0 | likely | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and the lightsaber-trail definition and 0 (a dynamic-cast helper), then retur… |

**SensoryEffectDefinitionManager<ActorMeshEmitterDefinition,256>** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090ef40` | ActorMeshEmitterDefinitionManager::ActorMeshEmitterDefiniti… | confirmed | Constructs the sensory-effect loader base, installs both manager vtables, initializes manager substructures through 009074d0 and 00907550, then returns this. |
| `0090eff0` | ActorMeshEmitterDefinitionManager::ScalarDeletingDtorAdjust… | confirmed | Subtracts four from this and jumps to 009104f0, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090f000` | ActorMeshEmitterDefinitionManager::HasDefinitions | confirmed | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090f010` | ActorMeshEmitterDefinitionManager::GetField34 | confirmed | Returns the dword stored at this+0x34. |
| `0090f020` | ActorMeshEmitterDefinitionManager::GetCount | confirmed | Returns the dword stored at this+0x3c. |
| `0090f160` | ActorMeshEmitterDefinitionManager::GetEmitterField | confirmed | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorMeshEmitterDefinition and returns the derived field at +0xc0. |
| `0090f320` | ActorMeshEmitterDefinitionManager::AddDefinition | confirmed | Copies and hashes the definition name, inserts a hash-map entry, stores param_2 in that entry, frees the temporary object, and calls virtual slot 5 with the na… |
| `0090f390` | ActorMeshEmitterDefinitionManager::~ActorMeshEmitterDefinit… | confirmed | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `009104f0` | ActorMeshEmitterDefinitionManager::ScalarDeletingDtor | confirmed | Calls 0090f390 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |

**SensoryEffectDefinitionManager<ActorModelMaterialDefinition,256>** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093c150` | SensoryEffectDefinitionManager::Ctor | confirmed | Constructs the sensory-effect loader, installs manager vtables, initializes the 256-bucket hash-list map, and initializes remaining manager state. |
| `0093c200` | SensoryEffectDefinitionManager::ScalarDeletingDtorThunk | confirmed | Adjusts ECX by four bytes, then jumps to 0093c630; this thunk does not contain the destructor body. |
| `0093c210` | SensoryEffectDefinitionManager::HasDefinitions | confirmed | Returns whether the signed count at object offset 0x3c is greater than zero. |
| `0093c230` | SensoryEffectDefinitionManager::GetEntryCount | confirmed | Returns the dword stored at object offset 0x3c. |
| `0093c5a0` | SensoryEffectDefinitionManager::Dtor | confirmed | Restores manager and hash-map vtables, releases manager-owned nodes, and destroys the sensory-effect loader subobject. |
| `0093c630` | SensoryEffectDefinitionManager::ScalarDeletingDtor | confirmed | Calls the manager destructor body and frees the object only when the deleting flag low bit is set. |
| `0093c220` | SensoryEffectDefinitionManager::GetManagerField34 | likely | Returns the dword stored at object offset 0x34. |
| `0093c370` | GetNextActorModelMaterialDefinition | likely | RTTI-casts the supplied object to ActorModelMaterialDefinition through SensoryEffectDefinition and returns the dword at cast-result offset 0x60. |

**SensoryEffectDefinitionManager<ActorParticleEmitterDefinition,256>** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090ea70` | ActorParticleEmitterDefinitionManager::ActorParticleEmitter… | confirmed | Constructs the sensory-effect loader base, installs both manager vtables, initializes two manager substructures through 00907150 and 009071d0, then returns thi… |
| `0090eb10` | ActorParticleEmitterDefinitionManager::ScalarDeletingDtorAd… | confirmed | Subtracts four from this and jumps to 009104d0, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090eb20` | ActorParticleEmitterDefinitionManager::HasDefinitions | confirmed | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090eb30` | ActorParticleEmitterDefinitionManager::GetField34 | confirmed | Returns the dword stored at this+0x34. |
| `0090eb40` | ActorParticleEmitterDefinitionManager::GetCount | confirmed | Returns the dword stored at this+0x3c. |
| `0090ec70` | ActorParticleEmitterDefinitionManager::GetEmitterField | confirmed | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorParticleEmitterDefinition and returns the derived field at +0xd0. |
| `0090ee40` | ActorParticleEmitterDefinitionManager::AddDefinition | confirmed | Copies and hashes the definition name, inserts a name-map entry, stores param_2 in the new entry, frees the temporary object, then calls virtual slot 5 with th… |
| `0090eeb0` | ActorParticleEmitterDefinitionManager::~ActorParticleEmitte… | confirmed | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `009104d0` | ActorParticleEmitterDefinitionManager::ScalarDeletingDtor | confirmed | Calls 0090eeb0 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |

**SensoryEffectDefinitionManager<ActorProjectileDefinition,256>** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090f420` | ActorProjectileDefinitionManager::ActorProjectileDefinition… | confirmed | Constructs the sensory-effect loader base, installs both manager vtables, initializes manager substructures through 009076d0 and 00907750, then returns this. |
| `0090f4d0` | ActorProjectileDefinitionManager::ScalarDeletingDtorAdjustor | confirmed | Subtracts four from this and jumps to 00910510, adapting the secondary manager-vtable pointer to its scalar deleting destructor. |
| `0090f4e0` | ActorProjectileDefinitionManager::HasDefinitions | confirmed | Returns whether the signed dword at this+0x3c is greater than zero. |
| `0090f4f0` | ActorProjectileDefinitionManager::GetField34 | confirmed | Returns the dword stored at this+0x34. |
| `0090f500` | ActorProjectileDefinitionManager::GetCount | confirmed | Returns the dword stored at this+0x3c. |
| `0090f640` | ActorProjectileDefinitionManager::GetProjectileField | confirmed | Performs an RTTI-checked cast from SensoryEffectDefinition to ActorProjectileDefinition and returns the derived field at +0xb0. |
| `0090f800` | ActorProjectileDefinitionManager::AddDefinition | confirmed | Copies and hashes the definition name, inserts a hash-map entry, stores param_2 in that entry, frees the temporary object, then calls virtual slot 5 with the n… |
| `0090f870` | ActorProjectileDefinitionManager::~ActorProjectileDefinitio… | confirmed | Restores manager and hash-map vtables, releases manager and hash-map contents through helpers, then destroys the sensory-effect loader base. |
| `00910510` | ActorProjectileDefinitionManager::ScalarDeletingDtor | confirmed | Calls 0090f870 to destroy the manager, frees it when flag bit 0 is set, and returns the object pointer. |

**SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256>** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00924140` | SensoryEffectDefinitionManager_ActorRumble_Ctor | confirmed | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256> vtabl… |
| `009241f0` | SensoryEffectActorRumble_Slot0Thunk | confirmed | Adjustor thunk: subtracts 4 from ECX and jumps to 009251a0. |
| `00924200` | SensoryEffectActorRumble_CountPositive | confirmed | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00924210` | SensoryEffectActorRumble_ReturnAt34 | confirmed | Returns the dword at this+0x34. |
| `00924220` | SensoryEffectActorRumble_ReturnAt3c | confirmed | Returns the dword at this+0x3c. |
| `00924590` | SensoryEffectDefinitionManager_ActorRumble_Dtor | confirmed | Destructor body: stores the SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440… |
| `009251a0` | SensoryEffectActorRumble_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 00924590 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00924360` | SensoryEffectActorRumble_CastGetAt0xc0 | likely | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and ActorRumbleEffectDefinition and 0 (a dynamic-cast helper), then returns t… |

**SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00944b00` | SensoryEffectDefinitionManager::SensoryEffectDefinitionMana… | confirmed | Constructor: calls the SensoryEffectDefinitionLoader base constructor 008cf920 on this with its two stack arguments, installs SensoryEffectDefinitionManager<Ac… |
| `00944ba0` | SensoryEffectDefinitionManager::ThunkSubEcx4ToScalarDtor | confirmed | Secondary-vtable thunk (018089ac slot 0). Bytes 83 e9 04 subtract 4 from ECX, then e9 jumps to 00944fc0. It is not a body. |
| `00944f30` | SensoryEffectDefinitionManager::~SensoryEffectDefinitionMan… | confirmed | Destructor body: writes SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>::vftable at +0 and +4 and the HashMap<int,NameEntry> vtable at +0x440, … |
| `00944fc0` | SensoryEffectDefinitionManager::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: calls the body 00944f30 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |

**SensoryEffectDefinitionManager<SoundControlDefinition,256>** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00924b60` | SensoryEffectDefinitionManager_SoundControl_Ctor | confirmed | Constructor: calls 008cf920 with the two stack arguments (loader constructor), stores the SensoryEffectDefinitionManager<SoundControlDefinition,256> vtable at … |
| `00924c10` | SensoryEffectSoundControl_Slot0Thunk | confirmed | Adjustor thunk: subtracts 4 from ECX and jumps to 00925240. |
| `00924c20` | SensoryEffectSoundControl_CountPositive | confirmed | Returns 1 when the dword at this+0x3c is greater than zero (signed), else 0. |
| `00924c30` | SensoryEffectSoundControl_ReturnAt34 | confirmed | Returns the dword at this+0x34. |
| `00924c40` | SensoryEffectSoundControl_ReturnAt3c | confirmed | Returns the dword at this+0x3c. |
| `00924fb0` | SensoryEffectDefinitionManager_SoundControl_Dtor | confirmed | Destructor body: stores the SensoryEffectDefinitionManager<SoundControlDefinition,256> vtables at this and this+4, stores the HashMap vtable at this+0x440, sca… |
| `00925240` | SensoryEffectSoundControl_ScalarDtor | confirmed | Scalar deleting destructor: calls the body 00924fb0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00924d80` | SensoryEffectSoundControl_CastGetAt0xd0 | likely | Calls 00d8d382 with the argument, the RTTI descriptors of SensoryEffectDefinition and SoundControlDefinition and 0 (a dynamic-cast helper), then returns the dw… |

**ShipCombatAssetProcessor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c52ac0` | ShipCombatAssetProcessor::Dtor | confirmed | Restores ShipCombatAssetProcessor's vtable, destroys its seven effect-definition maps, restores the AssetRequester vtable, then releases its retained requester… |
| `00c534f0` | ShipCombatAssetProcessor::Ctor | confirmed | Stores the two constructor inputs, installs the ShipCombatAssetProcessor vtable, and initializes seven 40-bucket maps for composite, particle, sound, model-mat… |
| `00c535c0` | ShipCombatAssetProcessor::ScalarDeletingDtor | confirmed | Calls the processor destructor, then frees the object when flag bit 0 is set; otherwise returns the object pointer. |

**SoeUtil::HashListMap<int,ActorCameraEffectDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c502a0` | CameraEffectDefinitionMapInit | confirmed | Installs the camera-effect map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorCompositeEffectDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c50020` | CompositeEffectDefinitionMapInit | confirmed | Installs the specialized map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorLightEmitterDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c50220` | LightEmitterDefinitionMapInit | confirmed | Installs the light-emitter map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorModelMaterialDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c501a0` | ModelMaterialDefinitionMapInit | confirmed | Installs the model-material map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorParticleEmitterDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c500a0` | ParticleEmitterDefinitionMapInit | confirmed | Installs the particle-emitter map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorRumbleEffectDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c50320` | RumbleEffectDefinitionMapInit | confirmed | Installs the rumble-effect map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoeUtil::HashListMap<int,ActorSoundEmitterDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c50120` | SoundEmitterDefinitionMapInit | confirmed | Installs the sound-emitter map vtable, clears its counters, zeroes 40 bucket slots, and returns this. |

**SoundControlDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010ba240` | SoundControlDefinition::ReadAttributes | confirmed | Virtual [7] of SoundControlDefinition, thiscall with ECX=this and one stack argument (the element). Calls 007ca120 on the element first. Reads category and sub… |

**(no class)** (34)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c51e00` | LogUnavailableEffectDefinitions | confirmed | Checks two virtual predicates on entries in seven effect lists and logs each unavailable entry with its category, label, and state values. |
| `00923a40` | KeyedElement_CopyKeyAndInitString | likely | Copies the dword at *param_2 into *param_1 and then calls 009239d0 (the empty IString constructor) on the element; returns param_1. |
| `00923f20` | KeyedElement_CopyKeyAndInitString_2 | likely | Copies the dword at *param_2 into *param_1 and then calls 00923eb0 (the empty IString constructor) on the element; returns param_1. |
| `00924400` | KeyedElement_CopyKeyAndInitString_3 | likely | Copies the dword at *param_2 into *param_1 and then calls 00924390 (the empty IString constructor) on the element; returns param_1. |
| `009248e0` | KeyedElement_CopyKeyAndInitString_4 | likely | Copies the dword at *param_2 into *param_1 and then calls 00924870 (the empty IString constructor) on the element; returns param_1. |
| `00924e20` | KeyedElement_CopyKeyAndInitString_5 | likely | Copies the dword at *param_2 into *param_1 and then calls 00924db0 (the empty IString constructor) on the element; returns param_1. |
| `00926cf0` | LinkActorLightEmitterDefinition | likely | Links a node after the tail, updates head if empty, inserts it into its hash bucket, and increments count. |
| `00927710` | LinkActorCameraEffectDefinition | likely | Links a node after the tail, updates head if empty, inserts by byte bucket key, and increments the count. |
| `0093d030` | InsertActorModelMaterialDefinition | likely | Allocates a node through the owner allocator; on success constructs the material-definition record, links previous/next at node offsets 0x64/0x60, updates mana… |
| `00941310` | InsertActorCompositeDefinition | likely | Calls the map virtual slot +4 on param_1 with no explicit argument to obtain node storage. For a nonzero result, constructs a definition with 009412a0 using pa… |
| `00946390` | DefinitionAllocConstructLink | likely | Calls virtual slot 1 of the receiver (ECX = the member at this+0x2c; call [vtbl+4] with no arguments) to obtain storage. When the result is non-zero it runs 00… |
| `00c51930` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x14, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the n… |
| `00c519e0` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0xc8, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the n… |
| `00c51a90` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x17c, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the … |
| `00c51b40` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x230, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the … |
| `00c51bf0` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x2e4, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the … |
| `00c51ca0` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x398, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the … |
| `00c51d50` | ClearHashListAndReleaseRefs | likely | Drains the list head at this+0x44c, removes each node from its 40-bucket chain and doubly linked list, decrements its owner's positive refcount, and frees the … |
| `00c53960` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c53660, increments the… |
| `00c539e0` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c536e0, increments the… |
| `00c53a60` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c53760, increments the… |
| `00c53ae0` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c537e0, increments the… |
| `00c53b60` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c53860, increments the… |
| `00c53be0` | AddIfMissingAndIncrementRefCount | likely | Checks two lookup/precondition helpers; when both lookups succeed and the map-membership helper returns zero, creates an entry through 00c538e0, increments the… |
| `00c53c60` | ProcessDefinitionDependencies | likely | Looks up the initial entry, creates it if absent, increments its count at +8 and invokes virtual slot +0x24 when the prior count was zero; then walks linked de… |
| `010b7fd0` | WriteEffectDefinitionEntry | likely | Fastcall, ECX=definition object (vtable at +0). Allocates 0x50 bytes and builds an element named EffectDefinition through 00d52e70. Writes attribute 017eda5c f… |
| `010b8650` | ReadFloat4Attribute | likely | Returns a 16-byte struct through the hidden result pointer param_1. Looks up the attribute named by param_3 through 00d517f0; when absent, copies the 16-byte d… |
| `010b8850` | ReadBoolAttributeOrDefault | likely | Stack arguments: attribute name and a byte* default; returns a byte in AL. Looks up the attribute through 00d517f0; when absent returns the byte at the default… |
| `010b8c20` | WriteBoolAttribute | likely | Stack arguments: attribute name and a byte* flag (the caller passes the address of a 0/1 byte). Builds an IString temporary from that flag through 010b56e0 (a … |
| `010b8cc0` | FormatFloat4ToIString | likely | Thiscall-style with the hidden result pointer param_1 and a float* argument. Formats the four floats at the argument with the format string "%f %f %f %f" throu… |
| `010b8e60` | WriteFloatAttribute | likely | Stack arguments: attribute name and a float value. Formats the float through 010b59d0 into an IString temporary and writes its buffer as the named attribute th… |
| `010b98a0` | WriteStringAttributeFromCStr | likely | Stack arguments: attribute name, the element, and a C string value. Builds an IString temporary from the value through 010b96f0, writes its buffer as the named… |
| `010b9940` | ReadIStringAttributeOrDefault | likely | Hidden result pointer param_1 (an IString), attribute name in param_3, and a default IString in param_4. Looks up the attribute through 00d517f0; when present,… |
| `010bfdd0` | TriggerListAppendNew | likely | Fastcall with the trigger list in ECX (head +4, tail +8, count +0xc). Allocates a block through the list's vtable slot [2] (+8); when the allocation succeeds, … |

### `engine.asset`

**ActorCameraEffectDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cfba0` | ActorCameraEffectDefinition::~ActorCameraEffectDefinition | confirmed | Destructor of an actor camera-effect definition: rewrites ActorCameraEffectDefinition::vftable at +0 and +0x90, sets the +0x90 sub-object to AssetManagement::A… |
| `008cfc90` | ActorCameraEffectDefinition::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the ActorCameraEffectDefinition destructor body FUN_008cfba0 and frees the object with _free when bit 0 of the flag argum… |

**ActorLightEmitterDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cfa90` | ActorLightEmitterDefinition::~ActorLightEmitterDefinition | confirmed | Destructor of an actor light-emitter definition: rewrites ActorLightEmitterDefinition::vftable at +0 and at +0x90, sets the +0x90 sub-object vtable to AssetMan… |
| `008cfb80` | ActorLightEmitterDefinition::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the ActorLightEmitterDefinition destructor body FUN_008cfa90 and frees the object with _free when bit 0 of the flag argum… |

**ActorLightsaberTrailDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cfdb0` | ActorLightsaberTrailDefinition::~ActorLightsaberTrailDefini… | confirmed | Destructor of an actor lightsaber-trail definition: rewrites ActorLightsaberTrailDefinition::vftable at +0 and +0x90, releases the embedded SoeUtil::IString at… |
| `008cfec0` | ActorLightsaberTrailDefinition::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the ActorLightsaberTrailDefinition destructor body FUN_008cfdb0 and frees the object with _free when bit 0 of the flag ar… |

**ActorRumbleEffectDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cfcb0` | ActorRumbleEffectDefinition::~ActorRumbleEffectDefinition | confirmed | Destructor of an actor rumble-effect definition: rewrites ActorRumbleEffectDefinition::vftable at +0 and +0x90, sets the +0x90 sub-object to AssetManagement::A… |
| `008cfd90` | ActorRumbleEffectDefinition::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the ActorRumbleEffectDefinition destructor body FUN_008cfcb0 and frees the object with _free when bit 0 of the flag argum… |

**AppUtil::FileRep** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae2d50` | FileRepCtor | confirmed | Calls the base file representation constructor, installs AppUtil::FileRep vtable, and returns this. |
| `00ae2db0` | FileRepIsComplete | confirmed | Calls vtable methods at offsets +0x1c and +0x18 and returns 1 when their 64-bit results are equal, otherwise 0. |
| `00ae2de0` | FileRep::ScalarDeletingDtor | confirmed | Calls the file representation base destructor, optionally frees this when flag bit 0 is set, and returns this otherwise. |

**AssetDeliveryHandlerInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089edd0` | AssetDeliveryHandlerInterface::ScalarDeletingDestructor | likely | Thin scalar deleting destructor: calls FUN_010935e0 on this (that routine writes AssetManagement::AssetDeliveryHandlerInterface::vftable and tells the handler'… |

**AssetManagement::AssetDeliveryIndirect** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01098b80` | AssetDeliveryIndirect::ForwardSub62cSlot6 | likely | Bytes: cmp dword [ecx+0x62c],0; if non-null loads that object into ECX, reads its vtbl and jumps to vtbl slot +0x18 (tail jump, slot [6]); otherwise returns. N… |
| `01098ba0` | AssetDeliveryIndirect::ReturnThis | likely | Bytes 8b c1 c3: mov eax,ecx; ret. Returns its receiver in EAX; no arguments and no other effect. |
| `01098bd0` | AssetDeliveryIndirect::TailCallSlot7 | likely | Bytes 8b 01 8b 40 1c ff e0: loads the receiver's vtable, reads slot +0x1c (slot [7]) and jumps to it with ECX unchanged (tail call to the same object's slot [7… |
| `01098bf0` | AssetDeliveryIndirect::CallSub4DrainPending | likely | Fastcall (ECX=this), no stack args. Calls the object at this+0x62c through its vtbl slot [4] (no arguments), calls FUN_00fc6820 with the argument -1 and ECX=th… |
| `01098c20` | AssetDeliveryIndirect::ForwardSub62cSlot5 | likely | Bytes 8b 89 2c 06 00 00 8b 01 8b 50 14 ff e2: loads the object at this+0x62c into ECX and tail-jumps to its vtbl slot +0x14 (slot [5]); no stack args. |
| `010994d0` | AssetDeliveryIndirect::ClearLinksToObject | likely | Thiscall (ECX=this), one stack arg (object pointer), no return value. Walks the list at this+0x63c (next at +0x64) and sets the +8 field of every entry that eq… |

**AssetManagement::AssetDeliveryIndirect::DecompressJob** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010998b0` | DecompressJobDtorBody | confirmed | Fastcall (ECX=this), no stack args, no return value. Stores the DecompressJob vtable (01884e08) at this, releases the object at this+0xb0 when set (refcount at… |

**AssetManagement::AssetDeliveryIndirect::PackJob** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010995e0` | PackJobCtor | likely | Fastcall (ECX=this), no stack args, returns this. Calls FUN_0083f1e0 with the argument 0, then stores the PackJob vtable (01884da8) at this. |

**AssetManagement::AssetDeliveryIndirect::PendingRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099b80` | PendingRequestDtorBody | confirmed | Fastcall (ECX=this), no stack args, no return value. Stores the PendingRequest vtable (01884e28) at this, sets a local pointer to this+0xc, and calls FUN_00402… |
| `01099b00` | AssetDeliveryIndirect::PendingRequestCtor | likely | Thiscall (ECX=this), three stack args (param_2 C string, param_3 stored at +8, param_4 stored at +0x60), returns this. Stores the PendingRequest vtable (01884e… |

**AssetManagement::AssetDeliveryIndirect::ReadJob** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01099730` | ReadJobDtorBody | confirmed | Fastcall (ECX=this), no stack args, no return value. Stores the ReadJob vtable (01884dc8) at this, releases the object at this+0xb0 when set (refcount at +0x14… |

**AssetManagement::AssetDeliveryIndirect::WriteJob** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010997f0` | WriteJobDtorBody | confirmed | Fastcall (ECX=this), no stack args, no return value. Stores the WriteJob vtable (01884de8) at this, releases the object at this+0xb4 when set (refcount at +0x1… |

**AssetManagement::AssetDeliveryInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010947d0` | AssetDeliveryInterface::DrainPendingLists | likely | Calls FUN_01094560(this) then tail-jumps (jmp) to FUN_010946b0 with ECX=this; no stack arguments of its own (bytes: push esi; mov esi,ecx; call 01094560; mov e… |

**AssetManagement::AssetRequestHandler** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01098940` | AssetRequestHandler::DtorBody | confirmed | Fastcall (ECX=this), no stack args, ret 0. Stores the AssetRequestHandler vtable (01805f24) at this+0 and (01805f08) at this+4. When the count at this+0xb5a4 i… |
| `01095e60` | AssetRequestHandler::ReleaseAllEntries | likely | Fastcall (ECX=this), no stack args. Walks the list headed at this+0x135b0 (next at node+4): for each node releases the object at node+0 (refcount at object+4, … |
| `01096830` | AssetRequestHandler::InsertHashedEntry | likely | Thiscall (ECX=handler), one stack arg (node), ret 4. Computes a hash with FUN_00705b00 of the name copied from the receiver by FUN_0071c050 (which copies recei… |
| `01096900` | AssetRequestHandler::FindHashedEntryByName | likely | Thiscall (ECX=handler), one stack arg (key string pointer), ret 4, returns the node in EAX or 0. Hashes the key with FUN_00705b00 and walks the 8192-bucket tab… |
| `010986f0` | AssetRequestHandler::PeriodicTick | likely | Thiscall (ECX=handler), two stack args (decrement amount, budget), no return value. Calls the subobject at handler+0x38 through its vtbl[22], runs ListWalkDisp… |
| `01098750` | Thunk_Adjust3c_010984f0 | likely | Bytes 83 c1 3c (add ecx, 0x3c) then e9 (jmp 010984f0): an adjustor thunk that moves ECX to the subobject at handler+0x3c and jumps to ListFindByKeyDispatchRemo… |

**AssetManagement::AssetRequestInternal::AssetReadyHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010988d0` | AssetReadyHandler::DtorBody | confirmed | Fastcall (ECX=this), no stack args, ret 0. Stores the AssetReadyHandler vtable at this, runs the DeliverableAsset list destructor FUN_0091fd60 and then the rea… |

**AssetRequestInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ec80` | AssetRequestInterface::~AssetRequestInterface | confirmed | Seven-byte destructor body: writes AssetManagement::AssetRequestInterface::vftable into the object and returns; nothing is released and no other call is made. |
| `0089ecc0` | AssetRequestInterface::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: writes AssetManagement::AssetRequestInterface::vftable into the object and calls _free(this) when bit 0 of param_2 is set. |

**BaseClientFile** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae2e00` | BaseClientFileCtor | confirmed | Constructs the FileRep base, installs BaseClientFile vtable, zeros cursor/size/status fields, and initializes the embedded helper at the object tail. |
| `00ae2e70` | BaseClientFile::SetStatus5 | confirmed | Stores integer 5 at this+0x10 and returns byte zero. |
| `00ae2ea0` | BaseClientFile::GetStatus | confirmed | Loads and returns the dword at this+0x10. |
| `00ae2fa0` | BaseClientFile::IsOpen | confirmed | Returns false when no shared representation exists and backend is not ready; otherwise returns true. |
| `00ae30c0` | BaseClientFile::Read | confirmed | Copies up to the available byte count from shared representation into the caller buffer, advances the 64-bit cursor and optional read count, or delegates to th… |
| `00ae3190` | BaseClientFile::Seek | confirmed | Delegates to backend seek when no shared representation exists; otherwise clamps the requested 64-bit position to file size and stores it as the cursor. |
| `00ae3280` | BaseClientFile::ScalarDeletingDtor | confirmed | Calls BaseClientFile destructor body, optionally frees this on flag bit 0, and returns this otherwise. |
| `00ae3310` | BaseClientFile::Open | confirmed | Resets cursor fields, gets the active opener, normalizes the path separator position, requests a shared representation, replaces any old representation with re… |
| `00ae31f0` | BaseClientFileDtor | likely | Destroys the embedded helper, decrements and clears the shared representation, and invokes the FileRep base destructor. |

**BaseClientFileOpener** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8990` | BaseClientFileOpener::BaseClientFileOpener | confirmed | Writes AppUtil::FileOpener::vftable followed by BaseClientFileOpener::vftable to the object. |
| `00ae32a0` | CreateBaseClientFile | confirmed | Allocates 0x80 bytes, constructs BaseClientFile when allocation succeeds, and returns the object pointer or null. |
| `00ae34d0` | BaseClientFileOpener::OpenForRead | confirmed | Gets the opener backend, normalizes path separators, checks existence, initializes output metadata when supplied, obtains file size from the shared representat… |

**BattleMagesProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b70cb0` | BattleMagesProcessor::CreateParticleDefinitionAsset | confirmed | Vtable slot 3 allocates 0x68 bytes with 00d833a6; if allocation succeeds, passes the block as this to 0071bed0 with the processor manager pointer, a manager-de… |

**FishingProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b663a0` | FishingProcessor::RequestAsset | likely | Vtable slot 3: allocates a 0x68-byte object with FUN_00d833a6 and, when the allocation succeeded, calls FUN_0071bed0 with this+4 (the value the constructor sto… |

**ImageDirectory** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00987f60` | ImageDirectory_FindImageId | likely | Searches three hash-table regions for param_2 using masks 0x3fff, 0x3ff and 0x3ff; on a matching key returns the first word of the value slot, or returns zero … |

**ImageDirectory::ImageAsset2** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989470` | ImageAsset2_Constructor | confirmed | Calls 0071bb80 for base setup, installs ImageAsset2's vtable, zeroes field param_1[10], and returns this. |
| `009894d0` | ImageAsset2_Destructor | confirmed | Installs the ImageAsset2 vtable, releases and detaches the image reference at param_1[10] when nonzero, clears that field, calls 0071bc20 for base teardown, an… |
| `00989540` | ImageAsset2_OnImageBytes | confirmed | Initializes a temporary IString and calls Asset vtable slot +8 with ECX=the Asset object and a stack pointer to that IString (raw bytes show the Ghidra stack-c… |

**ImageDirectory::PendingLoad2** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00989030` | PendingLoad2_OnAssetReady | confirmed | Casts param_2 from Asset to ImageAsset2 using the two RTTI descriptors, calls GFxWrap::ReclaimImageFromAssetLimbo and ReleaseImageReference using this+8, calls… |
| `00989740` | PendingLoad2_CreateImageAsset | confirmed | Allocates 0x2c bytes using the guarded allocator 00d833a6, returns null if allocation fails, otherwise calls the ImageAsset2 constructor and returns the new ob… |

**MiniMapImages** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0cc70` | MiniMapImages::Dtor | confirmed | Restores MiniMapImages and IString vtables, releases the string reference, clears two hash lists, and detaches the asset requester. |
| `00b0cff0` | MiniMapImages::Ctor | confirmed | Sets the requester base, constructs two hash-list maps, initializes fixed slots and an IString member, then zeros counters. |
| `00b0d100` | MiniMapImages::ScalarDeletingDtor | confirmed | Calls the MiniMapImages destructor and conditionally frees the object. |
| `00b0d3e0` | MiniMapImages::ParseTileInfo | likely | Reads records from a byte buffer, parses numeric fields, updates extents, hashes each record name, stores tile entries, then rebuilds an index. |
| `00b0d7a0` | MiniMapImages::OnTileInfoLoaded | likely | Selects the loaded string buffer when present, passes it to the tile-info parser, and returns success. |

**MiniMapInfoAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0c820` | MiniMapInfoAsset::Ctor | confirmed | Calls the asset base constructor, installs MiniMapInfoAsset's vtable, and zeros member index 10. |
| `00b0c880` | MiniMapInfoAsset::Dtor | confirmed | Restores the asset vtable, decrements and conditionally destroys the shared member, then calls base asset cleanup. |
| `00b0c900` | MiniMapInfoAsset::OnLoaded | confirmed | Drops an old shared reference, terminates nonempty input text, increments the new reference count, stores its pointer, sets a ready byte, and calls a completio… |
| `00b0ca30` | MiniMapInfoAsset::ScalarDeletingDtor | confirmed | Calls the asset cleanup and frees the object when the low flag bit is set; otherwise returns the pointer. |

**SensoryEffectDefinitionLoader** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008cf920` | SensoryEffectDefinitionLoader::SensoryEffectDefinitionLoader | confirmed | Constructor of the sensory-effect definition loader: installs the two interface vtables (SensoryEffectDefinitionManagerInterface then AssetManagement::AssetDel… |
| `008cf9d0` | SensoryEffectDefinitionLoader::~SensoryEffectDefinitionLoad… | confirmed | Destructor of the sensory-effect definition loader: rewrites both loader interface vtables, releases the embedded SoeUtil::IString at +0x14 (capacity +0x20) th… |
| `008cfa70` | SensoryEffectDefinitionLoader::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the loader destructor body FUN_008cf9d0 and frees the object with _free when bit 0 of the flag argument is set. |

**SoeUtil::HashListMap<int,AssetManagement::Asset*,8192,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d4270` | AssetHashListMap_ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (slot [0] of the asset hash-list-map vtable): rewrites the object vtable to the HashListMap vtable, frees every bucket node at … |

**SoeUtil::HashListMap<int,AssetManagement::AssetRequestInternal::AssetReadyHandler::HashEntry,1024,32>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e8a0` | HashListMap_AssetReadyHandler_Dtor | confirmed | Destructor of the asset-ready hash container: writes the derived HashListMap vtable, drains the node list at +8 through FUN_008fa980, frees the two bucket-arra… |

**SoeUtil::HashListMap<int,DelayedAssetLoader::TrackedAsset,256,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae3810` | TrackedAssetHashListInit | confirmed | Installs the 256-bucket map vtable, clears head/tail/count and zeroes 256 bucket heads, then returns this. |
| `00ae3d70` | TrackedAssetHashAlloc | confirmed | Initializes memory management, asks thread/global managers for a node allocation, and falls back to requesting 0x24 bytes aligned to 4. |
| `00ae3db0` | TrackedAssetHashFree | confirmed | Pushes pointer and alignment 4 to FreeMemory (00401c00), then returns. |

**SoeUtil::HashListMap<int,MiniMapImage,1024,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0c400` | MiniMapImageMap::Ctor | confirmed | Installs the hash-list map vtable, zeros list metadata, clears 1024 bucket heads, and returns the map object. |

**SoundDefinitionAssetLoader** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2530` | SoundDefinitionAssetLoader::SoundDefinitionAssetLoader | confirmed | Constructor of a multiple-inheritance asset loader: installs SoundDefinitionLoader::vftable, zeroes param_1[1], installs AssetManagement::AssetRequestInterface… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8af0` | DownloadMandatoryAssetsProgress | confirmed | Initializes a progress monitor, polls until the mandatory-asset download completes, logs progress when counters change, and logs the final error unless status … |
| `008cf9c0` | AdjustorThunk_sub4_to_008cfa70 | likely | Eight-byte adjustor thunk (83 e9 04 e9 a8 00 00 00): subtracts 4 from ecx (the +4 interface sub-object) and jumps to the SensoryEffectDefinitionLoader scalar d… |
| `008cfb70` | AdjustorThunk_sub0x90_to_008cfb80 | likely | Eleven-byte adjustor thunk (81 e9 90 00 00 00 e9 05 00 00 00): subtracts 0x90 from ecx and jumps to 008cfb80, the scalar deleting destructor of the main ActorL… |
| `008cfc80` | AdjustorThunk_sub0x90_to_008cfc90 | likely | Eleven-byte adjustor thunk (81 e9 90 00 00 00 e9 05 00 00 00): subtracts 0x90 from ecx and jumps to 008cfc90, the scalar deleting destructor of the main ActorC… |
| `008cfd80` | AdjustorThunk_sub0x90_to_008cfd90 | likely | Eleven-byte adjustor thunk (81 e9 90 00 00 00 e9 05 00 00 00): subtracts 0x90 from ecx and jumps to 008cfd90, the scalar deleting destructor of the main ActorR… |
| `008cfeb0` | AdjustorThunk_sub0x90_to_008cfec0 | likely | Eleven-byte adjustor thunk (81 e9 90 00 00 00 e9 05 00 00 00): subtracts 0x90 from ecx and jumps to 008cfec0, the scalar deleting destructor of the main ActorL… |
| `00ae3740` | StoreFieldAt18 | likely | Stores the second argument at object offset +0x18 and returns. |
| `00ae48d0` | RefillDelayedAssetRequests | likely | Consumes accumulated units in 0x100 chunks, rotates queued entries, allocates request records via vtable, appends them to array, and updates pending counters. |
| `00b16fd0` | Thunk_00ae6b90_Adjust10 | likely | Eight-byte adjustor thunk: raw bytes 83 c1 10 e9 add 0x10 to this and jump to 00ae6b90, the delayed asset loader, with no frame of its own. |
| `00b77760` | AcquireAssetRefs | likely | Increments the container's live count at +8 (running its vtable slot +0x24 hook on the 0->1 transition) and then, for every dependency entry of the resolved as… |
| `00b778c0` | ReleaseAssetRefs | likely | Mirror of FUN_00b77760: decrements the live count at +8 of every dependency resource of the resolved asset (same type-id dispatch 0/1/2/3/4/8 over the list at … |

### `engine.dbm`

**AppUtil::DirectoryWithCache** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015389d0` | AppUtil::DirectoryWithCache::DirectoryWithCache | confirmed | Fastcall constructor: calls 00af5a60 with 0, stores AppUtil::DirectoryWithCache::vftable at this, sets the stamp pair at +0x9338/+0x933c, the cached IString vt… |
| `01538a60` | AppUtil::DirectoryWithCache::~DirectoryWithCache | confirmed | Scalar deleting destructor (vtable slot [0]): calls 0109f6a0 (AssetDeliveryDirect::ReleaseBuffer9344 label) on this, then _free(this) only when bit 0 of the fl… |
| `01538720` | AppUtil::DirectoryWithCache::ResetCache | likely | Virtual slot [2]: if the count at +0x934c is positive, decrements the refcount of the buffer at +0x9344 and releases it through the vtable at +0x9340 (+8) when… |

**DBMInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c29c0` | DBMInterface::~DBMInterface | confirmed | Destructor body (fastcall): re-stores the DBMInterface vtable at +0 and returns. Nothing is freed here. |
| `013c29e0` | DBMInterface::ScalarDeletingDtor | confirmed | Virtual slot [0] of DBMInterface (bytes f6 44 24 04 01 / 56 / 8b f1 / c7 06 ... / 74 09 / 56 / e8 ... / c3): stores the DBMInterface vtable, calls _free(this) … |

**Deck** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012a8530` | Deck::Deserialize | likely | Reads one integer class id with 013c25b0 (returns null when that fails), creates an object through the factory found with 013ce130 for that id, dynamic-casts i… |

**DeckDB** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012a85e0` | DeckDB::StoreDeck | likely | Thiscall (ECX = DeckDB, param_2 = object): asserts the object is non-null (DeckDB.cpp line 0xdd) and that mDBMStorage at this+0xc is set (line 0xde). It calls … |
| `012a8ad0` | DeckDB::StoreDeckWithKey | likely | Thiscall (ECX = DeckDB, param_2 = object): asserts the object is non-null (DeckDB.cpp line 0xce) and that mDBMStorage at this+0xc is set (line 0xcf). It obtain… |

**DirDBM** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c4fe0` | DirDBM::DtorBody | confirmed | Sets the vtable to DirDBM::vftable, frees the block at this+0x54, erases and frees the string-keyed index at this+0x40 through 00f98150, tidies the std::string… |
| `013c5510` | DirDBM::DirDBM | confirmed | Stores DBMInterface::vftable then DirDBM::vftable, zeroes the std::strings at this+8 and this+0x24, initialises the index map at this+0x40 through 00f987e0, cl… |
| `013c55b0` | DirDBM::~DirDBM | confirmed | Scalar deleting destructor: calls the DirDBM destructor body 013c4fe0, frees the object when bit 0 of the argument is set, and returns this (ret 4). |
| `013c55d0` | DirDBM::ScanDirectory | confirmed | If GetFileAttributesA of this+0xc reports the directory attribute 0x10, builds the pattern from that directory, the constant '/*' (DAT_018db808) and the string… |
| `013c30c0` | DirDBM::WriteBlobToFile | likely | Virtual slot [7] of DirDBM: takes the file name from this+4 (inline or heap pointer by capacity at +0x18), opens it with fopen mode wb (returns false when that… |
| `013c4a90` | DirDBM::AddIndexEntry | likely | Copies its two std::string arguments into temporaries, builds a record with 013c3c10 and 013c3740, inserts it into the string-keyed map at this+0x40 through 00… |

**MessageDB** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012a66c0` | EnsureArchetypeDb | likely | Thiscall (ECX = MessageDB holder, param_2 = int id): asserts id != -1 ("v != -1", MessageDB.cpp line 0x20c) and looks the id up in the map at this+0xc, asserti… |
| `012a6950` | MessageDB::AppendIndexKeys | likely | Asserts param_2 != -1 ("v != -1", MessageDB.cpp line 100), calls 012a66c0(param_2) to make sure the archetype store for the id exists, then walks the tree retu… |
| `012a6a70` | MessageDB::LoadKeyedRecords | likely | Thiscall (ECX = MessageDB holder, param_2 = int id): asserts id != -1 ("v != -1", MessageDB.cpp line 0xf0) and ensures the id's store (012a66c0). Looks the id … |
| `012a7260` | MessageDB::FindMatchingKeys | likely | Asserts param_4 != -1 ("v != -1", MessageDB.cpp line 0x182) and ensures the id's store (012a66c0). Lowercases the query, walks the id's tree in order, lowercas… |
| `012a76e0` | MessageDB::AddTextIfAbsent | likely | Thiscall (param_3 = int id, param_1 = key, param_2 = std::string text): asserts id != -1 (MessageDB.cpp line 0x19a), ensures the id's store (012a66c0), finds t… |
| `012a77f0` | MessageDB::EraseTextIfPresent | likely | Thiscall (param_2 = int id, param_1 = key): asserts id != -1 (MessageDB.cpp line 0x1ad), ensures the id's store (012a66c0), finds the key in the id's tree (012… |
| `012a7890` | MessageDB::AddTextWithNextKey | likely | Thiscall (param_2 = int id, param_1 = std::string text): asserts id != -1 (MessageDB.cpp line 0x1b7), ensures the id's store (012a66c0), reads the tree size at… |
| `012a7b30` | MessageDB::ShutdownInstance | likely | Reads the global DAT_01cbd67c; when it is non-null it runs the teardown body 012a7a60 on it, frees it with _free, and then zeroes the global. |
| `012a7b60` | MessageDB::SetActiveIdAndLoad | likely | Thiscall (ECX = MessageDB, stack arg = int id): when id is -1 it stores 0x9f3 in this+0x30 and passes 0x9f3 on; otherwise it stores id at this+0x30. It then ta… |
| `012a7b90` | MessageDB::GetText | likely | Thiscall (out std::string at param_1, param_2 = key, param_3 = int id): asserts id != -1 (MessageDB.cpp line 0x130), ensures the id's store (012a66c0), loads t… |
| `012a7f10` | MessageDB::SaveArchetype | likely | Asserts id != -1 (MessageDB.cpp line 0x1d2). Serialises a header of the id and a __time64 timestamp with SerialAppendInteger, then walks the id's tree once to … |
| `012a8480` | MessageDB::DeleteInstance | likely | Thiscall (ECX = object): clears the singleton global DAT_01cbd67c to null first, then when the object is non-null runs the teardown body 012a7a60 on it and fre… |
| `012a84b0` | MessageDB::GetInstance | likely | Lazily creates the global singleton DAT_01cbd67c: when it is null, allocates 0x54 bytes with operator new, constructs the object through 012a83c0 and stores th… |

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a9990` | HashListMap_ColumnNameLookupNode_Construct | confirmed | Constructor of the column-name lookup map: installs the SoeUtil::HashListMap<int,ColumnNameLookupNode,128,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 1… |

**(no class)** (35)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01511ee0` | odb_get | confirmed | Takes this in arg1 (FILE* at this+0x10), an output pointer for the buffer (arg2), an optional size output (arg3) and a key (arg4). Sets *arg3 to -1 when non-nu… |
| `015134a0` | odb_put | confirmed | Writes one object into the binary object database: takes this in arg1, data in arg2, size in arg3 and key in arg4. Looks up the key with FUN_01536810. When the… |
| `01514960` | OdbNew | confirmed | Callocs 0x44 bytes for a database handle and strdups its path argument into +0. Strips trailing slashes from the copy, then builds three owned paths: path plus… |
| `01514b80` | OdbOpen | confirmed | Lazily callocs a 0x2000 global buffer, creates the handle with FUN_01514960, and stats the database directory (FUN_00d8dd48). If the directory is missing it pr… |
| `01514d10` | OdbCreate | confirmed | Lazily callocs the same 0x2000 global buffer, creates the handle with FUN_01514960, and stats the directory. If it is missing it creates it with FUN_0153b440 a… |
| `01536df0` | OdbLookup_SaveFile | confirmed | Thiscall on an ODB lookup object: serialises into a local serializer (initialised by 013c2210): an int at this+0x30, then every node of the int-keyed tree at t… |
| `01537b40` | OdbLookup_LoadFile | confirmed | Thiscall on an ODB lookup object: assigns the file name param_2 to the object's string at +0, opens it with _fopen mode rb, reads the whole file into a byte bu… |
| `01538110` | DirectoryCache_ProbeComBlock | confirmed | Returns a bool: calls 01539760 (guard setup), opens the named block 'DirectoryCacheComBlock' through 01539830, runs the cleanup thunk 015397c0, and returns whe… |
| `01538490` | DirectoryCache_SyncFromComBlock | confirmed | Thiscall on a directory-cache object (param_1): under the named mutex 'DirectoryCacheMutex' (00fc4630, 00fc45f0, 00fc4610) it opens the shared block 'Directory… |
| `012aa980` | DeckDB_Ctor | likely | Initialises the object: zeroes the tree at this+4/+8 through 012aa830, zeroes the 0x1c-stride vector at this+0x14/+0x18/+0x1c, allocates 0x58 bytes (00d833a6) … |
| `012aab80` | DeckDB_EnsureGlobal | likely | If the global DAT_01cbd680 is null, allocates 0x20 bytes with 00d833a6, constructs the DeckDB object in them with 012aa980 (ECX = block) and stores the block i… |
| `012aabf0` | DeckDB_ReleaseGlobal | likely | If the global DAT_01cbd680 is non-null, calls 012aa5d0 on it, frees the block with _free, then sets DAT_01cbd680 to null. |
| `012aace0` | DeckDB_AddDeck | likely | Asserts the deck argument is nonzero (DeckDB.cpp line 0xb4, text 'deck'), asserts that the key string from 013ce2c0 passes the 00f942a0 test (line 0xb7, text '… |
| `012aad90` | DeckDB_TryAddDeck | likely | Same assert pattern as DeckDB_AddDeck (lines 0xc1 and 0xc4, 'false' text), same key lookup via 012a9340, and inserts the deck under its key with 012aa8a0 only … |
| `012aae40` | DeckDB_GetCachedOrLoad | likely | Asserts the DBM object at this+0xc is nonzero (text mDBMStorage, DeckDB.cpp line 0x55). Searches the key with 012a9340; when the result is a real node (not the… |
| `012aaeb0` | DeckDB_ResolveKey | likely | Asserts the DBM object at this+0xc is nonzero (text mDBMStorage, line 0x86). Searches the 0x1c-stride vector at this+0x14..+0x18 for the key (012a90e0). When n… |
| `012aafe0` | DeckDB_AddKeyAndDeck | likely | Asserts the deck argument is nonzero (DeckDB.cpp line 0x12d, text 'deck'), copies the key string returned by 013ce2c0(deck) into a local std::string, and searc… |
| `012ab9f0` | CollectionDB_PutEntry | likely | Thiscall with the entry as its stack argument (ECX = this). Asserts the entry is nonzero (CollectionDB.cpp line 0x107, text collection) and that this+0xc (the … |
| `012ac430` | StrMapFlushEntries | likely | Thiscall (ECX = map owner). Walks the string-keyed tree in order from head.left using the iterator step 012ab5a0 on a local iterator. For each node whose value… |
| `012ad4c0` | DbmMap_FlushCloseClear | likely | Fastcall, ECX = owner object. If the DBM object at +0xc is nonzero: flushes the map (012ac430), calls the object's vtable +0x20, deletes the object through its… |
| `012adb30` | DbmMap_Teardown | likely | Fastcall (ECX = owner). Calls DbmMap_FlushCloseClear (012ad4c0) on the owner, then StdMapStrKey_TidyFreeFunclet2 (012adac0) on the same owner. Returns nothing. |
| `012adb90` | DbmMap_ScalarDtor | likely | Thiscall with one stack byte (ret 4, flag). Calls DbmMap_Teardown (012adb30) on the owner and frees the object with _free when bit 0 of the flag is set. Return… |
| `012adbb0` | DbmMap_ReleaseGlobal684 | likely | Fastcall, ECX = object pointer (param_1). Clears the global DAT_01cbd684; when the object is nonnull, runs DbmMap_Teardown (012adb30) on it and frees it with _… |
| `012add30` | CollectionDB_Ctor | likely | Fastcall, ECX = owner object (this). Initialises the string-keyed map at this+4 with 012adbe0, allocates 0x58 bytes and constructs a DirDBM there (013c5510), s… |
| `012adee0` | CollectionDB_EnsureGlobal | likely | No arguments. If DAT_01cbd684 is null, allocates 0x10 bytes (00d833a6) and constructs the collection database object there with CollectionDB_Ctor (012add30), s… |
| `012adf50` | CollectionDB_ReleaseGlobal | likely | No arguments. If DAT_01cbd684 is non-null, runs DbmMap_Teardown (012adb30) on it, frees the block with _free, then sets DAT_01cbd684 to null. |
| `012ae0f0` | CollectionDB_TryAddEntry | likely | Thiscall with one stack argument (collection object). Asserts the argument is nonzero (CollectionDB.cpp line 0xf7, text 'collection'), asserts that the key fro… |
| `012ae1a0` | CollectionDB_GetCachedOrLoad | likely | Thiscall (ECX = collection DB, param_2 = key). Asserts the DBM object at +0xc is nonzero (CollectionDB.cpp line 0x51, text mDBMStorage). Looks the key up with … |
| `012ae210` | CollectionDB_LoadAndRegister | likely | Thiscall (ECX = collection DB, param_2 = key). Asserts the DBM object at +0xc is nonzero (CollectionDB.cpp line 0x80, text mDBMStorage). Zeroes a local buffer … |
| `012ae350` | CollectionDB_PushKeysAndDbmSlot8 | likely | Thiscall (ECX = collection DB). Walks the string-keyed map in order from head.left with 012ab5a0 and passes each node's key string (node+0xc) to 01233d70 (Vect… |
| `012ae3f0` | CollectionDB_FindByState | likely | Thiscall with one stack argument (state value, compared with 0139f7a0 results). First walks the map in order and returns the first non-null entry whose 0139f7a… |
| `012ae630` | CollectionDB_FindByKey | likely | Thiscall with one stack argument (key string, size at +0x14, buffer at +4 or the pointer at +4 when capacity is 16 or more). Walks the loaded map in order and … |
| `015127e0` | BlockBitmapHexDump | likely | Zeroes an 80-byte stack buffer with _memset and writes the bytes of the buffer at this+0x14 as hex digit pairs from the table PTR_s_0123456789ABCDEF, inserting… |
| `01512ff0` | BlockBitmapAllocRun | likely | Takes this in EDI and a byte size in EAX. Converts the size to a block count (rounded up when there is a remainder). Searches the bitmap at this+0x14 for a run… |
| `01514ed0` | OdbClose | likely | Takes this in arg1 and a flag byte in arg2. Calls FUN_01536050 (no-op return) when the flag is zero, otherwise thunk FUN_01536df0. Then, if the composite at +0… |

### `engine.gds`

**BaseItemStatDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008acdd0` | BaseItemStatDefinition_DtorBody | likely | Writes BaseItemStatDefinition::vftable at offset 0 and returns - the base-class destructor emission (no members released). Reached from four EH unwind funclets… |
| `008ade10` | BaseItemStatDefinition_DtorBody_2 | likely | Second identical emission of the base-class destructor body: writes BaseItemStatDefinition::vftable at offset 0 and returns. No callers recorded in the export. |

**ClaimDataSource::AvailableItems** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c69150` | ClaimDataSource::AvailableItems::FieldName | confirmed | Virtual method at slot 1 of the class vtable: switches on the field index and returns the literal name of each column - 0 ItemId, 1 StackCount, 2 ImageSetId, 3… |
| `00c69570` | ClaimDataSource::AvailableItems::RefreshCachedCount | confirmed | Virtual method of the secondary AvailableItems sub-object vtable: adjusts this back by 0x220 to reach the complete object, calls virtual slot +0xc on it, then … |
| `00c695a0` | ClaimDataSource::AvailableItems::GetFieldValue | confirmed | Virtual method at slot 4, the field reader of the data source: for a valid row index (0 <= param_2 < the row count at table+0xa4) it computes the row as param_… |
| `00c697f0` | ClaimDataSource::AvailableItems::~AvailableItems | confirmed | Destructor body: rewrites the four AvailableItems vtables at +0, +0x30, +0x34 and +0x254 (the last one temporarily as InternalDsBase), then runs the GuiUserDat… |
| `00c69890` | ClaimDataSource::AvailableItems::ScalarDeletingDtorThunk | confirmed | Eleven-byte adjustor thunk at slot 0 of the secondary vtable: the bytes are sub ecx,0x254 followed by jmp 00c69a10, so it re-bases this onto the complete objec… |
| `00c69980` | ClaimDataSource::AvailableItems::AvailableItems | confirmed | Constructor taking a name pointer and an owner: runs the GUI data-table constructor FUN_00922890(param_2), stores the InternalDsBase vtable at +0x254, stores p… |
| `00c69a10` | ClaimDataSource::AvailableItems::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c697f0 and, when bit 0 of the flag is set, calls _free on this without … |

**ClaimDataSource::ClaimItemContents** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c691c0` | ClaimDataSource::ClaimItemContents::FieldName | confirmed | Virtual method at slot 1 of the class vtable: switches on the field index and returns the column name - 0 ItemId, 1 ItemName, 2 ItemQuantity, 3 ItemImageSetId,… |
| `00c69960` | ClaimDataSource::ClaimItemContents::ScalarDeletingDtorThunk | confirmed | Eleven-byte adjustor thunk at slot 0 of the secondary vtable: sub ecx,0x254 then jmp 00c69ad0, i.e. it re-bases this onto the complete item-contents object and… |
| `00c69970` | ClaimDataSource::ClaimItemContents::GetCount | confirmed | Virtual method at slot 2: returns the dword count at this+0x2a8, the number of claim-item entries in the contents array. |
| `00c69a30` | ClaimDataSource::ClaimItemContents::ClaimItemContents | confirmed | Constructor taking a name pointer and an owner: runs the GUI data-table base constructor FUN_00922890(param_2), stores the InternalDsBase vtable at +0x254 and … |
| `00c69ad0` | ClaimDataSource::ClaimItemContents::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c698a0 and calls _free on this when bit 0 of the flag is set. |

**ClaimDataSource::InternalDsBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c69140` | ClaimDataSource::InternalDsBase::~InternalDsBase | confirmed | Destructor body: writes ClaimDataSource::InternalDsBase::vftable into this+0 and returns without touching any other field or freeing memory. |

**ClientItemStatDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ace20` | ClientItemStatDefinition::ScalarDeletingDtor | confirmed | Scalar deleting destructor in slot 0 of ClientItemStatDefinition::vftable: rewrites BaseItemStatDefinition::vftable at offset 0 inside an SEH frame and, when b… |

**CompositeDrawObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00420615` | CompositeDrawObject::Ops324 | likely | Dispatches lifecycle operations for a 0x144-byte CompositeDrawObject. Op 0 default-constructs at param_2 through FUN_0041d672; op 1 copy-constructs there from … |

**DrawState** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00477d2c` | DrawStateScalarDeletingDtor | likely | Scalar deleting destructor: runs the class destructor body FUN_00477877 (which rewrites the two vtables 01765298/01765294, then calls FUN_00476f03, FUN_004735f… |

**GBitmap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047bcdb` | RegisterGBitmapClass | confirmed | Registers the class name GBitmap in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004ccf27… |

**GBitmapInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047bc10` | RegisterGBitmapInfoClass | confirmed | Registers the class name GBitmapInfo in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004c… |

**GBitmapInfoHeader** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047bb91` | RegisterGBitmapInfoHeaderClass | confirmed | Registers the class name GBitmapInfoHeader in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FU… |
| `0047bbc4` | CopyGBitmapInfoHeader | likely | Copies the 9-dword GBitmapInfoHeader structure field by field from param_2 to param_1. |

**GBitmapList** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047bfb3` | RegisterGBitmapListClass | likely | Registers the class name GBitmapList in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004c… |

**GBlitFx** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0040e7a8` | GBlitFx::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then frees the object through thunk_FUN_004c6a61 when the flag low bit i… |

**GCtrl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0040e507` | GCtrl::GetClientOffset | confirmed | Returns the control's client-area offset: reads the dwords at this+0x94 (x) and this+0x98 (y) and writes them through the hidden GPoint return pointer on the s… |

**GDSGeneric** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00481381` | RegisterGDSGeneric | confirmed | Registers the GFC class "GDSGeneric" (base size 0x18) with the class registry through FUN_0053da53/FUN_004ccf27/FUN_00536678. |

**GDSHitTest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00406875` | InstantiateGDSHitTest | confirmed | Creates/initializes the class instance named "GDSHitTest" with base size 0x18 through FUN_0053da53, finalizes it through FUN_004ccf27, then hands it to FUN_005… |

**GDrawDeviceD3D9** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00477947` | DrawDeviceShutdown | likely | When +0x49 is set, walks a global element list and calls each non-null entry's virtual +0x10 with 1. It then either iterates the +0x14 object's list when its +… |
| `004783f1` | DrawDeviceDtor | likely | Resets the device vtable words and clears +0x40; removes this device from the sequence associated with +0x10, and calls shutdown 00477947 when byte +0x49 is se… |
| `00479d94` | DrawDeviceFactory | likely | Class-factory dispatcher for the 0xcc-byte draw device: mode 0 constructs in place with 00474642, mode 1 copy-constructs in place with 0047441a, modes 2/3 allo… |
| `00479e2d` | DrawDeviceScalarDeletingDtor | likely | Scalar deleting destructor for the draw device: runs the destructor 004783f1 and, when the flag argument's low bit is set, frees the object through thunk_FUN_0… |

**GDrawSurface** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047a92b` | DrawSurfaceEmitLineRecord | likely | Ensures the cached adapter at +0x1cc exists. The param_9==0 path builds a line payload and submits it through FUN_00477fef with key DAT_01bc45d0. The two-part … |
| `0047aa87` | DrawSurfaceEmitStyledLine | likely | Returns 0 unless (+0x10 & 0x0c) is nonzero. It increments the nested counter at +0xa4, selects two values from fields +0x114/+0x118 according to (+0x104 & 0x30… |
| `0047ab48` | DrawSurfaceEmitPolyline | likely | Emits a connected polyline from a coordinate array: for param_2==1 walks pairs of points (stride 4 dwords) calling the line emitter on each pair, for param_2==… |
| `0047ae37` | EmitRectOutlineSegments | likely | After the (+0x10 & 0xc) guard, increments the counter at the nested object+0xa4 and emits lines through 0047a92b. Degenerate horizontal, vertical and point cas… |
| `0047afe1` | DrawSurfaceFactory | likely | Class-factory dispatcher for the 0x1e4-byte render surface: mode 0 constructs in place with FUN_00475da4, mode 1 copy-constructs in place with FUN_00475ac1, mo… |
| `0047b072` | DrawSurfaceScalarDeletingDtor | likely | Scalar deleting destructor for the render surface: runs the destructor 00479eb9 and, when the flag argument's low bit is set, frees the object through thunk_FU… |
| `0047b08e` | DrawSurfaceFillWithResource | likely | Checks param_4 and the mode helper, assembles style bits from +0x138 and color fields +0x154/+0x158, resolves the resource through FUN_004750e3, and emits a fi… |
| `0047b10f` | DrawSurfaceBlitResource | likely | Resolves param_4 through its virtual slot +8 using the context from FUN_004cd10a, then requires FUN_004772fc and (+0x10 & 0x0c)==0. For resolved state 1, it al… |
| `0047b2ab` | DrawSurfaceFillWithResourceTiled | likely | Tiled variant of the resource fill: same guard and style/colour assembly as 0047b08e, resolves the resource with FUN_004750e3, then emits through the tiled emi… |
| `0047b3ce` | DrawSurfaceFillTableWithResource | likely | Table variant of the resource fill: validates param_4, reads the surface style bits (+0x138 & 0x102) and colours +0x154/+0x158, resolves the resource into a fi… |
| `0047b44c` | DrawSurfaceBlitTableWithResource | likely | Blit variant driven by a rectangle table: converts the resource argument through its virtual slot +8, requires a bound target resource (+0x24) and an inactive … |

**GFillFx** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0040e740` | GFillFx::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then frees the object through thunk_FUN_004c6a61 when the flag low bit i… |

**GHBoxLayout** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00409b11` | GetStaticPair50EC | confirmed | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF50EC through param_2. |

**GLineFx** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0040e6f3` | GLineFx::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then, when the flag argument low bit is set, frees the object through th… |

**GPalette** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0047c1fe` | RegisterGPaletteClass | confirmed | Registers the class name GPalette in the engine class registry by calling FUN_0053da53 with the name and the constant 0x18, then publishing through FUN_004ccf2… |
| `0047b8b1` | GPaletteCtor | likely | Constructs a palette object: runs the base initialiser FUN_004d99d1, zeroes the colour-order field at +0x14, the 16-bit field at +0x1a, the colour-data pointer… |
| `0047b90d` | GPaletteCtorFromData | likely | Second palette constructor: same field initialisation as 0047b8b1 (vtables 01768b18/01768b10, +0x14/+0x1a/+0x1c zeroed) and then initialises the palette from p… |
| `0047c2fb` | GPaletteDataSize | likely | Calls FUN_004c6e81, whose body is a stub returning 0, zero-extends the u16 values at +0x1a and +0x18, multiplies them, adds the helper result, and returns the … |

**GStringAccessWrapper** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00409250` | InvokeVirtual50 | confirmed | Calls the receiver's virtual slot +0x50 with (param_2, 0, -1). |

**GVBoxLayout** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00409aeb` | GetStaticPair503C | confirmed | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF503C through param_2. |

**PlayerEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a8a40` | MatchPlayerEventKind_8_10_80_400_800 | confirmed | Predicate on one integer: returns 1 exactly when the argument is 8, 0x10, 0x80, 0x400 or 0x800, otherwise 0. No calls, no memory access. Installed as slot 2 of… |

**PromotionalBundleDatasource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c69cd0` | PromotionalBundleDatasource::FieldName | confirmed | Virtual method at slot 1: switches on the field index and returns Name (0 and 8), Description (1), ImageSetId (2), TintValue (3), ItemListDetails (4), BundleId… |
| `00c6a540` | PromotionalBundleDatasource::~PromotionalBundleDatasource | confirmed | Destructor body: rewrites the three PromotionalBundleDatasource vtables at +0, +0x30 and +0x34, destroys the embedded SoeUtil::HashListMap through FUN_009c8580… |
| `00c6a5d0` | PromotionalBundleDatasource::GetCount | confirmed | Virtual method at slot 2: returns the dword at this+0x238, the number of bundle rows the data source exposes. |
| `00c6a680` | PromotionalBundleDatasource::PromotionalBundleDatasource | confirmed | Constructor taking a name pointer and an owner: runs FUN_00922890(param_2) for the GUI data-table base, stores the owner at +0x254, installs the three Promotio… |
| `00c6a700` | PromotionalBundleDatasource::ScalarDeletingDtor | confirmed | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c6a540 and calls _free on this when bit 0 of the flag is set. |
| `00c6a2d0` | PromotionalBundleDatasource::GetFieldValue | likely | Virtual method at slot 4, the bundle field reader: resolves the bundle id through virtual slot +0x10 of the object at this+4, then looks the entry up in the bu… |

**PromotionalDataSource::LastPromotionalAward** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c69c60` | PromotionalDataSource::LastPromotionalAward::FieldName | confirmed | Virtual method at slot 1: switches on the field index and returns 0 ItemId, 1 Type, 2 Name, 3 Description, 4 ImageSetId, 5 TintValue, otherwise the shared <err… |
| `00c6a180` | PromotionalDataSource::LastPromotionalAward::GetFieldValue | confirmed | Virtual method at slot 4, the award field reader: requires this+0x228 plus a table reached through this+0x224's +0x15c, whose +0xa8 dword is the row count and … |
| `00c6a4b0` | PromotionalDataSource::LastPromotionalAward::~LastPromotion… | confirmed | Destructor body: rewrites the three LastPromotionalAward vtables at +0, +0x30 and +0x34 and then runs the GUI data-table base destructor FUN_0091bf60 under an … |
| `00c6a520` | PromotionalDataSource::LastPromotionalAward::GetCount | confirmed | Virtual method at slot 2: returns the dword at this+0x228, the current number of award rows the data source exposes. |
| `00c6a5e0` | PromotionalDataSource::LastPromotionalAward::LastPromotiona… | confirmed | Constructor taking a name pointer and an owner: runs the GUI data-table base constructor FUN_00922890(param_2), stores the three LastPromotionalAward vtables a… |
| `00c6a660` | PromotionalDataSource::LastPromotionalAward::ScalarDeleting… | confirmed | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c6a4b0 and calls _free on this when bit 0 of the flag is set. |

**SoeUtil::HashListMap** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa950` | HashListMap_RailDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,RailDefinition,64,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 64 bucket pointers (0x100 bytes) at +0… |
| `008aaba0` | HashListMap_RailCameraDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,RailCameraDefinition,64,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 64 bucket pointers (0x100 bytes)… |
| `008a9910` | HashListMap_ClientItemStatDefinition_Construct | likely | Constructor of the client item-stat-definition map: installs the SoeUtil::HashListMap<int,ClientItemStatDefinition,71,-1> vtable, zeroes +4/+8/+0xc/+0x10 and m… |
| `008a9d60` | HashListMap_IString_2048_Construct | likely | Constructor of the int-to-IString map with 2048 buckets: installs the SoeUtil::HashListMap<int,SoeUtil::IString<char>,2048,-1> vtable, zeroes +4/+8/+0xc/+0x10 … |

**Target** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b1b70` | Target::Target | confirmed | Constructor: writes Target::vftable at offset 0, zeroes this+4 and calls FUN_0101b910(param_2, param_3, param_4); returns this. No branches. |

**TileEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a8a80` | TileEventHandler::MatchesValues_8_10_40_8 | likely | Predicate on two integers: returns 0 only when param_2 is neither 8 nor 0x10 and param_1 is neither 0x40 nor 8; i.e. it returns 1 when param_2 is 8 or 0x10, or… |

**(no class)** (268)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004073e0` | InvokeVirtual24ShortId | confirmed | Calls virtual slot +0x24 on the receiver with the receiver's field at +0x8C (truncated to 16 bits) followed by param_2..param_4. |
| `004073fc` | InvokeVirtual2CShortId | confirmed | Calls virtual slot +0x2C on the receiver with the receiver's field at +0x8C (truncated to 16 bits) followed by param_2..param_5. |
| `0040741c` | InvokeVirtual28ShortId | confirmed | Calls virtual slot +0x28 on the receiver with the receiver's field at +0x8C (truncated to 16 bits) followed by param_2..param_4. |
| `00407c49` | NestedVirtual8_6C_8Probe | confirmed | Constructs a stack scratch object through FUN_004c6dcb, resolves a value through FUN_004cd10a, invokes the receiver's virtual slot +8 with it, copies the globa… |
| `00409025` | InvokeVirtual90Tag10Region | confirmed | Materializes the caller's record through FUN_004c9ed9, builds the stack argument record (vtable 0x175d184, region container, tag 0x10) with the element structu… |
| `00409563` | Clear400AndQuery2 | confirmed | Clears bit 0x400 of the word at +0x14 and returns the inverse of bit 2 of the same word (1 when bit 2 is clear); no calls. |
| `004098e5` | InvokeVirtual58Neg1 | confirmed | Calls the receiver's virtual slot +0x58 with (-1, param_2). |
| `004098f3` | InvokeVirtual58Neg1B | confirmed | Calls the receiver's virtual slot +0x58 with (-1, param_2); identical body to 004098e5 (second emission). |
| `0045bac0` | CreateGDrawDeviceD3D9 | confirmed | Instantiates the class named "GDrawDeviceD3D9" through FUN_0053da53(...,0x18), finalises the instance with FUN_004ccf27 and forwards it to FUN_00536678 (regist… |
| `0048163d` | GDSurfaceMMXRgb888CreateFromObject | confirmed | Variant of GDSurfaceMMXRgb888Create taking an object: checks this+0x20 and the descriptor magic (0x610888 or 0x618888) at *(+0x44) plus the 0x800000 global fla… |
| `0048ac1c` | ScalarDeletingDtor | confirmed | Restores the secondary vtable at this+4, resets the primary vtable to GInterface::vftable, and frees this through the thunk only when param_2 bit 0 is set; oth… |
| `0048b120` | ScalarDeletingDtor | confirmed | Restores the secondary vtable at this+4, resets the primary vtable to GInterface::vftable, and frees this through the thunk only when param_2 bit 0 is set; oth… |
| `0048b880` | ScalarDeletingDtor | confirmed | Restores the secondary vtable at this+4, resets the primary vtable to GInterface::vftable, and frees this through the thunk only when param_2 bit 0 is set; oth… |
| `0048c1f2` | ScalarDeletingDtor | confirmed | Restores the secondary vtable at this+4, resets the primary vtable to GInterface::vftable, and frees this through the thunk only when param_2 bit 0 is set; oth… |
| `008ac460` | TargetArrayRealloc | confirmed | Reallocates a Target array with 8-byte elements. Calls the container's virtual slot +4 with this in ECX and stack arguments param_2, &local_14, param_3; the re… |
| `009fad80` | BuildPlayerDataRecord | confirmed | Creates a PlayerData record and populates named fields for player identity, position, client version, graphics, abilities, equipped items, pet, house, jobs, ef… |
| `00c69f90` | FormatItemRowFields | confirmed | Walks the item list at param_2+0x20 (records end at entry+0x18, which is the next pointer) and appends one row per entry into the wide-string sink param_3 usin… |
| `00c6a720` | MarketingDataSourceCtor | confirmed | Constructs the marketing data source and stores it in the owning member: it allocates 0x25c bytes, constructs a LastPromotionalAward source registered under th… |
| `00406607` | ParamBlockProbe | likely | Builds a 32-byte stack record (zero mask at +0, param_2 at +4, remaining fields zero, the static short DAT_01bc3b40 at +0x1C), calls virtual slot +0x110 with i… |
| `00406798` | GetWordDC | likely | Copies the 16-bit field at +0xDC of the receiver to the caller's destination. |
| `004069a9` | SetFlag4 | likely | Sets bit 2 of the dword at +0x1C and returns 1. |
| `004069b0` | ClearFlag4 | likely | When bit 2 of the dword at +0x1C is set, clears that bit and returns 1 in the low byte; otherwise returns 0 with the low byte cleared. |
| `00406a75` | CompareKeysWithFlags | likely | Boolean key comparison: when bit 1 of param_1 is clear returns true; when bit 0x100 is set returns (param_3 & param_4) != (param_2 & param_3); otherwise return… |
| `00406c2f` | GetHolderWord188 | likely | Returns the dword at +0x188 of the object held at +0x44, or 0 when that holder pointer is null; no calls and no writes. |
| `00406d23` | SetMagicPair5 | likely | Passes param_1 (a byte size) to the pooled allocator FUN_004d9207 and, when a block is returned, writes the fixed dword pair 0x56471e89 at +8 and 0x9fe1234a at… |
| `00406e34` | SetMagicPair6 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC; called from the 00410f69 factory to… |
| `00406f45` | SetMagicPair7 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC; called from the 0041101e factory to… |
| `00406fc6` | CtorZero18 | likely | Zeroes six consecutive dwords (24 bytes) starting at param_1 and returns the pointer; caller 0041e189 allocates 0x18 bytes through FUN_00d833a6 and immediately… |
| `00406fd6` | DtorFreeC9F6E | likely | Runs the cleanup routine FUN_004c9f6e on the object and frees the block with _free when the low bit of the flag byte is set; returns the pointer. |
| `0040706d` | InvokeVirtual240Region | likely | Converts param_2 into the 7-dword region record through FUN_004c9ed9, calls the receiver's virtual slot +0x240 with that record and param_3, then releases the … |
| `00407166` | InvokeVirtual238Region | likely | Converts param_2 into the 7-dword region record through FUN_004c9ed9, calls the receiver's virtual slot +0x238 with that record and param_3, then releases the … |
| `0040719d` | InvokeVirtual24CRegion | likely | Converts param_2 into the 7-dword region record through FUN_004c9ed9, calls the receiver's virtual slot +0x24C with that record and param_3, then releases the … |
| `004071d4` | GetFieldC4Record | likely | Reads the nine-dword record at +0xC4 of the receiver into param_2[0..8] when that pointer is non-null, otherwise zeroes param_2[8]; always returns 1. |
| `00407283` | AdvanceWhileSlot8Zero | likely | Iterator advance step: each pass increments the counter at +0x14, calls FUN_004c82e7 and returns 0 when it reports false, then re-reads the current global dwor… |
| `004072cb` | IsGlobalEqualToField10 | likely | Loads the current global object through FUN_004e7384(1), compares it to the dword at +0x10, and returns the loaded value when they match, otherwise 0. |
| `004072e2` | SetFlag10000AndForward | likely | Rewrites the flag argument as (param_5 & 0x70FFFF) \| 0x10000 - clearing bits 17-19 of the incoming flags and forcing bit 16 - and tail-calls FUN_0042f518 with… |
| `00407308` | CopyValue16From78 | likely | Copies the value record at this+0x78 into the caller's stack record: the object code makes the caller's record the receiver (ECX = [esp+8]) and pushes this+0x7… |
| `00407438` | GetStaticTable1AF31E0 | likely | Returns the address of the static data table at 0x1AF31E0; no reads or writes. |
| `0040743e` | CtorThreeRegions | likely | Runs the region-record initializer FUN_004c9ebe three times with receivers this, this+0x1C and this+0x38 (per the object code), producing three consecutive 0x1… |
| `0040745a` | DtorThreeRegions | likely | Releases the three region records of the 0x54-byte object through FUN_004c9f6e in the order +0x38, +0x1C, +0 (per the object code); no other work. |
| `00407475` | ScalarDeletingDtorThreeRegions | likely | Runs the three-region destructor body FUN_0040745a and frees the object with _free when the low bit of the flag byte is set. |
| `00407491` | EnsureThreeRegionCache | likely | When the field at +0xB8 is null, allocates 0x54 bytes with FUN_00d833a6 and constructs the three-region cache through FUN_0040743e; stores it at +0xB8 and retu… |
| `004074c6` | DispatchSelectedRecordMethodsA | likely | Uses the first stack argument as a selector: 0 chooses this, 1 chooses this+0x1C, and 2 chooses this+0x38; the chosen receiver gets FUN_004e8134(source,0), whi… |
| `00407527` | DispatchSelectedRecordMethodsB | likely | Uses the first stack argument as a selector: 0 chooses this, 1 chooses this+0x1C, and 2 chooses this+0x38; the chosen receiver gets FUN_004e820b(source,0), whi… |
| `00407588` | DispatchSelectedRecordMethodsC | likely | Uses the first stack argument as a selector: 0 chooses this, 1 chooses this+0x1C, and 2 chooses this+0x38; the chosen receiver gets FUN_004dbd55(source), while… |
| `004075e7` | EnsureBlockAndAssignA | likely | Ensures the three-record block exists through FUN_00407491 and, when that succeeds, runs the FUN_004074c6 record assignment on the block with (param_1,param_2)… |
| `0040760c` | EnsureBlockAndAssignB | likely | Ensures the three-record block exists through FUN_00407491 and, when that succeeds, runs the FUN_00407527 record assignment with (param_1,param_2) as its (sele… |
| `00407631` | EnsureBlockAndAssignC | likely | Ensures the three-record block exists through FUN_00407491 and, when that succeeds, runs the FUN_00407588 record assignment with (param_1,param_2) as its (sele… |
| `00407759` | SetMagicPair8 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC; called from the factory at 0043978f… |
| `004077d6` | GetGlobal352C | likely | Copies the global dword DAT_01AF352C into the caller's destination. |
| `004077fc` | SetMagicPair9 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00407b9d` | GetGlobal35DC | likely | Copies the global dword DAT_01AF35DC into the caller's destination. |
| `00407bf6` | ChainContains | likely | Null-safe walk: starts at param_2+0x24 (0 when param_2 is null), follows the +0x24 links and returns 1 when a link equals param_1, 0 when the chain ends. |
| `00407c19` | CallSlot3CIfSlot20Accepts | likely | When param_2 is non-null and its virtual slot +0x20 accepts param_3 (non-zero low byte), calls the receiver's virtual slot +0x3C(param_2,0,0) and returns its r… |
| `00407ca6` | QuerySlot88OfField20 | likely | When the pointer at +0x20 is non-null, calls virtual slot +0x88 of the object at *(*(param_1+0x20)+0xBC) and returns the byte result; returns 0 otherwise. |
| `00407cc2` | SetSlot8COnField20 | likely | When the pointer at +0x20 is non-null, forwards param_2 to virtual slot +0x8C of the object at *(*(param_1+0x20)+0xBC) and returns its byte result; returns 0 o… |
| `00407dde` | LookupTempNameA | likely | When the input string is non-empty, builds a Temp_-prefixed name: derives stack strings from the static base object at 0x175CEBC (rendered as the "Temp_" liter… |
| `00407ef9` | LookupTempNameC | likely | When the input string is non-empty, builds a "Temp_"+input name, calls the receiver's slot +0x4C to resolve it and, when an object comes back, forwards it thro… |
| `00407f86` | GetField24 | likely | Returns the dword at +0x24 of the receiver; no calls or writes. |
| `00407fa6` | GetField0C | likely | Returns the dword at +0xC of the receiver; no calls or writes. |
| `0040810b` | SetMagicPair10 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `0040817d` | ProbeFlagE4AndFlush | likely | Returns 0 unless the object at +0x24 exists and bit 4 of its +0xEC flags is set; otherwise snapshots through FUN_004d93dc, sets the byte at +0x18C of that obje… |
| `0040823c` | SetMagicPair11 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00408304` | SetFlag16C | likely | Stores 1 at the dword +0x16C of the receiver. |
| `00408311` | GetFieldEC | likely | Returns the dword at +0xEC of the receiver; no calls and no writes. |
| `00408375` | MapModeToIndex | likely | Switch on param_1: 2 and 3 map to 1, 4..0xB map to 0xC..0x13 respectively, everything else maps to 0. |
| `0040840d` | GetGlobal3BC4 | likely | Copies the global dword DAT_01AF3BC4 into the caller's destination. |
| `00408433` | SetMagicPair12 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `004085f6` | TrySetPairAt4 | likely | Guarded two-dword setter: when (flags byte at +0 & 0x11) == 0 it copies param_2[0]/param_2[1] to +4/+8 and returns 1 in AL (upper EAX bits are leftovers of the… |
| `00408684` | GetGlobal3F0C | likely | Copies the global dword DAT_01AF3F0C into the caller's destination. |
| `004086aa` | SetMagicPair13 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00408742` | GetGlobal406C | likely | Copies the global dword DAT_01AF406C into the caller's destination. |
| `00408816` | SetMagicPair14 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `004088ae` | SetMagicPair15 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00408c34` | SetMask30000000 | likely | Replaces bits 29-30 of the dword at +0x14 with the same bits of param_2, leaving all other bits unchanged. |
| `00408ec5` | QueryVirtual94 | likely | Zeroes a stack dword, calls the receiver's virtual slot +0x94 with its address, and returns the (possibly updated) dword. |
| `00408f35` | InvokeVirtual90Tag8 | likely | Builds a stack argument record whose vtable is 0x175d184 with an initialized region container from FUN_004c9ebe, tag field 8 and the two dwords of param_2; cal… |
| `00408f87` | InvokeVirtual90Tag3D | likely | Builds the same stack argument record (vtable 0x175d184, region container, tag 0x3d) filled through FUN_0047bbc4(param_2), calls the receiver's slot +0x90 with… |
| `00408fd6` | InvokeVirtual90Tag10 | likely | Builds the stack argument record (vtable 0x175d184, region container, tag 0x10) with the element structure copied through FUN_004dbd55(param_2), calls the rece… |
| `004091ce` | SetMagicPair16 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00409283` | GetStaticPair45A4 | likely | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF45A4 through param_2. |
| `0040929c` | SetMagicPair17 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00409338` | GetStaticPair470C | likely | When param_3 is non-null, stores param_1 through it, and stores the global dword DAT_01AF470C through param_2. |
| `00409351` | SetMagicPair18 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00409455` | SetMagicPair19 | likely | Passes param_1 (a byte size) to FUN_004d9207 and, when a block is returned, writes 0x56471e89 at +8 and 0x9fe1234a at +0xC. |
| `00409576` | ResolveLayoutStretchFlags | likely | Resolves the effective layout flags of a control from its own +0x10 word and its parent's: queries the receiver slot +0x30 and slot +0x3C, folds horizontal/ver… |
| `00409711` | FindFlaggedParentField | likely | Walks the +0x18 parent links while bit 0 of the byte at +0x14 is clear; returns the +0x18 field of the first node whose bit 0 is set, or 0 when the chain ends. |
| `00409723` | GetGlobal4D1C | likely | Copies the global dword DAT_01AF4D1C into the caller's destination. |
| `00409749` | AssignLayoutCtrl | likely | Queries the argument object's name through its slot +0x1C; when that name is empty, or when the receiver's slot +0x6C(name,1) is non-zero, builds the literal n… |
| `00409805` | GetGlobal4E9C | likely | Copies the global dword DAT_01AF4E9C into the caller's destination. |
| `00409831` | InvokeVirtual28Mask | likely | Calls the receiver's virtual slot +0x28 with (param_2, 0xC000000). |
| `00409883` | InvokeVirtual74Flags | likely | Calls the receiver's virtual slot +0x74(param_2, flags, param_3) where flags = 0x80000 when byte 0 of param_3 is set and 0x800000 when byte 1 is set (combined … |
| `00409906` | ResolveInheritedAlignment | likely | Takes bits 0x0C000000 of the +0x10 word; when zero, walks the +0x18 parent chain through FUN_00409711 (thunk at 00409706) and finally queries the found object'… |
| `004099ab` | SetFiveFields | likely | Stores param_2/param_3/param_4 at +8/+0xC/+0x10 and param_5/param_6 as bytes at +0x14/+0x15. |
| `00409b2a` | GetGlobal519C | likely | Copies the global dword DAT_01AF519C into the caller's destination. |
| `00409b56` | CheckBothEntryPairs | likely | If all six dwords of the three argument-record pairs are equal, runs AdjustEntryCounters on the first components (+0x64/+0x6C, counters +0x74/+0x7C) and, only … |
| `0040dd5e` | RectUnionInPlace | likely | In-place rectangle union (bytes: cmp/jg-pair then mov per edge): left = min(*param_1, param_2), top = min(param_1[1], param_3), right = max(param_1[2], param_4… |
| `0040de33` | RectTranslateInverse | likely | Builds a rectangle in param_1 by subtracting the 2D offset (dx = *param_3, dy = param_3[1]) from the rectangle in param_2: left = x1-dx, top = y1-dy, right = x… |
| `0040defc` | QueryVirtual18 | likely | Zeroes a 4-byte local, calls virtual slot +0x18 on the receiving object with (param_2, this, &local) and returns whatever the slot wrote into the local. |
| `0040df18` | QueryVirtual18 | likely | Second emission of the virtual-query wrapper: zeroes the local, calls slot +0x18 with (param_2, this, &local) and returns the local. |
| `0040df34` | InvokeVirtual1C | likely | Calls virtual slot +0x1c on the receiving object with (param_2, this, &param_3) and returns nothing. |
| `0040e2be` | ScalarDeletingDtor | likely | Scalar deleting destructor: calls the shared GInterface-derived teardown FUN_00526562, then frees the object through the engine heap free thunk_FUN_004c6a61 wh… |
| `0040e327` | InvokeWithNegatedPair | likely | Calls FUN_004ca149 with the negated pair of the two dwords pointed to by param_2 (-x, -y) and returns the first argument (the object pointer). |
| `0040e6bf` | InvokeSlot1OnField0C | likely | Calls the second vtable slot (+4) of the object at this+0xc with (this+4, param_2). |
| `0040e6d2` | Call5017F8WithThisAndTwoArgs | likely | Pushes the two stack arguments and this pointer, calls FUN_005017f8, removes 12 bytes of call arguments, and returns with ret 8. |
| `0040e872` | DtorReleaseField3C | likely | Destructor body: releases the interface pointer held at +0x3c through vtable slot +0x10 with a zero argument, then runs the GInterface-derived teardown FUN_005… |
| `0040e8fe` | GetField304IfNegative | likely | Returns 0 when the dword at +0x174 is non-negative, otherwise returns the dword at +0x304 (a guarded field getter on a large control object). |
| `0040ec40` | ForwardToFieldC0 | likely | Returns 0 when the interface pointer at +0xc0 is null, otherwise forwards (this, param_2) to that object's virtual slot +0x1c and returns its result. |
| `0040ee03` | GetOwnerStateValue | likely | Recursive owner-state getter: when the slot at +0xbc is null it accepts the parent at +0x44 only if the link matches (+0x48 == parent+0xbc, parent state 2), re… |
| `0040ee59` | GetOwnerPointerOrForward | likely | When the parent at +0x44 is in state 2 (field +0x11c) it add-refs the interface pointer at parent+0x138 through slot +0xc when non-null and returns that pointe… |
| `0040eeb3` | GetOwnerPointOrForward | likely | Returns a 2-dword point through the out parameter: when the parent at +0x44 is in state 2 it copies the point from the interface at parent+0x138 (virtual slot … |
| `0040efe8` | ForwardWithOwnerSync | likely | When the owner interface at +0xd4 exists and its slot +0x11c reports bit 3 while the boolean argument is set, it calls the owner's slot +0x118 and, if that fai… |
| `0040f1b3` | ForwardPointToOwner | likely | When the owner interface at +0xd4 exists it forwards the two dwords of param_2 to the owner's virtual slot 0x134 together with this, then forwards the point to… |
| `0040f1e3` | ForwardCodeToOwner | likely | When the owner interface at +0xd4 exists and the 16-bit argument matches either of the two global codes (0x1bc37a4 or 0x1bc2c8c) it notifies the owner through … |
| `0040f224` | DtorVtables175D4C8 | likely | Destructor body of the class built around the vtable pair 0175d4c8/0175d4c4: writes those two vtable pointers, releases the interface pointer at +0x24 (dword i… |
| `0040f289` | ScalarDeletingDtor | likely | Scalar deleting destructor: runs the destructor body FUN_0040f224 and frees the object through the engine heap free thunk_FUN_004c6a61 when the flag's low bit … |
| `0040f332` | FindControlByHandle | likely | Scans the global object list (count FUN_00409c73, element accessor FUN_00409c86) for an entry whose 16-bit field at +0x40 equals the global type code DAT_01bc3… |
| `0040f51e` | FindAndRequestByGlobalKey | likely | Looks up the object registered under the shared global key DAT_01bc36cc through the manager at +0x24 (manager slot +0x18) and, when it is found and its virtual… |
| `0040f567` | HandleCodeViaResolvedObject | likely | When param_2 is 0 it first calls its own virtual slot 0x21c with 0; it then resolves a target through FUN_0040ee03(0) and returns 0 when there is none; for par… |
| `0040f6b2` | GetScreenRect | likely | Writes the control's rectangle: left/top from +0x68/+0x6c, right = left + width-1 (width at +0x70), bottom = top + height-1 (height at +0x74); when the flag wo… |
| `0040f874` | GetInnerRect | likely | Writes a four-dword rectangle: x = +0x94, y = +0x98, right = (width at +0x70 - insetX) - x, bottom = (height at +0x74 - insetY) - y, where insetX = max(0, (fie… |
| `0040f8df` | GetOwnerField170 | likely | Returns the dword at parent+0x170 when the parent pointer at +0x44 exists, else 0 (a cheap owner-state getter with 23 call sites). |
| `0040f8f0` | GetOwnerInterface160 | likely | Returns the pointer stored at owner+0x160 when the parent pointer at +0x44 is non-null, and 0 when it is null; the emitted conditional-release branch cannot ru… |
| `0040f92e` | ForwardOwnerFlag | likely | When the owner pointer at +0x44 exists it calls FUN_00408304 (which sets the dword at +0x16c of the passed object to 1) and returns its byte result, else retur… |
| `0040f941` | GetOwnerInterface178 | likely | Returns the pointer stored at owner+0x178 when the parent pointer at +0x44 is non-null, and 0 when it is null; the emitted conditional-release branch cannot ru… |
| `0040f97f` | AttachToOwner178 | likely | Attaches this control to its owner's slot at +0x178; returns 0 when the parent pointer at +0x44 is null. Otherwise it add-refs the current occupant through slo… |
| `0040fa82` | DetachFromOwner178 | likely | Detaches this control from owner+0x178: when the owner's slot still points at this object it releases the stored reference (slot +0x10), clears the slot and no… |
| `0040faf8` | GetOwnerInterface164 | likely | Returns the pointer stored at owner+0x164 when the parent pointer at +0x44 is non-null, and 0 when it is null; the emitted conditional-release branch cannot ru… |
| `0040fb36` | GetOverrideOrResolveByKey | likely | Returns the override interface stored at +0xe0 when it is non-null (the field that 00419ad8/00419a70 install); otherwise it performs three keyed lookups throug… |
| `0040fbad` | LookupManagerObjectByKey | likely | Thin keyed-lookup helper (bytes: 55 8b ec 51 83 65 fc 00 8d 55 fc 83 c1 24 8b 01 52 51 ff 75 08 ff 50 18 ... = zero a 4-byte local, then call manager slot +0x1… |
| `0040fbcc` | SetShownFlag | likely | Sets or clears bit 0x10000000 of the +0x64 flag word (returning immediately when the bit is already in the requested state), then re-reads the word and sends t… |
| `0040fc51` | SetOwnerInterface | likely | Attaches or replaces the owner interface at +0xd4: when the old occupant accepts slot +0x118 but refuses slot +0x114 the change is rejected (returns 0); otherw… |
| `0040fd06` | GetOwnerInterfaceField | likely | Seven-byte field getter (bytes: 8b 81 d4 00 00 00 c3 = mov eax,[ecx+0xd4]; ret): returns the interface pointer stored at this+0xd4, the slot the attach routine… |
| `0040fd0d` | QueryOwnerSlot10C | likely | Queries the resolved owner (FUN_0040ee03 with force=1) through its virtual slot 0x10c; when no owner exists it falls back to the object at +0x20 and calls slot… |
| `00410161` | ControlBaseCtor | likely | Constructs the base object: writes the two vtable pointers (PTR_LAB_0175c44c first, then PTR_LAB_0175d3a0) and copies the two dwords at +4/+8 from the source o… |
| `00419961` | ScalePairWithDisplayRatio | likely | Guarded geometry helper: returns param_3/param_4 as defaults unless the display host at +0x44 has state 2 at +0x11C and the getters FUN_0040EDC6, FUN_0040EE03(… |
| `00419a70` | SetStateShortAndNotify | likely | Setter for the 16-bit state at +0xDC: when the value differs or a stale override exists at +0xE0 it releases that override through virtual slot +0x10 with argu… |
| `00419ad8` | SetOverrideInterfaceAndNotify | likely | Overrides the interface pointer at +0xE0: when the incoming pointer differs from the stored one or the short at +0xDC is non-zero it AddRefs the new interface … |
| `00419e37` | InvokeRenderAFromTarget | likely | Wrapper that obtains the render buffer from the target object's virtual slots: calls param_3 slot +0x34 with (0,0); on success invokes FUN_004104A3(param_1,par… |
| `00419f3a` | InvokeRenderC | likely | Seven-argument render/state wrapper: requires param_6 non-null and its +0x44 object non-null with index 9 non-zero, then calls FUN_00410733(param_2,param_3,par… |
| `00419fc8` | InvokeRenderCFromTarget | likely | Variant of the seven-argument wrapper that acquires the buffer from the target's virtual slot +0x34(0,0), calls FUN_00410733(param_1,param_2,param_3,param_4,bu… |
| `0041a016` | HasHostHandle184 | likely | Boolean wrapper: calls FUN_00410E38 (whose accepted label GetMember184Pointer reads the dword at +0x184 of the object referenced by this+0x44, or 0 when +0x44 … |
| `0041d53b` | SetScreenPosition | likely | Positions the object's display holder: when the display host at +0x44 exists it resolves the holder and its local origin through FUN_00412B06, adds the caller'… |
| `0041d5df` | GetLocalPositionFromHolder | likely | Screen-to-local position: when the display host exists it resolves the holder via FUN_00412B06, queries the holder's rect through slot +0x80, converts the pair… |
| `0041e189` | RefreshHeldHandles | likely | Resolves the provider with FUN_00419C2B and allocates a zeroed 0x18-byte handle block at +0xE4 when missing. Acquires the primary handle through slot +0x98 whe… |
| `004206c2` | RoutePointerEvent | likely | Returns 0 when param_5 or the owner screen at param_1[0x11] is null, when FUN_0041a016 or FUN_0040faf8 returns non-zero, or when FUN_0040f97f(0) fails and FUN_… |
| `00420a42` | BlitLayeredImages | likely | For each of the two image handles held by the object at param_1[0x39] (+8 and +0xc, size at +0x10) it selects that handle on the renderer interface param_2 (sl… |
| `00420d27` | Ops72Byte | likely | Opcode-dispatched lifecycle table for a 72-byte object allocated through FUN_00408433(0x48): op 0 initialises in place with FUN_00413d0f, op 1 copy-initialises… |
| `00459484` | MapFormatIdToPackedFlags | likely | Maps an image-format enum (0x14..0x1A, 0x1E) to packed format constants (0x410888, 0x618888, 0x610888, 0x210565, 0x210555, 0x211555, 0x214444, 0x210444) and wr… |
| `004597cd` | InitFlashFxDescriptor | likely | Calls the shared field copy 00459784(dest,src) and then copies the source's +0x24 field into +0x18. |
| `0045998b` | BufferIsFree | likely | Returns 1 when the descriptor holds a block at +8 whose first dword (+4 of that block) is zero (block not in use), else 0. |
| `00459c31` | QuadFromRect | likely | Fills a seven-dword quad record: *p=param_2, p[1]=param_3, p[2]=0, p[3]=1.0f (0x3f800000), p[4]=*param_4, p[5]=param_5, p[6]=param_6. |
| `00459c69` | RectToQuad | likely | From a source rect (left/top/right/bottom at +0/+4/+8/+0xC) writes a quad record: copies the first two fields, computes width/height as inclusive differences (… |
| `00459cb1` | PushVariantRef | likely | Writes a two-dword variant record {2, param_2} and, when param_2 and its +0x24 field are non-null, increments the reference count at (param_2+0x24)+8. |
| `00459ccf` | ReleaseVariantRef | likely | When the pointer at +4 is non-null and its +0x24 sub-object exists it releases that sub-object through FUN_00459af8 (which decrements its +8 counter); +4 is cl… |
| `00459ced` | SpaceFits3 | likely | Checks three space spans (dword at +4 minus +8, at +0x14 minus +0x18 and at +0x20 minus +0x24) against the three requested amounts; returns 1 when all fit, els… |
| `00459d1b` | SpaceFits2 | likely | Two-span variant of the space check: compares the available space at +4-+8 and +0x14-+0x18 against param_2/param_3 and returns 1 when both fit, else 0. |
| `00459e9e` | IntersectRect2D | likely | 2D rect intersection: when the two rects overlap (four bound tests) it writes max-left, max-top, min-right and min-bottom into param_1[0..3], copies param_2[4]… |
| `00459f0e` | IntersectRect2D_2 | likely | Tests inclusive overlap of param_2 and param_3 bounds; on hit writes max-left, max-top, min-right and min-bottom to param_1[0..3], copies param_2[4] to param_1… |
| `00459f7e` | IntersectRotatedRect | likely | Intersects two rotated rect records and, on overlap, clips the origin fields upward while adjusting the quarter-turn bytes at +0x1C/+0x1D modulo 16; returns 1 … |
| `0045a7c8` | AlignedRecordSize | likely | Returns the aligned size of a record: align4(+0x14) + 4 + align4(+0x10). |
| `0045a7df` | RecordInit1of9 | likely | Initialises a nine-dword record: stores param_2 at +0 and zeroes the eight following dwords. |
| `0045a826` | FreelistPush16 | likely | Pool push with capacity 0x10: when the count at +0x10 is below 0x10 it links param_2 after the free-list head at +0xC (clearing +0x14/+0x10 of the node) and in… |
| `0045a863` | FreelistPush3 | likely | Pool push with capacity 3: when the count at +0x18 is below 3 it pushes param_2 onto the free list at +0x14 (storing the old head inside the node) and incremen… |
| `0045aabd` | SetCurrentTexture | likely | When the requested texture differs from the bound one (+0x20) it stores it, sets the dirty flag +0x7A, propagates the record's surface flags (bytes +0x20/+0x21… |
| `0045abc6` | QueueDrawQuad | likely | When the flag +0x74 is set and a texture is bound, builds a quad record from param_2 through FUN_00459c69 and submits it with the device's virtual slot +0xBC, … |
| `0045ac08` | QueueDrawStoredQuad | likely | When a texture is bound, submits the stored quad record (bound texture + 8) through the device's virtual slot +0xBC, increments the quad counter and clears the… |
| `0045ac34` | DrawEnter | likely | Increments the draw nesting counter at +0x7C and returns 1 in the low byte of the result. |
| `0045ac3e` | DrawLeave | likely | Decrements the nesting counter at +0x7C and, when it reaches zero, submits the stored quad through FUN_0045ac08. |
| `0045ac51` | FlushStateBlockA | likely | When the flag at +0x34 is set it releases the interface at +0x24 through its slot +0x30, adds param_2 to the accumulator at +0x30, clears +0x28 and the flag. |
| `0045ac7a` | FlushStateBlockB | likely | Second flush path: when the flag at +0x48 is set it releases the interface at +0x38 through its slot +0x30, adds the caller's count (recovered from the return … |
| `0045ac9f` | DrawIndexedPrimitive | likely | When a texture is bound and param_6 equals 1 it submits through the device's virtual slot +0x148 with (4,0,param_4,param_5,param_2,param_3), incrementing the p… |
| `0045b6b1` | CopyCtorVtbl17647c4 | likely | Initialises the receiver with the base table 0x175c44c, copies the two dwords at +4/+8 from the source and installs the table 0x17647c4. |
| `0045b725` | SetStaticTag01af6394 | likely | Stores the static dword from data address 0x1af6394 into the receiver's first field (static table/type tag). |
| `0045b9c6` | D3D9ModuleSubobjectCopyCtor | likely | Installs table 0x175c44c, copies source fields at +4 and +8 into the destination, then installs table 0x176487c. |
| `0045bafc` | InitPairZero | likely | Zeroes the two dwords at +0 and +4 of the receiver. |
| `0045bbfa` | ProbeCapsThenCallSlot28 | likely | Requires capability masks 0x80000, 0x337, 0x3000300 and 0x104e, plus both dimensions above 1. Then pushes 0x15, 3, 0, [EBP-4], 1, 0 and the interface at this+0… |
| `0045bcb3` | CopyCtorVtbl17648b0 | likely | Third construction-from-source emission: base table 0x175c44c, copies +4/+8 from the source and installs the table 0x17648b0. |
| `0045bd90` | LookupIdMap | likely | Linear search of the static table at 0x1af6508 (records of three dwords, key = first short) for param_1 and returns the associated dword; returns 0 when the id… |
| `0045be7d` | CopyCtorVtbl17648b4 | likely | Fourth construction-from-source emission: base table 0x175c44c, copies +4/+8 from the source and installs the table 0x17648b4. |
| `0045bec7` | SetStaticTag01af687c | likely | Stores the static dword from data address 0x1af687c into the receiver's first field (static table/type tag). |
| `0045f753` | SetSurfaceFrame | likely | Initialises a frame/bitmap object from a surface interface and an optional sub-rect: queries the descriptor through surface virtual slot +0x44 (desc words at l… |
| `0045f84e` | SetFrameBitmap | likely | Guarded frame assignment: returns 0 unless the frame's active flag (+0x11) is set and the frame's own virtual slot +0x154 reports positive width/height; with a… |
| `0045f90f` | CopyFrameStateWithDevice | likely | Copies frame state from a source frame object: builds a device/context probe via FUN_00459393 plus FUN_004cd10a, optionally calls the source's convert entry th… |
| `0045f9a4` | CopyFrameState | likely | Copies frame state from param_2: invalidates this frame through virtual slot +0x128 when its +0x11 flag is set, applies the source's rect (fields +0x68..+0x74)… |
| `0045ff4a` | ClipGeomRecord | likely | Clips a 0x3c-byte control-geometry record (param_2) to a target rect (param_3) and writes the result to param_1: a full containment short-circuits with a plain… |
| `004600c2` | PackGeomBig | likely | Serializes a full geometry record (up to 15 dwords) into the compact byte form: it bumps a refcount through param_3[0xe]+8, then picks a record-type tag in *pa… |
| `00460459` | UnpackGeomBig | likely | Parses the compact geometry record produced by 004600c2 back into the full in-memory form: switch on the type byte (0..6) that scales/expands 16-bit or 32-bit … |
| `004606d0` | PackGeomSmall | likely | Packs a 5-dword small geometry record: when all four coordinates fit in signed 16-bit it writes type 1 and 0xc bytes of 16-bit coordinate pairs plus the image … |
| `00460761` | UnpackGeomSmall | likely | Parses the small geometry record: type 0 expands the 4-dword rect (0x14 bytes consumed), type 1 expands the packed 16-bit coordinate quads back to dwords (0xc … |
| `004607c5` | PackGeomScaled | likely | Packs a 7-dword scaled geometry record: coordinates that fit 16-bit are written as shorts with base tags 4 (equal endpoints, 0xc bytes) or 2 (0x10 bytes); othe… |
| `00460885` | UnpackGeomScaled | likely | Parses the scaled geometry record by type tag 0..5: tags 2..5 expand the 16-bit coordinate pairs with the image id words (0xc..0x18 bytes consumed, scaled flag… |
| `0046093b` | PackGeomExt | likely | Packs an 8-dword extended geometry record: 16-bit-fitting coordinates are written as shorts with tag 4 (equal endpoints, 0x14 bytes) or tag 2 (0x18 bytes), oth… |
| `00460a45` | UpdateCachedRecord | likely | Maintains a cached 0x98-byte record slab: with *param_2 == 0 it releases the cached block (virtual slot +8 of the holder at param_1[1]) and clears the two head… |
| `00461ffa` | BufferReserve | likely | Reserve helper of a growable two-buffer arena: fails (returns 0) when (count+1)*size plus both used lengths exceeds 0xfff0; otherwise it grows the first buffer… |
| `004620ae` | ScalarDeletingDtor_004C9F6E | likely | Deleting destructor: runs the FUN_004c9f6e cleanup chain, then frees the object with _free when the flag's low bit is set, returning the object pointer. |
| `004620f9` | CachePushOrFree | likely | Cache push for the 0x80-entry object cache: below the +0x20 count limit it runs FUN_004ca065(0), links the incoming object into the list at +0x1c through the o… |
| `00463806` | DeviceModeDispatch | likely | Dispatches display-object modes. Mode 0 hashes the string GDSGeneric and creates a 0x18-byte registry instance through 0053da53 and the GDS helpers. Mode 2 rel… |
| `00467770` | ExpandPackedPixelTo32Bit | likely | Expands a packed source pixel for the target surface: fetches it through vtable slot +0xe0 and obtains the format id by calling slot +0x30 on the object at *(*… |
| `004689c4` | AddQuadClip60 | likely | From a 0x10-byte element vector or the caller record, appends 0x3c-byte draw records to the list owned by this: it optionally offsets a local 0x3c-byte copy of… |
| `00468d1c` | AddQuadBounds60 | likely | Appends 0x3c-byte draw records like 004689c4 (fast path copies with FUN_00465135 and appends with FUN_004652aa; the element vector path converts with FUN_0045f… |
| `00468ebf` | AddQuadStreamBounds60 | likely | Stream counterpart of 00468d1c: expands the packed entries of param_2 with FUN_00460459, clips with FUN_0045ff4a and appends with FUN_004652aa (or forwards the… |
| `0046923b` | AddQuadStreamClip20A | likely | Stream counterpart for the 20-byte record layout: param_5 == 1 forwards to FUN_004653d8, otherwise each packed entry of param_2 is expanded with FUN_00460761, … |
| `004695b9` | AddQuadStreamBounds20A | likely | Stream counterpart of 0046941f: entries expanded with FUN_00460761, clipped with FUN_00459e9e and appended with FUN_00465395, or forwarded wholesale to FUN_004… |
| `0046a8c0` | SetBlitSource | likely | Swaps the source held at this+8: clears the cached index at this+0x2c when it equals the current source, releases the current source with FUN_004620f9 when par… |
| `0046b665` | AppendRecordWithOptionalRectIntersection | likely | Copies the 28-byte record from param_2 into a local and offsets its four coordinate fields by param_4[0] and param_4[1]. Calls 0x0046556b to append/encode that… |
| `0046d1a3` | AppendBackGuardedShim | likely | Builds two checked iterators for the container at +4/+8 (FUN_00464f1b on the words at +8 and +4) and passes the two iterator pairs to FUN_0046aff5 (AppendSlotB… |
| `0046d1de` | VectorBackPointerC | likely | Same end()-minus-one accessor: FUN_0046484d builds the iterator, FUN_0046b1d0 steps it backwards, FUN_0045d591 dereferences it. |
| `0046d296` | AppendBackGuardedShim_2 | likely | Same shape as AppendBackGuardedShim with the second iterator constructor FUN_00464882 and the second guarded back-insertion FUN_0046b193 (AppendSlotBackGuarded… |
| `0046fe00` | BuildIteratorPairAndCall | likely | Constructs checked iterator records for the values at +8 and +4 using this object as their container, then passes the records and endpoint fields to 0046e08a; … |
| `00477fef` | DrawQueueSubmitRecord | likely | Calls adapter slot +4 to fill a local rectangle and offsets it by this+0x90/+0x94. It sets the predicate result to 1 when +0x98 is null, otherwise calls FUN_00… |
| `004781b0` | DispatchRecordHandler | likely | Loads a handler pointer from DAT_01bc45cc using the record kind, builds a 12-byte view from blob-base plus record offset and length with FUN_00459906, and opti… |
| `00479514` | DrawQueueFlushSurface | likely | Runs only when state +0x14 is 4, or is 1 with +0x24 nonzero, and the queue at +0x44 exists. If the +0x4c/+0x50 vector count is nonzero, it constructs and deref… |
| `004795f3` | SurfaceTransferResult | likely | When the required surface members are present, offsets the requested coordinates by the bound object's +0x90/+0x94 values and checks FUN_004f7eb9. It flushes v… |
| `00479860` | D3D9FrameBufferFactory | likely | Class-factory dispatcher for the 200-byte frame-buffer class: mode 0 constructs in place with 00477c01, mode 1 copy-constructs in place with 00477b31, modes 2/… |
| `004798f1` | D3D9FrameBufferScalarDeletingDtor | likely | Scalar deleting destructor for the frame-buffer class: runs the destructor 00477c64 and, when the flag argument's low bit is set, frees the object through thun… |
| `00479c05` | DrawQueueDtor | likely | Destructor body of the record-list class: reinstalls its two vtables (017652b4/017652b0), calls the free-reset FUN_00477d48 with flag 1, then runs the member t… |
| `00479c56` | DrawQueueReplayTable | likely | Returns 1 immediately when byte +0x36 is clear. Otherwise it prepares the segment cursor and passes the replay guard. With byte +0x34 clear it processes the co… |
| `00479e49` | DrawDeviceCreateDefaultFrameBuffer | likely | Queries the owner through virtual slot +0x44 into a 39-byte local buffer, then calls 004784cd. If the first status has low bit 0 and the owner query succeeded,… |
| `00479eb9` | DrawSurfaceDtor | likely | Reinstalls the object's two vtable pointers. When the +0x14 field is 1 and the +0x30 container is nonempty, iterates the container rooted through +0x2c: entrie… |
| `0047a0a4` | DrawSurfaceSuspend | likely | Suspends the surface: clears the not-ready bit 0x100, and when the active bit 4 was set runs FUN_0046fa3f, flushes the pending records through FUN_00479514, ca… |
| `0047a0f8` | DrawSurfaceEmitFlashRecord | likely | When (+0x10 & 0xc) is nonzero, builds a local GFlashFx descriptor with kind 6 and payload param_3, initializes it, and obtains a record adapter. It copies four… |
| `0047a1bb` | CommitListsAndClearBit3 | likely | Returns 0 when +0x48 is null. Otherwise compiles that object through 0047990d, clears its byte +0x37, replaces members +0x44 and +0x48 using the two back-point… |
| `0047a4e9` | DrawSurfaceEmitTiledFillRecord | likely | Emits one fill record, or a tiled grid of fill records when the style flags param_7 carry bit 0x4000/0x8000: computes the tile counts by dividing the pixel off… |
| `0047a734` | DrawSurfaceEmitFillTable | likely | Increments the nested counter at +0xa4, then walks param_3 entries at a 0x20-byte stride. For each positive-width and positive-height entry, uses entry+0x1c as… |
| `0047a7de` | DrawSurfaceEmitSpanRecord | likely | Emits a line/span record for the rectangle between (param_2,param_3) and (param_4,param_5): normalises the bounds, picks the cached adapter at +0x1c4 or +0x1c8… |
| `0047afc5` | DrawQueueScalarDeletingDtor | likely | Scalar deleting destructor for the record-list class: runs the destructor 00479c05 and, when the flag argument's low bit is set, frees the object through thunk… |
| `0047b7e1` | ColorBytesToFloatTriplet | likely | Converts three consecutive colour bytes (param_1[2], param_1[1], param_1[0]) into three floats scaled by the constant at DAT_01768b08, writing them to *param_2… |
| `0047bc43` | InitClassToken_01af6cd4 | likely | Writes the class token global DAT_01af6cd4 into the first dword of the supplied object, the initialisation step used by the bitmap classes before the object is… |
| `0047bd0e` | CreateBitmapFromResourceHandle | likely | Looks up a resource through FUN_0052f167(param_1,0x21,0x180), prepares a local object-token slot through 0047bc43 and obtains context from FUN_004cd10a. The ra… |
| `0047bd74` | ConditionalStrideBytes | likely | Calls FUN_004c6e81 (a stub that returns 0), then reads the format word at the object referenced by +0x44. If the word is nonzero and lacks bit 0x02000000, it a… |
| `0047bf1b` | InitClassToken_01af6d84 | likely | Writes the class token global DAT_01af6d84 into the first dword of the supplied object. |
| `0047c314` | GetPixelChannel | likely | Masks the format word to 24 bits. Formats 8, 0x40008 and 0x80008 return the low byte; 0x410888/0xc18888 return 24 bits, and 0x610888/0x618888 return the full d… |
| `0047c460` | SetPixelChannel | likely | Masks the format word to 24 bits. Formats 8, 0x40008 and 0x80008 store one byte; 0x410888/0xc18888 store three bytes, and 0x610888/0x618888 store a dword. Reco… |
| `0047c5df` | SetPixelAlpha | likely | Sets or clears the alpha component of a pixel for the three formats that carry alpha: for 0x211555 it sets/clears bit 15, for 0x214444 it masks the value to 0x… |
| `0047c79a` | SetPixelChannel_2 | likely | Second code copy of the channel setter at 0047c460: stores direct byte/word/dword values for recognized formats, mask-merges selected 12-/15-bit formats, and u… |
| `0047c985` | DerivePixelFormatFromGdiObject | likely | Determines the GDS pixel format for a GDI bitmap/dib object: releases the previous buffer through virtual slot +0x128 when present, calls GetObjectA with the 0… |
| `004813b4` | DtorVtables1769430 | likely | Destructor: installs vtables 01769430/0176942c, releases the member interface at +0x1c through its virtual slot 0x10, then sets the secondary slot to the base … |
| `0048158e` | GDSurfaceMMXRgb888Create | likely | If this+0x20 is unused and the descriptor/global-flag checks pass, calls FUN_004e5340 with "GDSurfaceMMXRgb888", then invokes virtual slot +8 with (DAT_01bc7e2… |
| `00481833` | ScalarDeletingDtorA | likely | Scalar deleting destructor: runs the teardown helper FUN_004508ba and, when bit 0 of the flag is set, frees the object with the CRT free. |
| `00489cc0` | ReadBitmapPropertyValue | likely | Checks the serializer and helper capability predicates, resolves the class descriptor named L"Bitmap", and passes it to stream virtual slot +0x58. It releases … |
| `00489d28` | WriteBitmapPropertyValue | likely | Property serialisation callback mirroring 00489cc0: requires FUN_004c6e70 and the helper slot +0x28 check to pass, then when the member at this+0x30 is non-nul… |
| `0048ab97` | CtorVtbl1769E64 | likely | Constructor of a 16-byte framework object: installs &PTR_LAB_01768e68 as the first vtable slot, runs the base initialiser FUN_004c9e10, then slots in the secon… |
| `0048b064` | CtorVtbl1769E8C | likely | Installs the initial vtable, calls 004c9e10, then installs secondary vtable values at this+4 and the primary vtable at this. It returns this. The related copy … |
| `0048b0d1` | CopyCtorVtbl1769E8C | likely | Installs the initial vtable, passes either null or param_2+4 to 0047e61f depending on whether param_2 is null, then installs secondary and primary vtable value… |
| `0048b7c4` | CtorVtbl1769EB4 | likely | Installs the initial vtable, calls 004c9e10, then installs secondary vtable values at this+4 and primary vtable at this; returns this. |
| `0048b831` | CopyCtorVtbl1769EB4 | likely | Installs the initial vtable, copies the secondary base from param_2+4 through 0047ea73, installs the final secondary and primary vtable values, and returns thi… |
| `0048c136` | CtorVtbl1769EDC | likely | Installs the initial vtable, calls 004c9e10, then installs secondary vtable values at this+4 and primary vtable at this; returns this. |
| `0048c1a3` | CopyCtorVtbl1769EDC | likely | Installs the initial vtable, copies the secondary base from param_2+4 through 0047eb83, installs final secondary and primary vtable values, and returns this. |
| `00496799` | MapPixelInPlace | likely | Converts one packed pixel in place under the format descriptor param_1 (field 0 = format word, field 5 = bytes per pixel, field [0xb] = palette object): it dec… |
| `004974be` | ResizeSurfaceStorage | likely | Rebuilds the draw-surface object's bitmap storage for a new extent (virtual method in vtable 01769ba0 slot +0x130): it constructs a stack temporary of the same… |
| `0049767b` | CropSurfaceToRect | likely | Rect-scoped variant of the storage rebuild at 004974be (virtual method in vtable 01769ba0 slot +0x134): param_2 points at four ints (x0, y0, x1, y1) and param_… |
| `0049787f` | ExpandPixelValueToRGB | likely | Value-level decoder for one pixel value of a surface's own format (virtual method in vtable 01769ba0 slot +0x178): it reads the descriptor pointer at this+0x44… |
| `00497a74` | MapColorChannels | likely | Free helper that combines two 32-bit colours per the packed channel map param_4: it byte-permutes param_3 into the source channels and param_2 into the keep ch… |
| `00497b48` | GetPixelRGB | likely | Reads the pixel at (param_3 = x, param_4 = y) from the descriptor at *(this+0x44) (virtual method in vtable 01769ba0 slot +0x198) and writes it as a 32-bit col… |
| `00497eb6` | GetPixelRGBA | likely | Alpha-aware pixel reader for the descriptor at *(this+0x44) (virtual method in vtable 01769ba0 slot +0x1A0): bounds-checks x (param_3) and y (param_4) against … |
| `00498603` | GetPixelAlpha | likely | Returns one alpha value for the pixel at (param_2 = x, param_3 = y) of the descriptor at *(this+0x44) (virtual method in vtable 01769ba0 slot +0x1A8): null des… |
| `00498a07` | SetPixelColorMapped | likely | Writes one pixel of the descriptor at *(this+0x44) (virtual method in vtable 01769ba0 slot +0x190): returns 0 for a null descriptor, a null pixel base (field [… |
| `009931e0` | InitializeCollectionRewardDataSource | likely | Stores param_2 at the first dword, clears a 200-byte member region, then constructs a reward child data source and two other child sources. |
| `00bdd010` | NotifyThreeDataSources | likely | Calls virtual slot 4 (vtable offset 0x10) on the object at this+4 and on the object at this+8, then tail-jumps to the same slot of the object at this+0xc (the … |
| `00bdd4f0` | ProgressiveQuestDataSources_Ctor | likely | Constructor of the object that owns the progressive-quest data sources: stores the owner argument at +0, zeroes the fields at +4..+0x10, then allocates three 0… |
| `00bdd600` | ProgressiveQuestDataSources_Dtor | likely | Destructor body: when the DAT_01be1090 guard is set it releases the three refcounted pointers at +4, +8 and +0xc (virtual slot 0x28 on each followed by FUN_008… |
| `00c69380` | Wrapper_00c69380 | likely | Calls FUN_0105fef0(param_2) first, then virtual slot +4 of the sub-object at *(this+4)+0x254, then virtual slot +0x10 of the object at *(this+4). Both indirect… |
| `00c693e0` | RefreshCachedCount_00c693e0 | likely | Calls virtual slot +0xc on the sub-object at this+0x34, then resolves this+0x258 (the owning data table), follows its +0x1c4 field and, when that is non-null, … |
| `00c69af0` | ClaimSystemDataSourcesCtor | likely | Builds the claim system's two data sources and stores them in the two owning members: it allocates 0x260 bytes and constructs an AvailableItems source named Ba… |
| `00c69f40` | RefreshCachedCount_00c69f40 | likely | Takes the object pointer from *param_1, clears its cached count at +0x258, then follows obj+0x254 to the data manager, reads manager+0x15c and, when that is no… |
| `00c6a160` | FormatFieldFromOwner_00c6a160 | likely | Calls FUN_0082ecd0(this,&DAT_017703e0,*param_2) - filling this from the value the owner pointer points to, using the %d-shaped literal at DAT_017703e0 - and th… |
| `00cb5520` | SetValueAndNotify | likely | When the supplied value differs from field +0x258, stores it, sets the dirty byte at +0x26c, and invokes the object's virtual method at vtable offset 0x10. |

### `lib.soegems`

**SoeGems::ArrayPooled<unsigned_int>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c8d80` | ReserveStorage | confirmed | Virtual slot 1: with flag param_4 == 0 it grows the requested count to 5/4 of param_2 when this+0xc is smaller, or shrinks it to 6/5 when this+0xc exceeds 4/3 … |
| `010c8e50` | ReleaseStorage | confirmed | Virtual slot 2: if this+0x10 is 0 and the one-time flag DAT_01cb9261 is clear, runs the SoeGems singleton init 010818b0 and sets the flag; then loads the pool … |

**SoeGems::ObjectFactory<IRequirement,int>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbe800` | ObjectFactory::ScalarDeletingDtor | confirmed | Scalar deleting destructor (bytes checked, 30 bytes): calls the object factory destructor body 00bbe340 on ECX, then if bit 0 of the flag argument is set calls… |

**SoeGems::ObjectFactory<IRequirement,int>::BasePool** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5e80` | BasePoolDtorBody | confirmed | Clears the pool block tree of the object through 00bc3ab0 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc5ee0` | BasePoolDtorBody | confirmed | Clears the pool block tree of the object through 00bc3b80 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc5f40` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc3c50 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc5fa0` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc3d20 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc6000` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc3df0 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc6060` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc3ec0 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc60c0` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc3f90 (ClearBlockTree), then stores the BasePool vtable at +0. |
| `00bc6120` | BasePoolDtorBody | confirmed | Clears the pool block tree through 00bc4060 (ClearBlockTree), then stores the BasePool vtable at +0. |

**SoeGems::ObjectFactory<IRequirement,int>::ICreator** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbdc20` | ICreator::ScalarDeletingDtor | confirmed | Stores the ICreator vftable (01825d08) into *this, then if bit 0 of the flag argument is set calls 00401c00 (FreeMemory) with (this, 0). Returns this. The SEH … |
| `00bbdc80` | ICreator::ScalarDeletingDtor | confirmed | Same body as 00bbdc20: stores the ICreator vftable (01825d08) into *this, then if flag bit 0 is set calls 00401c00 (FreeMemory) with (this, 0); returns this. |
| `00bbdce0` | ICreator::ScalarDeletingDtor | confirmed | Stores the ICreator vftable (01825d08) into *this; if bit 0 of the flag argument is set, pushes 0 and this and calls 00401c00 (FreeMemory); returns this in EAX… |

**SoeGems::ObjectFactory<IRequirement,int>::InternalRegister<SoeUtil::ObjectPool<ClientRequirementAlwaysFails,0>,ClientRequirementAlwaysFails>::__l2::MyCreator** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc7a40` | CreateFromPool | likely | Virtual slot 1 creator: allocates 0x60 bytes when the pool free head is empty and constructs ClientRequirementAlwaysFails (00bc4590), otherwise calls 00bc4e60;… |

**SoeGems::ObjectFactory<IRequirement,int>::InternalRegister<SoeUtil::ObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>,ClientRequirementCheckLastPlayedProfileOfTypeRank>::__l2::MyCreator** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc7970` | CreateFromPool | likely | Virtual slot 1 creator: allocates 0x60 bytes when the pool free head is empty and constructs ClientRequirementCheckLastPlayedProfileOfTypeRank (00bc44d0), othe… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementAlwaysFails,SoeUtil::ObjectPool<ClientRequirementAlwaysFails,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5b10` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementAlwaysFails,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10 and th… |
| `00bc5ba0` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCountry,SoeUtil::ObjectPool<ClientRequirementCheckCountry,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5040` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckCountry,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10 and t… |
| `00bc5530` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object (virtual deleting destructor with flag 0, which destroys without freeing), then pushes the node back to the pool thr… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCurrentProfile,SoeUtil::ObjectPool<ClientRequirementCheckCurrentProfile,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5c70` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckCurrentProfile,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x1… |
| `00bc5d00` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLanguage,SoeUtil::ObjectPool<ClientRequirementCheckLanguage,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5550` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckLanguage,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10 and … |
| `00bc55e0` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc54a0. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank,SoeUtil::ObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5600` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank,...> vtable at +0, zeroes the words at +4, +… |
| `00bc5af0` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckNudge,SoeUtil::ObjectPool<ClientRequirementCheckNudge,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5dd0` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckNudge,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10 and the… |
| `00bc5e60` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckRenamePending,SoeUtil::ObjectPool<ClientRequirementCheckRenamePending,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5d20` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckRenamePending,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10… |
| `00bc5db0` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckVipRank,SoeUtil::ObjectPool<ClientRequirementCheckVipRank,0>_>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc5bc0` | Ctor | confirmed | Constructor: stores the BasePool vtable and then the PoolWrapper<ClientRequirementCheckVipRank,...> vtable at +0, zeroes the words at +4, +8, +0xc, +0x10 and t… |
| `00bc5c50` | ReleaseToPool | confirmed | Virtual slot 1: calls slot 0 of the object with flag 0, then pushes the node back to the pool through 00bc5a60. |

**SoeGems::TextIndex<int>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b93f70` | TextIndex::BuildWeightedResults | confirmed | Tokenizes a query, finds matching terms, creates weighted leaf providers, merges them through a provider heap, then stores result scores and updates weighted-t… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b93020` | RebuildDocumentIdLookup | likely | Builds a temporary 0x4000-bucket unsigned-key map from linked document rows, assigns sequential row IDs, rewrites IDs in tree-held records from the map, then c… |
| `00b93210` | OwnedStringArrayDeletingDtor | likely | Calls cleanup for the owned string and inverted-instance array, frees this only when flag bit 0 is set, and returns this otherwise. |
| `00b93230` | ConstructStringKeyedEntry | likely | Allocates an entry through the object's virtual allocator, constructs its string and instance-array members, then inserts it into a string-keyed tree. |
| `00b932b0` | AppendHashedOccurrence | likely | Allocates a node, links it at the ordered-list tail and in the bucket selected by key masked with 0x3fff, then increments the map count. |
| `00b93800` | DestroyDocumentSubtree | likely | Recursively visits child links at offsets 0x38 and 0x3c, destroys each node's string and instance array, then invokes the container's virtual node-release slot. |
| `00b93840` | InsertTokenOccurrence | likely | Normalizes the supplied token, finds or creates its string-keyed entry, appends supplied row values to an entry array, and when enabled adds the token occurren… |
| `00b93950` | ComputeNormalizedCountScore | likely | Walks records, increments a frequency count indexed by each record key, accumulates its value divided by that count, and returns the accumulator scaled by the … |
| `00b93d00` | ClearDocumentTree | likely | If root at offset 4 is nonzero, recursively destroys both child subtrees and each node's string and instance array, invokes the node-release slot, then zeros r… |
| `00b93d70` | IndexDocumentTokens | likely | Assigns the next document ID, inserts the document key, tokenizes text into filtered fixed-width words, and for each nonempty token calls the index insertion r… |
| `00b93ef0` | TextIndexReset | likely | Resets the document ID counter, destroys the document tree and bucket map, clears the document hash list, and removes remaining linked entries. |

### `lib.soegems.refarray`

**SoeGems::RefArrayPooled<unsigned_char>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010df8c0` | RefArrayPooledCtorAppend | confirmed | Stores the vtable chain SoeUtil::Array<unsigned_char,0,1>, SoeUtil::RefCounted, SoeUtil::RefCountedImplicit, then SoeGems::RefArrayPooled<unsigned_char> (final… |
| `0107fd20` | RefArrayPooledUcharCtor | likely | Constructor body: calls 0083fe90 on this, installs the SoeGems::RefArrayPooled<unsigned_char> vftable at this+0x0 and at this+0x10 (second base), and clears th… |
| `01080060` | RefArrayPooledSecondaryDtorThunk | likely | Virtual slot 0 of the secondary vtable: this-adjusting thunk (bytes: sub ecx,0x10; jmp 01080970). Adjusts the receiver by -0x10 and jumps to 01080970, which is… |
| `01080590` | RefArrayPooledDtorBody | likely | Non-deleting destructor body: stores the SoeGems::RefArrayPooled<unsigned_char> vftable at this+0x0 and at this+0x10, clears the size dword at +0x8, calls 0107… |
| `01080970` | RefArrayPooledScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor of the primary vftable: calls the destructor body 01080590 on this, then frees this with _free when flag bit 0 of the… |
| `01081b60` | RefArrayPooledFreeBuffer | likely | Virtual slot 2 (args: buffer, size, flag): when the flag bit 0 is set frees the buffer with 00401c00 (FreeMemory alignment 1); otherwise initialises the single… |

## Source shards

[[S0001 00401040-00406b84|S0001]] (35), [[S0002 00406c2f-0040afca|S0002]] (113), [[S0003 0040affb-004103c5|S0003]] (73), [[S0004 00410405-004149c8|S0004]] (1), [[S0006 00419961-0041e544|S0006]] (17), [[S0007 0041e58b-00422a66|S0007]] (6), [[S0008 00422dcc-00426a69|S0008]] (3), [[S0018 004574f6-0045cca3|S0018]] (37), [[S0019 0045ccf9-00462140|S0019]] (20), [[S0020 004621ca-0046734c|S0020]] (1), [[S0021 00467368-0046b665|S0021]] (33), [[S0022 0046b77b-0046efeb|S0022]] (6), [[S0023 0046f261-00472f3a|S0023]] (4), [[S0025 00477947-0047ca0c|S0025]] (72), [[S0026 0047ca5e-004823b9|S0026]] (6), [[S0028 0048741e-0048c4f5|S0028]] (14), [[S0031 00496799-00498a07|S0031]] (9), [[S0149 00719de0-0071efe0|S0149]] (10), [[S0151|S0151]] (7), [[S0165|S0165]] (2), [[S0168|S0168]] (5), [[S0170|S0170]] (5), [[S0171|S0171]] (1), [[S0172 00798490-0079d890|S0172]] (5), [[S0177|S0177]] (27), [[S0179|S0179]] (2), [[S0184|S0184]] (48), [[S0185|S0185]] (11), [[S0186|S0186]] (27), [[S0192 00805a50-0080a790|S0192]] (34), [[S0200|S0200]] (3), [[S0202 0083ea10-00844c00|S0202]] (4), [[S0219 0089d990-008a3c80|S0219]] (4), [[S0221 008a86b0-008b1d80|S0221]] (15), [[S0223 008b7790-008bcf40|S0223]] (3), [[S0224 008bcf60-008c5290|S0224]] (3), [[S0226 008ca9e0-008d08b0|S0226]] (16), [[S0227 008d0920-008d5c60|S0227]] (2), [[S0236|S0236]] (4), [[S0239 0090a890-009105b0|S0239]] (31), [[S0240 009105e0-009161d0|S0240]] (26), [[S0242 0091af50-009200d0|S0242]] (1), [[S0243 00920130-00925ec0|S0243]] (55), [[S0244 00925fe0-0092b110|S0244]] (48), [[S0248 009392c0-0093d740|S0248]] (17), [[S0249 0093daf0-00942c60|S0249]] (17), [[S0250 00942c80-00947180|S0250]] (18), [[S0257|S0257]] (2), [[S0263 00982f10-009897a0|S0263]] (8), [[S0264 00989810-0098eae0|S0264]] (16), [[S0265 0098ec00-00995660|S0265]] (2), [[S0275 009c8e90-009cdee0|S0275]] (7), [[S0276 009cdf80-009d3fe0|S0276]] (4), [[S0284 009f8470-009feec0|S0284]] (2), [[S0294 00a35150-00a39a80|S0294]] (1), [[S0300 00a59220-00a5da20|S0300]] (4), [[S0316 00aae6e0-00ab2c20|S0316]] (1), [[S0325 00adee60-00ae4e70|S0325]] (29), [[S0326 00ae4fa0-00ae9b00|S0326]] (36), [[S0334 00b0a370-00b10840|S0334]] (15), [[S0336 00b150e0-00b1b340|S0336]] (1), [[S0349 00b5ffe0-00b66a80|S0349]] (1), [[S0352 00b6cd80-00b71320|S0352]] (1), [[S0353 00b71490-00b77eb0|S0353]] (2), [[S0357 00b84e90-00b8a5f0|S0357]] (12), [[S0359 00b90750-00b94a30|S0359]] (12), [[S0368|S0368]] (5), [[S0369|S0369]] (32), [[S0374 00bd9880-00be0090|S0374]] (3), [[S0394|S0394]] (25), [[S0399 00c68550-00c6de90|S0399]] (38), [[S0405 00c89e70-00c8f7c0|S0405]] (2), [[S0410 00ca6401-00cab4e0|S0410]] (12), [[S0412 00cb0bf0-00cb6b80|S0412]] (4), [[S0419 00cd5e90-00cdb1a0|S0419]] (1), [[S0421 00ce0340-00ce4c30|S0421]] (8), [[S0423 00ceb9b0-00cf13e0|S0423]] (28), [[S0438 00d2f8c0-00d34430|S0438]] (6), [[S0439|S0439]] (27), [[S0443 00d49d90-00d4f150|S0443]] (26), [[S0542|S0542]] (4), [[S0552|S0552]] (1), [[S0565 01025bf0-0102b850|S0565]] (9), [[S0571 01044480-0104a2b0|S0571]] (37), [[S0572|S0572]] (17), [[S0573|S0573]] (9), [[S0575|S0575]] (3), [[S0580 0107d0c0-010829c0|S0580]] (8), [[S0582|S0582]] (22), [[S0583|S0583]] (21), [[S0584 01093ee0-01099b80|S0584]] (34), [[S0585 01099be0-0109ea50|S0585]] (58), [[S0586|S0586]] (137), [[S0587|S0587]] (12), [[S0588|S0588]] (1), [[S0590|S0590]] (13), [[S0591|S0591]] (38), [[S0592|S0592]] (6), [[S0594|S0594]] (2), [[S0597|S0597]] (1), [[S0602|S0602]] (1), [[S0652|S0652]] (2), [[S0653|S0653]] (4), [[S0654 012393a0-0123f010|S0654]] (8), [[S0674|S0674]] (6), [[S0675|S0675]] (21), [[S0676|S0676]] (30), [[S0689|S0689]] (4), [[S0690|S0690]] (1), [[S0727 013be8e0-013c3b90|S0727]] (5), [[S0728|S0728]] (13), [[S0740|S0740]] (1), [[S0790|S0790]] (16), [[S0797|S0797]] (9)
