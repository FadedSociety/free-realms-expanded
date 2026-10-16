# System: game.item

103 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (29)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00720bf0` | `IconData::IconData` | IconData | Constructor of the IconData pair used by item definitions: image set id and tint id. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00720cd0` | `IconData::Print` | IconData | Debug dump of one IconData record into the client log stream. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007218e0` | `ItemDefinitionManager::ItemDefinitionManager` | ItemDefinitionManager | Base-subobject constructor that turns a raw block into a full ItemDefinitionManager (called from the ClientAccountDataManager constructor). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722720` | `BaseDefinitionManager::ScalarDeletingDestructor` | BaseDefinitionManager | MSVC scalar deleting destructor for BaseDefinitionManager. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00722750` | `ItemDefinitionManager::ScalarDeletingDestructor` | ItemDefinitionManager | MSVC scalar deleting destructor for the item-definition manager. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00723040` | `BaseItemDefinition::~BaseItemDefinition` | BaseItemDefinition | Destructor of the shared item-definition base (three strings plus its Loggable subobject). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007230f0` | `BaseItemDefinition::BaseItemDefinition` | BaseItemDefinition | Constructor of the shared item-definition base record (icon data, three strings, numeric defaults). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724820` | `ItemDefinitionStyleCard::ItemDefinitionStyleCard` | ItemDefinitionStyleCard | Constructor of the style-card item definition variant (used when a creator style card row is parsed). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724960` | `ItemDefinitionCustomization::ItemDefinitionCustomization` | ItemDefinitionCustomization | Constructor of the customization item definition variant parsed for type 0x11. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724b80` | `LoadItemDefinitions` |  | Loads the master ItemDefinitions.txt table that feeds every item lookup in the client. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `008b1e00` | `ItemRecord::Field4PositiveAndField8NonNegative` | ItemRecord | Shared ItemRecord predicate, also installed in derived item vtables; field meanings beyond this exact test are unknown. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b1e20` | `ClientItem::~ClientItem` | ClientItem | Destructor of the client-side item object; it is the non-deleting half of the dtor pair whose deleting half is 008b1e70, and it chains into the ItemInstance/ItemRecord base cleanup at 0103dd90. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008b1e70` | `ClientItem::ScalarDeletingDestructor` | ClientItem | The scalar deleting destructor paired with ClientItem::~ClientItem (008b1e20); it is the vtable slot the client's packet/entity teardown path uses when an item object must be destroyed and its storage released. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `009ff220` | `ItemRecord::~ItemRecord` | ItemRecord | Non-deleting ItemRecord cleanup used by compiler-generated unwind funclets; this body only restores the vtable. | [[Native Map/Shards/S0285 009fef30-00a059e0|S0285]] |
| `00a3c290` | `ItemDefinition_OnReceived` |  | The single entry point through which received item definitions enter the client cache and the pending-download bookkeeping: the four log strings name the three states (received, overwritten, pending request satisfied) a… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3d6e0` | `ClientItemDefinitionManager::ClientItemDefinitionManager` | ClientItemDefinitionManager | Start-up of the client's item/definition subsystem: it creates the definition cache, the pending-request tree area, the item-group data sources and the definition manager's helpers; the PlayClient initialiser 0092b170 c… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3d870` | `ClientItemDefinitionManager::FindDefinitionById` | ClientItemDefinitionManager | Virtual lookup at ClientItemDefinitionManager vtable slot 0; its body is a cache-chain search, not a deleting-destructor shape. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3d8a0` | `ClientItemDefinitionManager::~ClientItemDefinitionManager` | ClientItemDefinitionManager | Full teardown of the item-definition manager; the compiler-generated scalar delete 008a61c0 calls this body before _free, and each member is destroyed in reverse construction order. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3ddd0` | `ItemDefinition_RequestById` | ClientItemDefinitionManager | Client-side demand path for item definitions: definitions are not pushed for everything, the client asks by id, and this is the function that decides between cached, already-requested and new-request cases. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00bf6d60` | `ProxiedCharacterAttachmentGroup::Ctor` | ProxiedCharacterAttachmentGroup | Constructs a ProxiedCharacterAttachmentGroup (callers 0095d780, 00964ba0 and 010c9ed0 are outside this shard). The List<int,-1> at +0x17c is the list that ListIntContainsAt180 walks. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf75d0` | `ProxiedCharacterAttachmentGroup::CheckDuplicateSlots` | ProxiedCharacterAttachmentGroup | Rejects an attachment group in which two items use the same equipment slot. Reports via the log only. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00c2f490` | `ParseQuotedWideToken` |  | Tokenizes text for the client-side item search index. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2fa70` | `BaseItemSearchEngineDtor` | BaseItemSearchEngine | Tears down BaseItemSearchEngine-owned search state. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c30040` | `BaseItemSearchEngineCtor` | BaseItemSearchEngine | Initializes client-side item-search state. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c30130` | `BaseItemSearchEngineScalarDeletingDtor` | BaseItemSearchEngine | Scalar deleting destructor for BaseItemSearchEngine. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c707e0` | `ItemRecord::CopyCtor` | ItemRecord | Copy-constructs an ItemRecord, used by coin-store code 00c712f0 when it duplicates a record. | [[Native Map/Shards/S0400 00c6df10-00c73b60|S0400]] |
| `00c8b7c0` | `RecipeComponentData::Copy` | RecipeComponentData | Copies one recipe component record and its nested list. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `00c952f0` | `HashListMapScalarDeletingDtor → SoeUtil::HashListMap<int,BaseItemDefinitionData…` |  | Implement scalar-deleting destruction for the pooled item-entry map. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `01045c10` | `ItemRecord::FormatDebugString` | ItemRecord | Virtual slot 2 of ItemRecord that formats a debug text for the record (definition id and tint) into the string argument. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (33)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00720d50` | `BaseDefinitionManager::~BaseDefinitionManager` | BaseDefinitionManager | Base-class destructor restoring the BaseDefinitionManager vtable during unwinding of definition objects. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00720d60` | `ItemDefinitionManager::~ItemDefinitionManager` | ItemDefinitionManager | Destructor body of ItemDefinitionManager, tearing down its two vtable slots before the BaseDefinitionManager base. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007217b0` | `CallItemStatDefinitionDtor` |  | Base-subobject destructor trampoline for an unnamed class deriving from (or containing) ItemStatDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007225e0` | `ScalarDeletingDtor_ItemStatDerived` |  | MSVC scalar deleting destructor of an unnamed class whose base is ItemStatDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007227c0` | `ItemDefinitionManager::RemoveAll` | ItemDefinitionManager | Destroys every loaded item definition and empties the definition manager list and hash. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007230d0` | `BaseItemDefinition_ScalarDeletingDtor` | BaseItemDefinition | MSVC scalar deleting destructor for the shared item-definition base. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007233b0` | `DefinitionList_Append` |  | Inserts one freshly parsed item definition into the manager list and id hash (called by the ItemDefinitions.txt loader). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724800` | `ScalarDeletingDtor_ItemDefinition` |  | MSVC scalar deleting destructor of a full ItemDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007248b0` | `ItemDefinitionDtorCall` |  | Base-subobject destructor trampoline for a class deriving from ItemDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724900` | `ScalarDeletingDtor_ItemDefinition_B → ItemDefinitionStyleCard::ScalarDeletingDe…` |  | Second scalar deleting destructor variant over ItemDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007249d0` | `ItemDefinitionDtorCall_B` |  | Base-subobject destructor trampoline over ItemDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724a20` | `ScalarDeletingDtor_ItemDefinition_C → ItemDefinitionCustomization::ScalarDeleti…` |  | Third scalar deleting destructor variant over ItemDefinition. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724a80` | `NewItemDefinition` |  | Definition factory used by the ItemDefinitions.txt loader: picks the concrete definition class from the row type column. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00990d80` | `CollectMatchingItemCandidates` |  | Collects matching candidates for the item-search path. ItemSearchEngine constructor/destructor calls show that comparison state is temporary; the exact matching policy remains unknown. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `00a15690` | `ItemDefinition_EvaluateUsability` |  | Outer usability gate that decides whether the detailed slot check 00a142c0 is worth running, and returns its 1-4 status to the callers (00a156f0 and eight others). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a1ec00` | `ProcessItemRecordListActions` |  | Dispatches item-record actions supplied by the store-bundle path 00b38ea0 and claim path 00a041b0. The client sends several action types here; this code does not establish how the original server answered them. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a3ac00` | `ItemDefinitionCache_Construct` |  | Constructor of the 1024-bucket cache whose single-node unlink helpers are 00a3b000 (list links +0xb8/+0xbc) and 00a3b070 (bucket chain +0xb0, key +0xb4); the binding rests on that identical bucket layout and on the item… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b000` | `ItemDefinitionCache_UnlinkFromList` |  | List-unlink half of the item-definition cache removal path; the item-definition receive handler 00a3c290 calls it right before 00a3b070 unlinks the same node from its bucket. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b070` | `ItemDefinitionCache_UnlinkFromBucket` |  | Bucket-unlink half of the item-definition cache removal path, called by the same handler 00a3c290 as the list unlink; 0x3ff confirms the 1024-bucket table that 00a3ac00 constructs. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b390` | `MemberInit_ZeroFields0_44` |  | Initialises one sub-object of the item-definition manager while the manager constructor 00a3d6e0 runs; the manager binding comes from the caller, the body itself is generic zero-init. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b410` | `FindListedRecordValueById` |  | The direct caller is the Lua GetGroupIdFromItemId handler, which uses the returned +0x18 value as a number; the record type meaning comes from that caller, not this routine. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b460` | `ListUnlinkAll` |  | Bulk list teardown used by the item-group file parsers 00a3d350 and 00a3d2f0 when they discard or rebuild their record lists. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3b530` | `ItemDefinitionCache_RemoveAll` |  | Cache teardown for the item-definition manager (called from its method 00a3d8a0); the two unlink shapes prove it operates on the same 1024-bucket cache that 00a3ac00 constructs. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3c1f0` | `ItemDefinitionCache_InsertNode` |  | Insert step of the item-definition cache receive path: the definition handler 00a3c290 adds every received definition node here, mirroring the unlink helpers 00a3b000/00a3b070. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3db70` | `ItemDefinitions_ReadBatchFromReader` |  | Receive path that turns a PlayerUpdatePacketItemDefinitions payload into ClientItemDefinition objects and feeds each into the definition cache; the PlayClient player-update dispatcher 0092f460 calls it. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00bf6a60` | `ListIntContainsAt180` |  | Membership test on the List<int,-1> at +0x180 of the attachment group; used by ProxiedCharacterAttachmentGroupManager::ProcessGroup (00bf1530). | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00c2e940` | `SumMatchedValuesCapped` |  | Calculates a capped contribution in search scoring; exact field semantics unknown. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2eaf0` | `AppendSearchToken` |  | Appends a token-like record for BaseItemSearchEngine indexing. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2f420` | `InitSearchResultState` |  | Initializes search-result state; owning class is not identified here. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2fb00` | `BuildNormalizedTokenList` |  | Builds lowercase search tokens for BaseItemSearchEngine. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2fc80` | `AccumulateMatchedFieldScores` |  | Accumulates text matches in item-search results. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c30450` | `RebuildItemSearchIndex` |  | Refreshes client-side item search against a source snapshot. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c75210` | `DerivedHashList4096Ctor` |  | Object whose base is the 4096-bucket ItemDefinitionMetaData list. The derived class is not identified here. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |

## 🟡 speculative (26)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00724660` | `GetField_214 ? → ItemDefinition::GetField_214` |  | Returns the dword field at this+0x214. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724670` | `GetField_218 ?` |  | Returns the dword field at this+0x218. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724680` | `GetField_21c ? → ItemDefinition::GetField_21c` |  | Returns the dword field at this+0x21c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724690` | `GetField_220 ?` |  | Returns the dword field at this+0x220. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007246a0` | `GetField_224 ?` |  | Returns the dword field at this+0x224. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007246b0` | `GetField_228 ? → ItemDefinition::GetField_228` |  | Returns the dword field at this+0x228. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007246c0` | `GetField_22c ?` |  | Returns the dword field at this+0x22c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007246d0` | `GetField_230 ?` |  | Returns the dword field at this+0x230. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007246e0` | `GetField_c8 ?` |  | Returns the dword field at this+0xc8. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724780` | `SetField_214 ? → ItemDefinition::SetField_214` |  | Stores param_2 into the dword field at this+0x214. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724790` | `SetField_218 ?` |  | Stores param_2 into the dword field at this+0x218. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247a0` | `SetField_21c ?` |  | Stores param_2 into the dword field at this+0x21c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247b0` | `SetField_220 ? → ItemDefinition::SetField_220` |  | Stores param_2 into the dword field at this+0x220. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247c0` | `SetField_224 ?` |  | Stores param_2 into the dword field at this+0x224. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247d0` | `SetField_228 ?` |  | Stores param_2 into the dword field at this+0x228. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247e0` | `SetField_22c ?` |  | Stores param_2 into the dword field at this+0x22c. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007247f0` | `SetField_230 ?` |  | Stores param_2 into the dword field at this+0x230. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724880` | `ReturnArg ?` |  | Returns its only argument unchanged (a 3-byte body: mov eax,[esp+4]; ret). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00724890` | `GetField_214_B ? → ItemDefinitionStyleCard::GetField_214` |  | Returns the dword field at this+0x214; duplicate body of 00724660 for a second vtable. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007248a0` | `GetField_218_B ?` |  | Returns the dword field at this+0x218; duplicate body of 00724670 for a second vtable. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `007249c0` | `ReturnArg_B ?` |  | Returns its only argument unchanged; duplicate 3-byte stub of 00724880. | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00a156f0` | `Client_EvaluateCurrentItem ? ⚠downgrade` |  | Entry point used by the item UI path (caller 00cb1500) to evaluate the item currently held. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a3df00` | `LookupOrRequestItemDefinition ? ⚠rename` |  | Convenience entry used by callers such as 00c09220 that want either the definition or a download kicked off. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00b2a4d0` | `CheckItemRecordConditions ?` | ItemRecord | Evaluates whether an item record passes a set of runtime checks; exact return-code meanings remain unknown. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00baec20` | `ItemEquipRequirements_BuildText ? ⚠downgrade` |  | Builds the equip-requirement text block shown for an item. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bf6980` | `AttachmentGroupSlot4Forward ?` | ProxiedCharacterAttachmentGroup | Shared implementation for the three attachment group classes (named for ProxiedCharacterAttachmentGroup; the others are listed in the vtables). | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |

