# System: minigame.shipcombat

185 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (30)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4fb70` | `ShipCombatProcessor_VSlot1_ReleaseAndDispatch` | ShipCombatProcessor | First virtual entry of the ship-combat processor, invoked by the engine dispatcher; it cancels a pending state and forwards to the slot-0x38 handler. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a6bcb0` | `ShipCombatProcessor::~ShipCombatProcessor` | ShipCombatProcessor | Full teardown of the processor and all its members; reached from the delete wrapper 00a6cbb0. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6cbb0` | `ShipCombatProcessor::ScalarDeletingDtor` | ShipCombatProcessor | Deleting-destructor entry (vtable slot 0) of ShipCombatProcessor. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00c99ea0` | `ShipCombatProxiedShip::DispatchSlot1` | ShipCombatProxiedShip | Forwards dispatch to slot one of a proxied ship object's vtable. | [[Native Map/Shards/S0407 00c955b0-00c9baf0|S0407]] |
| `00c9aa40` | `PirateColorFromName` |  | Maps ship combat pirate color tokens to display colors. | [[Native Map/Shards/S0407 00c955b0-00c9baf0|S0407]] |
| `00ca1440` | `ShipCombatProxiedShip::Dtor` | ShipCombatProxiedShip | Destroys a proxied ship and its owned rendering, asset, hold, and reference state. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca1d70` | `ShipCombatProxiedShip::Ctor` | ShipCombatProxiedShip | Constructs a proxied ship object with default state and asset-requester integration. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca2100` | `ShipCombatProxiedShip::DeletingDtor → ShipCombatProxiedShip::ScalarDeletingDtor` | ShipCombatProxiedShip | Implements scalar deletion for the proxied ship allocation. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca2120` | `ShipCombatProxiedShip::InitializeState` | ShipCombatProxiedShip | Initializes a proxied ship's state and associated runtime objects. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4ca0` | `ShipCombatShipProcessor::Ctor` | ShipCombatShipProcessor | Constructs the ship-combat processor and initializes its proxied ship list and sound events. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4d40` | `ShipCombatShipProcessor::Dtor` | ShipCombatShipProcessor | Releases processor-owned dependencies and all linked proxied ships. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00cabd60` | `ShipCombatProxiedProjectile::Dtor` | ShipCombatProxiedProjectile | Destroys a proxied ship-combat projectile and its owned references. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cabf50` | `ShipCombatProxiedProjectile::ScalarDeletingDtor` | ShipCombatProxiedProjectile | Implements scalar deleting destruction for proxied projectiles. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cabf70` | `DrawCannonballShadow` |  | Renders the ship-combat cannonball shadow. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac100` | `ShipCombatProxiedProjectile::Update` | ShipCombatProxiedProjectile | Updates and renders a proxied projectile each frame. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacfa0` | `ShipCombatProxiedChest::Ctor` | ShipCombatProxiedChest | Constructs a proxied ship-combat chest. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad590` | `ShipCombatProxiedChest::ScalarDeletingDtor` | ShipCombatProxiedChest | Implements scalar deleting destruction for proxied chests. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad5b0` | `ShipCombatProxiedChest::Update` | ShipCombatProxiedChest | Updates the proxied chest visual state and effects. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cadb20` | `ShipCombatProxiedFort::Dtor` | ShipCombatProxiedFort | Destroys proxied fort and its owned barrage state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cadc60` | `ShipCombatProxiedFort::Ctor` | ShipCombatProxiedFort | Constructs a proxied ship-combat fort. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cadd20` | `ShipCombatProxiedFort::ScalarDeletingDtor` | ShipCombatProxiedFort | Implements scalar deleting destruction for proxied forts. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf230` | `ShipCombatResultsDataSource::GetColumnName` | ShipCombatResultsDataSource | Defines columns exposed by the ship-combat results datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf380` | `ShipCombatResultsDataSource::GetCount` | ShipCombatResultsDataSource | Returns one count exposed by the results datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf4d0` | `InitResultsEntryString` |  | Initializes an IString field embedded in a results row. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf7e0` | `DestroyShipCombatResultsArray` |  | Destroys results-row array storage. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf810` | `ResultsArray::ScalarDeletingDtor` | SoeUtil::Array<ShipCombatResultsDataSou… | Implements scalar deleting destruction for results array. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cafa50` | `ShipCombatResultsDataSource::Dtor` | ShipCombatResultsDataSource | Destroys ship-combat results datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cafb30` | `ShipCombatResultsDataSource::Ctor` | ShipCombatResultsDataSource | Constructs the ship-combat results datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cafbc0` | `ShipCombatResultsDataSource::ScalarDeletingDtor` | ShipCombatResultsDataSource | Implements scalar deleting destruction for results datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cafbe0` | `ShipCombatResultsDataSource::GetCellValue` | ShipCombatResultsDataSource | Supplies per-cell data for ship-combat result UI rows. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## 🟢 likely (47)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4f780` | `ShipCombatShipProcessor::ScalarDeletingDtor` | ShipCombatShipProcessor | Deletes the per-ship processor object of the pirate ship-combat minigame; the class binding comes from the destructor body 00ca4d40 it invokes. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f7f0` | `ForwardFieldsToManager_330_33c` |  | Pushes the pirate-battle client's local configuration fields into the shared manager interface (same virtual-slot pattern as 00a52b20). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4fa10` | `SelectField_14_Else_8` |  | Chooses the effective value source (current object at +0x14 versus fallback object at +8) for the pirate-battle client. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a50450` | `ShipCombatProxiedFort::DtorAdapter` | ShipCombatProxiedFort | Destructor adapter for the ShipCombatProxiedFort object. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a50840` | `HashBucket_RemoveByKey360` |  | Mechanical hash-bucket unlink used by caller 00a68c10; the bucket count is read from the map rather than fixed by this function. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a50d00` | `ShipCombatProxiedProjectile::CtorAdapter` | ShipCombatProxiedProjectile | Adapts a callback call shape to the ShipCombatProxiedProjectile constructor. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a51c20` | `ListStepFromHead184` |  | List traversal helper used by callers 00a62280 and 00a68950; the node's higher-level meaning is not inferred from this function alone. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52cf0` | `CycleIndex_38c` |  | Cycles the pirate-battle target/selection index and pushes the mapped value into the active kind-9 object. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a55160` | `FindNodeByField9C_LinkE0` |  | Finds a node by its +0x9c field while following +0xe0 links; ShipCombat context comes from caller 00a5f260. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a55190` | `FindLinkedNodeByField20_Link90` |  | Finds a node by its +0x20 field while following +0x90 links; ShipCombat association is based on callers 00a5fe30 and 00a68950. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a55ce0` | `ShipCombatProxiedFortList_Append` |  | Likely appends a proxied fort to the CreateProxiedStructure variant-0 list; this association follows caller 00a68950 and initializer wrapper FUN_00a542c0. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a55d90` | `ShipCombatProxiedChestList_Insert` |  | Likely inserts a proxied chest into the CreateProxiedStructure variant-1 list: this association depends on caller 00a68950 and the initializer reached through FUN_00a54330. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a5da90` | `ClearArrays_3cc_3dc` |  | Conditional cleanup of two pointer arrays; caller 00a602a0 uses it before rebuilding them. The manager field +8 meaning is unresolved. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5e6a0` | `AppendNodeLink70` |  | Tail-append helper for the +0x70/+0x74 node list; direct callers are 00a60be0 and 00a63930. The element type is unresolved. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5ebc0` | `ReadCountedNodeList` |  | A nested list decoder used by 00a61a70 and 00a683d0. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5ed80` | `ReadUpdateGameState` |  | Body reader for the UpdateGameState branch; parent opcode dispatch is at 00a6d270. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5ee90` | `ReadUpdateShipCannons` |  | Body reader for parent ship-combat dispatch branch 0x13 at 00a6d270. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5f030` | `ReadSetPlayerCount` |  | Body reader for the parent dispatcher branch selected by sub-opcode 7. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5f120` | `ReadDestroyProxiedShip` |  | Body reader for parent dispatcher branch 10 at 00a6d270. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5f260` | `ProcessMatchedRecord` |  | Body selected by parent ship-combat dispatch case 0x1a at 00a6d270; the record meaning and helper effects are unresolved. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5f470` | `UpdateMatchedEntryTransform` |  | Packet-body handler selected by branch 0x1b in 00a6d270; the exact gameplay meaning of the transform remains unknown. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5f860` | `CreateEntryIfIdAbsent` |  | Body selected by branch 0x1c in 00a6d270. Creation also depends on the sentinel, float, and nonzero-field checks in this function. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5fd20` | `ReadShipCombatCountedValues` |  | Payload handler selected by branch 0x1e in dispatcher 00a6d270. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a5ff90` | `ReadUpdateProxiedStructure` |  | Body reader for parent dispatcher branch 0x21 at 00a6d270. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a602a0` | `BuildTransformPairArrays` |  | Builds paired transformed-handle arrays. The wider stage setup context comes from caller 00a64960; this function alone does not establish a stage identity. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a604c0` | `BuildShipFlagMetadata` |  | Setup attribution follows caller 00a64960; the direct binary anchor is the home-flag asset identifier. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a60940` | `ClearPairArray_3ec` |  | Conditional cleanup of the two-pointer entries at +0x3ec. The identity of the manager field and pointer types is unresolved. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a615b0` | `ReadCountThenPayload` |  | Used as a nested input reader by 00a62140. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a618a0` | `ReadCountedRecordNodes` |  | Nested UpdatePlayer record decoder called from 00a665e0; the two per-item helpers create and fill records rather than simply append the dword. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61940` | `ReadCountedRecordNodes` |  | Nested UpdatePlayer record decoder called from 00a665e0; the per-item helpers create and fill records rather than simply append the dword. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61a00` | `ReadCountedNodeList` |  | Nested decoder used by ReadRenderDebugPayload at 00a67350. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61a70` | `ReadProxiedStructureFields` |  | Nested field reader used by CreateProxiedStructure packet parsing at 00a669d0 and 00a68dc0. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61c30` | `ReadCountedNodeList` |  | Nested field decoder used by 00a69cf0. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61d40` | `ReadCountedFloatPayloadRecords` |  | Nested decoder used by ReadShipState at 00a66c00; the item body is twelve floats plus a counted byte region. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a61dd0` | `ShipCombatConfigMembersDtor` |  | Aggregate destructor body; no enclosing RTTI class is identified in this function. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a62030` | `ShipCombatCannonConfigDtor` | ShipCombatCannonConfig | Destructor body for ShipCombatCannonConfig elements, called by array resize/shrink operations and its scalar deleting destructor. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a62140` | `HandleCameraConfig` |  | Handler for the camera-config sub-opcode; the packet constructor and parent dispatch establish the message context, while the precise meaning of the object updates is not named. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a68950` | `ShipCombatProcessor_HandleCreateProxiedStructure` |  | Dispatcher-reached client handler for proxied structure data. The nested lookup is a linked structure/chest-list search, not an owner-player map lookup; creation and Lua notification paths follow its result and type dis… | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00ca0f50` | `UpdateHoldUiReferences` |  | Cleanup helper called by ShipCombatProxiedShip destruction; the hold-UI association is inferred from that caller. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca2470` | `ShipCombatProxiedShip::Update` | ShipCombatProxiedShip | Per-frame update for a proxied ship and its scene effects. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4510` | `ShipCombatProxiedShip::ScalarDeletingDtor` | ShipCombatProxiedShip | Implements scalar deletion for a proxied ship. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4a00` | `AddProxiedShipIfMissing` |  | Adds a proxied ship to the processor list after a successful setup check. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4ac0` | `RemoveProxiedShip` |  | Removes a proxied ship node and clears the current selection when it matches. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4ae0` | `ClearProxiedShipList` |  | Drains and destroys every node in the proxied ship list. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00cabeb0` | `ShipCombatProxiedProjectile::Ctor ⚠downgrade` | ShipCombatProxiedProjectile | Initializes a proxied ship-combat projectile. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac340` | `InitializeProxiedProjectile` |  | Initializes projectile references and optional visual effect state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad030` | `ShipCombatProxiedChest::Dtor ⚠downgrade` | ShipCombatProxiedChest | Destroys a proxied ship-combat chest. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## 🟡 speculative (59)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a529b0` | `AllDotsSameSign ? ⚠rename` |  | Geometric consistency test over five 2D points of the pirate battle (e.g. checking that a sensor/target point lies strictly on one side of the chain of four others). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52cc0` | `CallMember8WithPair ? ⚠rename` |  | Two-value dispatch into the object held at +8 (a ship-combat helper object) used by callers 00af7850/00af8120. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a5fe30` | `ReadDestroyProxiedStructure ?` |  | Likely body handler for DestroyProxiedStructure; the name and message role are supported by its constructor callee and parent dispatcher case 0x20, but the two removal branches are not identified as claim/fallback seman… | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00c9a170` | `ShipCombatProxiedShip::GetField90 ?` | ShipCombatProxiedShip | Calls a helper to retrieve a global object, then returns its 32-bit field at offset +0x90. | [[Native Map/Shards/S0407 00c955b0-00c9baf0|S0407]] |
| `00ca10b0` | `InitializeHoldUiSlots ?` |  | Computes the number of hold slots, initializes each 0x1c-byte record with configured values, zeroes two fields, and calculates a slot position before setting +0x4e1. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca13a0` | `CopyOrientationState ? ⚠rename` |  | Copies the object at +8, two counted buffers at +4 and +0x14, clears destination +0x18, then walks a linked list at +0x24 using +0x30 links. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca17d0` | `CopyProxiedShipState ?` |  | Copies 44 dwords from the source, then delegates copying of the subobjects at +0xb0 and +0x104 to helpers. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca1970` | `UpdateProxySceneObjects ?` |  | Obtains an object through a virtual call, updates its position and orientation, configures two scene objects through virtual methods, and creates or updates a third object according to flags. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca1d50` | `ApplyOrientationCopyAndSetFlag ? ⚠rename` |  | Copies orientation state from the second argument through 00ca13a0 and sets byte +0x590 on the receiver to one. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca2340` | `ApplyProxyTransform ?` |  | Copies a 16-dword transform, computes a yaw angle, updates several referenced objects and forwards the transform into another object's virtual methods. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3010` | `SegmentCrossingPoint2D ?` |  | Normalizes a 2D segment direction, tests signed side values for a crossing, computes the intersection distance, writes the intersection coordinates, and returns one or zero. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3210` | `CopyFourDwordVector ?` |  | Copies four consecutive dwords from the input into receiver offsets +0xe0 through +0xec. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca32b0` | `ReleaseSixReferences ?` |  | For six pointer/reference pairs at offsets 0x20 through 0x4c, calls the associated object's virtual release when present, decrements its reference count, and clears the pointer. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca33c0` | `AreSixReferencesReady ?` |  | Checks six referenced objects in order using virtual slot +0xc and returns one only if each object exists and reports ready. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3450` | `UpdateSixReferencedObjects ?` |  | Calls the helper at 0078fcc0 once with the delta for each non-null reference at offsets 0x24, 0x2c, 0x34, 0x3c, 0x44, and 0x4c. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3750` | `ResizeEightDwordArray ?` |  | Requests new storage through a virtual allocator, copies existing 8-dword elements into it, releases the previous buffer, and stores the new pointer and size metadata. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4070` | `ReadCurrentWorldPosition ?` |  | Reads two words at offsets +0x20 and +0x28 from the object returned by 00fedc30 and writes them to the output pair. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca46a0` | `FindLinkedEntryByTwoIds ?` |  | Starts at the node pointer at +0x10, follows +0x5d0 links, and returns the first node whose +8 and +0xc fields match both arguments. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4700` | `FindLinkedEntryByField10 ?` |  | For nonnegative input, walks the node chain from +0x10 via +0x5d0 and returns the first node whose +0x10 field matches, else null. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4730` | `CheckNearbyWorldPoint ?` |  | Copies two input words into a temporary structure and calls 00ca41a0 with the supplied point; the callee performs the distance comparison. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4770` | `AllLinkedEntriesReady ?` |  | Traverses the nodes beginning at +0x10, asks each one whether it is ready through 00c9af30, and returns false on the first failure. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca47c0` | `SetByteAndVisitList ?` |  | Stores the low byte of its second argument at receiver offset +0x1c, then invokes a fixed callback over nodes in the list at +0xc. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4960` | `InitializeOrientationElement ?` |  | Ensures an array element at the requested index, initializes its four vector words, clears three fields, sets one field to -1, and returns its address. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca4dc0` | `BuildShipOrientationCandidates ?` |  | Builds an array of direction and distance records from active linked entities and another linked entity list, transforming positions and orientations into the supplied reference frame. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca5210` | `ClassifyShipCombatContact ?` |  | Builds an orientation candidate array, compares candidates against current forward and lateral vectors, then returns a classification value and releases the temporary array. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca6134` | `ApplyUpdatePlayerStateFields ? ⚠rename` |  | Uses the supplied identifiers and values to find a linked entity, adjust counters and flags, notify dependent objects, update one nested state object, then destroy a temporary. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca7480` | `ShipCombatFrameUpdate_00ca7480 ? ⚠downgrade` |  | Per-frame ship-combat update of the pirate processor; its caller 00a6bff0 is the processor update. | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00cac170` | `LookupShipCombatActor ?` |  | Resolves an actor reference used by the projectile path. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac450` | `GetReferencedObjectPlus400 ?` |  | Calls a virtual method on the referenced object and returns its result plus 400. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac490` | `InitializeStructureState ?` |  | Stores two supplied references and initializes remaining structure state fields to zero, a sentinel, or a current default value. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac760` | `ReleaseEffectReferences ?` |  | Releases resources owned by a ship-combat effect object. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac7d0` | `CreateTimedEffect ?` |  | Creates a timed effect attached to a ship-combat actor. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac8b0` | `ReleaseAndNotifyReferencedObject ? ⚠rename` |  | Releases stored effect references and invokes virtual slot 8 on the referenced object. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacba0` | `ShipCombatProxiedStructure::CtorBase ? ⚠rename` | ShipCombatProxiedStructure | Initializes the structure base vtable during construction. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacbc0` | `StructureTransformOffset ? → ShipCombatProxiedStructure::GetTransformMatrix` |  | Returns the input pointer advanced by 0x30 bytes. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacc00` | `StructureTransformOffset2 ? ⚠rename` |  | Returns the input pointer advanced by 0x30 bytes. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caccf0` | `BuildStructureEffectParameters ?` |  | Fills a local effect parameter structure and passes it to the effect setup helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacd80` | `BuildStructureEffectParametersVariantA ?` |  | Fills a local effect parameter structure and passes it to the effect setup helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cace00` | `BuildStructureEffectParametersVariantB ?` |  | Fills a local effect parameter structure and passes it to the effect setup helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cace90` | `BuildStructureEffectParametersVariantC ?` |  | Fills a local effect parameter structure and passes it to the effect setup helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacf10` | `EnableStructureChildEffects ?` |  | When the input collection has elements and a referenced object exists, calls the collection helper with arguments 1 and 0. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad0b0` | `UpdateChestEffectsByState ?` |  | For state values 1 and 4, checks the actor-height predicate and calls the matching effect parameter helper when it passes. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad100` | `UpdateShipCombatStructureEffects ? ⚠rename` |  | Maintains time-dependent visual effects for ship-combat structures. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad420` | `TraverseAndReleaseActorEffects ?` |  | Walks actor effect references, checks flags, and releases effect objects when tracked state fails cleanup conditions. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad4f0` | `BuildStructureEffectTransform ?` |  | Gets a referenced actor value, builds a transform from global vectors and local position fields, and passes it to an effect helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad6b0` | `RefreshStructureEffectState ?` |  | Selects one actor reference, refreshes structure effects, and removes prior references when selected state changes. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cadbf0` | `CopyShipCombatRecord ?` |  | Copies ten leading words, invokes a copy helper for the embedded string, copies following fields, and copies a trailing nested value. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cadd40` | `CreateFortCannonBarrage ?` |  | Builds a fort's ship-combat cannon barrage state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cae510` | `FindCannonByKey ?` |  | Looks up cannon record by string key. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf150` | `InitializeCannonMapForActor ?` |  | Initializes cannon barrage map state for an actor. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf1f0` | `InsertCannonByStringKey ?` |  | Inserts a cannon record using its string key. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf390` | `GrowShipCombatResultsArray ? ⚠rename` |  | Chooses a new array capacity from requested size and current capacity, then returns the capacity through an output pointer. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf430` | `FreeShipCombatResultsArray ?` |  | Passes supplied storage pointer and size 8 to the allocation helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf450` | `MoveOrDestroyResultsEntry ? ⚠rename` |  | Either dispatches the entry destructor through its vtable or swaps array storage and count fields with the supplied descriptor. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf5c0` | `DestroyResultsEntryStringsFromIndex ? ⚠rename` |  | Destroys entries from computed starting index through current array count, then stores the reduced count. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf600` | `CopyShipCombatResultsEntry ?` |  | Copies entry header fields, initializes embedded IString, calls a shared copy helper for that string, and copies remaining row fields. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf7c0` | `ClearResultsDataSourceRows ?` |  | Clears all result rows from owning datasource. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf9c0` | `InitializeResultsEntryAtIndex ?` |  | Obtains or extends indexed row, initializes embedded string fields, and returns the row value. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cb0080` | `RefreshShipCombatResults ?` |  | Refreshes results datasource from participants and result state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## ⚪ unknown (49)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a4f680` | `— ⚠reject` |  | Tears down the two proxied/registered objects the pirate-battle client keeps at +0x3b0 and +0x3b8; called from the Pirates UI update 00a6c630. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f880` | `— ⚠reject` |  | Client-side asset gate for the pirate ship-combat match: the client reports the zone ready only after every sub-loader has finished. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4f9b0` | `— ⚠reject` |  | Resolves the active 'kind 9' object (the object type used by several pirate-battle helpers such as 00a4fa70 and 00a52cf0). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a4fa70` | `— ⚠reject` |  | Applies an argument to the active kind-9 object of the pirate-battle client. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a503b0` | `— ⚠reject` |  | Registered trampoline for a proxied-projectile operation in the pirate ship-combat system (the wrapper pattern is generated for static callback tables). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a50400` | `— ⚠reject` |  | Registered trampoline for a ship-combat proxied-object operation (same generated pattern as 00a503b0). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a504a0` | `— ⚠reject` |  | Registered trampoline for a proxied-chest/structure operation in the pirate ship-combat system. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a528d0` | `— ⚠reject` |  | Maps a damage/points value onto a normalised 0..1 ratio with per-phase slopes, used by the pirate ship-combat scoring (caller 00a6a4b0). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52b20` | `— ⚠reject` |  | Applies the zone/render configuration held by the pirate-battle object to the shared view interface (same virtual-slot pattern as 00a4f7f0); called from the ZoneConfig handling path 00a6cdf0. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a52c20` | `— ⚠reject` |  | Moves a recycled node from the free list onto the active proxied-object list during pirate ship-combat updates. | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a53d20` | `— ⚠reject` | ShipCombatProxiedProjectile | Deletes a proxied projectile object of the pirate ship-combat subsystem. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a53de0` | `— ⚠reject` | ShipCombatProxiedFort | Deletes a proxied fort object of the pirate ship-combat subsystem (second copy next to the vtable-registered deleting destructor). | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a53e40` | `— ⚠reject` | ShipCombatProxiedChest | Deletes a proxied treasure-chest object of the pirate ship-combat subsystem. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a5faa0` | `— ⚠reject` |  | Likely body handler for DestroyProxiedPickup; the name and message role are supported by its packet-constructor callee and parent dispatcher case 0x1d, while exact field semantics remain uncertain. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a60a90` | `— ⚠reject` |  | Called by the Pirates ship-combat stage setup at 00a64960. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a620d0` | `— ⚠reject` | ShipCombatPacketCreateProxiedStructure | Destructor body called by the CreateProxiedStructure scalar deleting destructor at 00a65440. | [[Native Map/Shards/S0301 00a5da90-00a62140|S0301]] |
| `00a67a70` | `— ⚠reject` |  | Feeds the processor debug-render lists that the ship-combat debug view draws; the temporary it builds is the packet whose vtable is ShipCombatPacketRenderDebugData. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a69480` | `— ⚠reject` |  | Updates locally computed ship-combat readiness flags and reports a changed flag word through the game-state packet. The raw branch makes the supplied third-argument mask ineffective; the state-entry calls in 00a6b0c0 do… | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6b0c0` | `— ⚠reject` | ShipCombatProcessor | Client-side Pirates match state transitions and their UI/physics entry actions; the calls with 0x80 and 0x400 do not prove those bits are forced or reported. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6b5f0` | `— ⚠reject` | ShipCombatProcessor | Per-match cleanup called by the processor destructor and match-exit path; interface release at +0x308 is conditional on both client and buffer pointers being null. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6b8a0` | `— ⚠reject` | ShipCombatProcessor | Builds the processor that owns all Pirates match state on the client and its script-side mirror; the vtable list of this object enumerates every container the shard operates on. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6bff0` | `— ⚠reject` | ShipCombatProcessor | The per-frame tick of the Pirates minigame client, called from the frame updater 0090c970 and from the GUI abort path 00936d00 which logs GUI is aborting: ShipCombat. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6c3b0` | `— ⚠reject` |  | Applies the server announcement that another player ship exists: parses the packet record, creates or finds the ship entity and registers it in the local processor and script layer. | [[Native Map/Shards/S0303 00a674c0-00a6c3b0|S0303]] |
| `00a6c630` | `— ⚠reject` |  | Switches the Pirates ship-combat presentation (indicators, timer, standings) on or off and sets up or releases the per-match state. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00ca17c0` | `— ⚠reject` | ShipCombatProxiedShip | Implements the scalar deleting destructor vtable entry. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3270` | `— ⚠reject` | ShipCombatShipProcessor | Updates a processor-owned object after its nested state is available. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca35b0` | `— ⚠reject` |  | Cleans up a stack temporary whose construction occurs outside this helper. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca3690` | `— ⚠reject` |  | Runs constructor cleanup for a temporary proxied ship. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca40b0` | `— ⚠reject` |  | Increments the source reference count, allocates a 0x58-byte object, initializes it using five source fields and a global context, stores it, then balances a temporary reference. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca41a0` | `— ⚠reject` |  | When receiver and nested state are available and a flag is clear, subtracts the supplied 2D point from a transformed world point and returns whether its length is below the supplied threshold. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca46d0` | `— ⚠reject` |  | Starts from the node at +0x10 and follows up to the requested number of +0x5d0 links, stopping on a null node; it returns no node. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca47a0` | `— ⚠reject` |  | Walks nodes from +0x10 through +0x5d0; returns one at the end, or returns the first node address masked with 0xffffff00 when its +0x14 field is zero. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00ca47f0` | `— ⚠reject` |  | Removes a list node and releases its associated object storage. | [[Native Map/Shards/S0409 00ca0f50-00ca6310|S0409]] |
| `00cabe50` | `— ⚠reject` |  | Synchronizes a projectile transform with its referenced object. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac470` | `— ⚠reject` |  | Calls virtual slot 0x10 on the referenced object and discards the returned value. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac4f0` | `— ⚠reject` |  | Advances a timed ship-combat visual effect. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cac8d0` | `— ⚠reject` |  | Creates an effect from a named ship-combat definition. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacc70` | `— ⚠reject` | ShipCombatProxiedStructure | Implements scalar deleting destruction for the proxied structure base. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cacc90` | `— ⚠reject` |  | Queries an actor through a virtual method and compares returned floating-point values against a global threshold before returning a boolean. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cad750` | `— ⚠reject` |  | Creates the effect set used by ship-combat structures. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cae5a0` | `— ⚠reject` |  | Advances cannon barrage motion from actor effect data. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cae6f0` | `— ⚠reject` |  | Stores a referenced actor pointer, reads three global transform vectors, and passes a local transform structure to the transform helper. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cae7d0` | `— ⚠reject` |  | Advances cannon barrage effect and motion state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caebb0` | `— ⚠reject` |  | Adds or refreshes one cannon record in barrage state. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caec40` | `— ⚠reject` |  | Creates cannon barrage state for an actor. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf6f0` | `— ⚠reject` |  | Resizes results rows while retaining entries. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf870` | `— ⚠reject` |  | Obtains a writable ship-combat results row. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00caf940` | `— ⚠reject` |  | Appends one row to results array. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cafb00` | `— ⚠reject` |  | Initializes row when requested index is beyond current count, then returns address computed from base plus index times 0x50. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ca7480` | `ShipCombatFrameUpdate_00ca7480 ? ⚠downgrade` | send | PacketTunneledClientPacket |  |  |
| `00a69480` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server received changed client readiness words in ShipCombatPacketUpdatePlayerGameStates and used them to coordinate match readiness. This depends on this updater, packet constructor 00a4ee00, and tunneled sender 00a67ff0; it does not establish server acceptance or aggregation rules. |