## ⚪ unknown (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00724300` | `— ⚠reject` | ItemDefinition | Constructor of a full item definition (stats container, ability list, name string, id/hash links). | [[Native Map/Shards/S0150 0071f030-00725ff0|S0150]] |
| `00a142c0` | `— ⚠reject` |  | Validity/equip check for one item definition, including the client-side slot table; the two log strings are the only artifacts in this shard that write an item error log. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a3c7e0` | `— ⚠reject` |  | Lazy loader/cache of the per-item-group data sources (ItemGroupItemsDataSource, allocated here at 0x264 bytes) keyed by group name, i.e. the lookup the item system uses when it needs the items of a group it has not load… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3cf00` | `— ⚠reject` |  | Loads the id-to-name mapping table that the item-group system uses, filling the string field (+0x4c) of records already present in the map at this+0x14; the caller 00a3dab0 builds the resource path. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3d2f0` | `— ⚠reject` |  | Destructor of a structure that owns both a list of records and an IString-bearing member; the item-definition manager method 00a3d8a0 and two EH unwind funclets call it. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3d350` | `— ⚠reject` |  | Parses the ItemGroups.txt table into 0xf8-byte item-group records with an embedded IString (at +0x4c); the shared loader 00a3dab0 calls it together with the mappings loader 00a3cf00. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3dab0` | `— ⚠reject` |  | Creates the client's ItemGroup.Items data source and loads both item-group tables during PlayClient initialisation (the only caller is 0092b170). | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00baeae0` | `— ⚠reject` |  | Formats one item stat value (additive or multiplicative) for display. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bf6e10` | `— ⚠reject` | ProxiedCharacterAttachmentGroup | Name-based attachment resolution pass over the group's entries. Shared by the three classes; named for ProxiedCharacterAttachmentGroup. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00bf7240` | `— ⚠reject` | ProxiedCharacterAttachmentGroup | Validates attachment slot population of a group and logs failures. Log-only: it reports, it does not repair. | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |
| `00c2f080` | `— ⚠reject` |  | Computes and orders BaseItemSearchEngine search scores on the client. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c30150` | `— ⚠reject` |  | Builds client-side searchable terms from profile/catalogue fields. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c30a70` | `— ⚠reject` |  | Likely client-side item ranking, inferred from the field-score and candidate-reordering helpers it calls. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c95510` | `— ⚠reject` |  | Destroy pooled item-entry map subobject state. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `0107f4e0` | `— ⚠reject` | ClientItemFilterManager | Loads the item filter definitions, option table and option map for the client item filter manager from ItemFilters.txt, ItemFilterOptions.txt and ItemFilterOptionMap.txt. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a3ddd0` | `ItemDefinition_RequestById` | send | PlayerUpdatePacketItemDefinitionRequest |  | Hypothesis: the original server answered each PlayerUpdatePacketItemDefinitionRequest (35/34) with the definition rows for that id in a PlayerUpdatePacketItemDefinitions payload, and the client dropped duplicate requests while one was in flight, as the two log strings describe. Depends on 008a1100 (request constructor), 00a3dca0 (tunnel sender) and 00a3db70 (batch reader). |
| `00a1ec00` | `ProcessItemRecordListActions` | send | InventoryPacketEquipByItemRecord |  |  |
| `00a1ec00` | `ProcessItemRecordListActions` | send | InventoryPacketUseStyleCardByItemRecord |  |  |
| `00a1ec00` | `ProcessItemRecordListActions` | send | InventoryPacketItemActionBarAssign |  |  |
| `00a1ec00` | `ProcessItemRecordListActions` | send | ClientHousingPacketEnterRequest |  |  |
| `00a1ec00` | `ProcessItemRecordListActions` | send | PacketMountSpawnByItemDefinitionId |  |  |
| `00a3db70` | `ItemDefinitions_ReadBatchFromReader` | recv | PlayerUpdatePacketItemDefinitions |  | Hypothesis: the original server answered item-definition requests (PlayerUpdatePacketItemDefinitionRequest, opcode 35 sub-opcode 34) with a PlayerUpdatePacketItemDefinitions payload (35/37) whose body is exactly the count plus that many serialised ClientItemDefinition rows read here, and the client then cached them through 00a3c290. Depends on the request path 00a3ddd0/00a3dca0 and on the definit… |
| `00a3df00` | `LookupOrRequestItemDefinition ? ⚠rename` | send | PlayerUpdatePacketItemDefinitionRequest |  | Hypothesis: when the local lookup misses, this path ends in the same 35/34 request the manager sends in 00a3ddd0, so a server must answer 35/34 requests with 35/37 definition batches; depends on 00a3ddd0 and 00a3dca0. |
