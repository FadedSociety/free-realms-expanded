# System: game.factory

80 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00991050` | `ValidateFactoryToolsDataSource` |  | Validates the factory tools UI data-source type. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009911b0` | `FindFactoryToolsDataSourceByNameHash` |  | Looks up a FactoryToolsDataSource by name hash and validates its runtime type; matching hashes are not disambiguated by string comparison. | [[Native Map/Shards/S0265 0098ec00-00995660|S0265]] |
| `009dfb20` | `List_Dtor_ItemRequestEntry` |  | member destruction of the item-request list used by factory recipes. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfb60` | `HashListMap_Dtor_BlueprintDataSource32` |  | member destruction of the blueprint data source map. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfb90` | `List_Dtor_ClientItemConstPtr` |  | member destruction of a list of client item pointers used by factory data. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009e1590` | `DumpFactoryData` |  | Diagnostic factory-state dump. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e1b60` | `LogPlacementNodeAtPosition` |  | Factory placement diagnostic. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e42a0` | `FactoryProcessor::Ctor` | FactoryProcessor | Initializes client factory processor. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e4dc0` | `FactoryMisfortuneMap_scalarDeletingDtor` |  | Deleting destructor counterpart of 009e4960 for the same factory-misfortune map; it has no direct callers because the vtable invokes it. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5040` | `FactoryMisfortuneMap_copyCtor` |  | Constructs a copy of the factory-misfortune map when a factory record (FactoryInfo/plot data) is copied; the exact key type is fixed by the vtable stored at entry. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5080` | `PlacementNodeMap_copyCtor` |  | Copy constructor of the FactoryPlot placement-node map; the FactoryPlot copy constructor 009e9520 calls it for the map at FactoryPlot+0x250 (param+0x94). | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e8590` | `FoundationMap_copyCtor → FactoryFoundationMap_copyCtor` |  | Copy constructor of the foundation map whose value type is the factory-foundation record; the RTTI names both the FoundationGuid key and the FactoryFoundation value. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |

## 🟢 likely (55)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d9b40` | `LookupValueByKey` |  | keyed lookup helper used by the plot/factory data model (the same this+0x2270 owner used by the plot update functions). | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009d9b70` | `SetHousingGridSpacingFromDefinition` |  | writes the grid spacing derived from a factory/plot definition into the housing object field +0x4188 - the same field FUN_00c0ba80 (House.SetGridSpacingInCentimeters) stores 100.0/cm in - so the reciprocal of the defini… | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009d9bf0` | `ForwardKeyToMember_23a4` |  | forwards a key to the object stored at +0x23a4 (the object FUN_00c64b50 matches against its node list at +0x264); the only caller 0092f460 is the mount/despawn update path, so the member is likely the mount/appearance o… | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009d9c60` | `GetValueForOwnerKey` |  | value query for the owner key on the member map at this+0x10; its caller 00aa6140 adds the returned value to a base after the 009d9c10 predicate succeeds. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009d9c80` | `LookupEntryForKey` |  | look up the object registered for the owner's key in the plot/factory data model without using the result. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009db860` | `HasEntriesInBothTables` |  | validity predicate for a (guid,id) pair against two different lookup tables inside two different receiver objects; called from the factory NPC tooltip creators 009e8630 and 009e8760. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009db8d0` | `GetActiveNpcTooltipInfo_22c0` |  | accessor for the factory NPC tooltip info shown by the tooltip UI: the only caller 00a79250 builds the tooltip from this struct (FactoryNpcTooltip strings, EffectIcon%d/EffectText%d over the effect list) and 009e8760 fi… | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009db970` | `Wrapper_KeyPairMapLookup_01057ab0` |  | thin forwarding wrapper around the key-pair lookup of the member at this+0x10 (nodes chained at +0x1268, keys compared inside the callee). | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dd3c0` | `FindPairThenCall_01059480` |  | two-step lookup-and-act helper: resolve a pair into two objects and act on the second one. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dd420` | `PlotCollection_FindCopyAndSelect` |  | move the map entry for a key pair into the plot record's staged slot (+0x2280) and drop it from the map. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009de470` | `FactoryPlot_StampGridQuads` |  | generate the per-cell visual tiles of each factory/plot grid (farm and ranch plots use grids of about 1360x730 cells per Resources/FactoryPlots.txt); the alternating flag arguments produce a checkerboard-like quad patte… | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dea70` | `FactoryPlot_StampGridQuadsByCell` |  | stamp the plot's grid quads while colouring/filtering each cell according to the plot's own occupancy query. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009df1d0` | `List_AddTailEffectTag` |  | add-tail of a SoeUtil::List<EffectTag>: the only caller 009e3450 is the list copy used by the placement-record copy 009e4f70 (list at +0x44) and the factory data-record copies 009e6b30/009e6d60 (list at +0x50). | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfba0` | `Tree_DeleteAndZero_Alignment4` |  | teardown of a tree-shaped member (alignment 4), reached from the same EH funclets as the item-request list destructor 009dfb20. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfca0` | `List_ScalarDeletingDtor_ItemRequestEntry` |  | virtual delete stub of the item-request list. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfcd0` | `HashListMap_ScalarDeletingDtor_BlueprintDataSource32` |  | virtual delete stub of the blueprint data source map. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dfd10` | `List_ScalarDeletingDtor_ClientItemConstPtr` |  | virtual delete stub of the client item pointer list. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009e4000` | `ZeroField14AndInitBoundedList` |  | Initializes tooltip info embedded by 009e42a0; tooltip role is supported by the bounded-list callee and caller context. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e4510` | `CopySixDwordsAndListNodes` |  | Copies tooltip data used by handler 009e79c0; feature role comes from caller context. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e4570` | `BuildBlueprintDataSourceMap` |  | Populates the blueprint-browser map constructed by only caller 009e9060. A missing item definition enters the request-if-missing path through 00a3ddd0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e48e0` | `ToolDefinitionMap_scalarDeletingDtor` |  | Deleting destructor of the same HashListMap<int,FactoryPacketListToolsResponse::FactoryToolDefinitionNode,256,-1> instance that 009e4140 destroys; it has no direct callers because the vtable reaches it. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4960` | `FactoryMisfortuneMap_dtor` |  | Destructor for the map keyed by FactoryMisfortuneGuid. Its EH unwind callers establish cleanup use; they do not identify the owning record or a housing build-data role. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e49f0` | `FactoryMisfortuneMap_assign` |  | Copy step of the factory-misfortune map constructor 009e5040 and the in-place assignment for the same map type; the map identity comes from that constructor vtable. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4a60` | `HashListAssign_CopyChainB` |  | Assignment for one of the nested maps of the client FactoryInfo record (used by the FactoryInfo copy at 009e9be0 on the map at +0x1130); the element type is not established by this function's own evidence, so the name s… | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4e00` | `HashListDrain_00` |  | Cleanup of one of the FactoryInfo nested maps, using the same free routine FUN_009e0680 as the map assignment 009e6220; the element type is not established here. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4e20` | `HashListMap_Insert64` |  | Insert primitive of a 64-bucket FactoryInfo nested map (paired with the assignment 009e6220 and with the deserializer insert FUN_009e0c20); the concrete element type is not named by this function's evidence. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4f10` | `ToolMap_NodeCtor` |  | Value-construction step of the tool-map insert 009e62e0, which reads the key back from +0x54 to pick the bucket; that insert fills the FactoryPacketListToolsResponse tool map. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e4f70` | `PlacementRecord_copyCtor` |  | Copy constructor of the factory record that owns a List<EffectTag> and is used as the map value by the node constructor 009e6390; the placement role comes from the callers while the EffectTag list layout comes from this… | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5290` | `DwordKeyedMap_read` |  | Nested dword-keyed map reader called by 009e9640; this function does not establish the mapped value type. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e5370` | `PlacementRecord_read` |  | Reader of the record whose copy constructor is 009e4f70; it is the value reader of the 1024-bucket map deserialized by 009e71f0 inside the FactoryInfo record. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6220` | `HashListMap_Assign64` |  | Copy/assign used by the FactoryInfo copy at 009e9be0 for the map at +0x852 dwords; element type is not established by this function alone. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e62e0` | `ToolMap_Insert` |  | Insert primitive of the FactoryPacketListToolsResponse tool table (assignment 009e6c50 and the handler copy 009e8de0 use it); the key at +0x54 is the tool id dword. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6390` | `PlacementMap_NodeCtor` |  | Value/key construction step of the 1024-bucket placement map insert 009e6ca0, which hashes the two key dwords at +0x68/+0x6c. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6c50` | `ToolMap_Assign` |  | Copies an incoming tool map into the live BaseClient.Factory.Tools data source; its only caller 009e8de0 is the tool-list response handler, which also fills the UI with the parsed rows. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6ca0` | `PlacementMap_Insert` |  | Insert primitive of the map whose nodes carry the placement record plus the dword key pair; the assignment for the same map is 009e7340, used by the FactoryInfo copy 009e9be0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6d60` | `FactoryDataRecord_CopyCtor` |  | Second copy constructor of the composite factory record used by the FactoryPlot copy path 009e7410; it delegates to the sub-object copy constructors instead of re-inserting nodes directly. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7340` | `PlacementMap_Assign` |  | Copy step for the placement map of the FactoryInfo record, used by the FactoryInfo copy constructor 009e9be0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7410` | `FoundationRecord_copyCtorB` |  | Copy path used when a foundation value is copied into a map node (called by the node constructor 009e7720); it differs from 009e72a0 only in delegating to the composite copy constructor 009e6d60. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7720` | `FoundationMap_NodeCtor` |  | Value/key construction step of the foundation map insert 009e7b80; the key pair sits right after the 0x108-byte value record inside the node. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e79c0` | `FactoryHandler_NpcTooltipReply` |  | Applies an NPC tooltip reply only when it matches the cached request pair or the cached pair is null; the data and state belong to the FactoryPacketNpcTooltipReply handler. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e7b80` | `FoundationMap_Insert` |  | Insert primitive of the foundation map that the plot record owns at +4 (assignment 009e83b0, copy constructor 009e8590). | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e83b0` | `FoundationMap_Assign` |  | Copy step for the foundation map of a plot record, used by the FactoryPlot copy constructor chain (009e8590 is its dedicated copy constructor). | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e8630` | `Tooltip_RequestIfStale` |  | Builds and sends an NPC tooltip request when the cached tooltip state is absent, mismatched, or older than the observed 5000 threshold; timer units are unknown. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e89d0` | `PlotRecord_Assign` |  | Overwrite operation used by 009e98b0 to replace an existing plot in the client table after the incoming guid matched; it is the in-place counterpart of the copy constructor 009e9520. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e8aa0` | `FactoryHandler_UpdateFoundation` |  | Applies a server foundation update to the addressed factory plot, creating the plot entry first when needed; it is a dispatcher case routed from 009e9d40. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9060` | `FactoryHandler_OpenBlueprintBrowserOnFoundation ⚠downgrade` |  | Server-driven opening of the blueprint browser on a foundation: this is the only caller of 009e4570 and of the ShowBlueprint helper 009e5ac0, which is how those two functions inherit their blueprint role. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9280` | `Tooltip_RefreshTick` |  | Periodic tooltip-request retry after the 10000 threshold when the shown guid is cached but the requested guid remains null; timer units are unknown. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9520` | `PlotRecord_copyCtor` |  | Likely the per-plot value copied by PlotTable_NodeCtor (009e9780); that identity is inferred from its matching field layout with PlotRecord_read/Assign, while this body directly establishes the contained factory maps an… | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9780` | `PlotTable_NodeCtor` |  | Value/key construction step of the plot table insert 009e99b0, which buckets on the guid stored at +0x1270. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e98b0` | `FactoryHandler_UpdatePlot` |  | Applies a parsed plot update to an existing table entry when GUIDs match; FactoryPacket_Dispatch (009e9d40) invokes this handler for sub-opcode 0x1c. Its handled result reports parse success, not a table match. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e99b0` | `PlotTable_Insert` |  | Insert primitive of the client plot table used by the update-plot handler path; the node layout (guid at +0x1270, chain at +0x1274) matches the table searched by 009e98b0/009e8aa0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9af0` | `PlotTable_Assign` |  | Copy step for the plot table used by the factory-info copy 009e9be0. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9c50` | `FactoryHandler_UpdateFactoryInfo` |  | Case 1 of the factory packet dispatcher: refreshes the whole client factory info (plot table and nested maps) from the server snapshot. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00c23970` | `RequestRecipeStart` |  | Dispatches the client's recipe-start command request. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c23fc0` | `RequestRecipeReward` |  | Requests the reward for a completed recipe. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009d9c10` | `LookupField40e8Found ?` |  | Returns whether FUN_01032950 finds an entry for the dword read from the accessor result at +0x40e8 (or 0xffffffff when the accessor is null). The lookup table and the meaning of that field are not established. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009e72a0` | `FoundationRecord_copyCtor ? ⚠rename` |  | Used where a parsed foundation record is copied into client state; its only caller 009e8aa0 is the update-foundation packet handler that first looks up or creates the plot. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e9be0` | `FactoryInfo_copyCtor ? ⚠rename` |  | Copies the whole factory-info state (plot table, placement map and two nested maps) when the update-factory-info handler stores a parsed record; the maps it copies are the ones read by 009e9640. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `00b84600` | `SpawnObjectByKindAndRegister ?` |  | Thiscall with three arguments. When the third argument is 0, it fills a request block from param_2 (dwords +0x60, +0x64, +0x68, +0x6c, with FUN_006f9e60 on +0x84) and the dword at +0x260 of the object at this+0x1a4, the… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00bfeea0` | `Factory::RequestCreateBlueprint ? ⚠downgrade` | Factory | Requests a factory blueprint using item identifier and quantity pairs supplied by Lua. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |
| `00c240b0` | `RequestFactoryOperation ? ⚠rename` |  | Submits a factory operation represented by the resolved object. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c24150` | `RequestFactoryOperation ? ⚠rename` |  | Submits a factory operation represented by the resolved object. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009db900` | `— ⚠reject` |  | apply the staged placement record (this+0x2280) to the map node keyed by this+0x2278 and clear the staging words; removal of a key is FUN_01059900, the routine 009dd420 calls. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009e4410` | `— ⚠reject` | FactoryProcessor | FactoryProcessor teardown; the operation implemented by helper virtual slot +0x28 is not identified. | [[Native Map/Shards/S0279 009e0340-009e4510|S0279]] |
| `009e4860` | `— ⚠reject` |  | Constructor of the tool-definition table used by the factory tool-list packet and by the BaseClient.Factory.Tools data source (caller 00a726b0); the packet constructor 009e5580 builds the same object inside FactoryPacke… | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e51c0` | `— ⚠reject` |  | Reads the 8-byte-keyed map region at +0x94 of the factory record deserialized by 009e7880; both the map size class and the value type come from its caller, not from this function. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e6b30` | `— ⚠reject` |  | One of the two copy constructors of the composite factory record (the other is 009e6d60 which delegates to the sub-map copy constructors); the record is the value type of the factory foundation entry. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |
| `009e82c0` | `— ⚠reject` |  | Client-side update of a factory blueprint for the addressed plot; it is one of the dispatcher cases routed from 009e9d40. | [[Native Map/Shards/S0280 009e4570-009ea840|S0280]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009e4570` | `BuildBlueprintDataSourceMap` | send | PlayerUpdatePacketItemDefinitionRequest |  | Hypothesis: on a cache miss with no request pending, the original server receives a PlayerUpdatePacketItemDefinitionRequest inside PacketTunneledClientPacket. This depends on 00a3ddd0 calling 00a3dca0 and its packet constructor; the server response behavior is unknown. |
| `009e4570` | `BuildBlueprintDataSourceMap` | send | PacketTunneledClientPacket |  | Hypothesis: on a cache miss with no request pending, the original server receives a PlayerUpdatePacketItemDefinitionRequest inside PacketTunneledClientPacket. This depends on 00a3ddd0 calling 00a3dca0 and its packet constructor; the server response behavior is unknown. |
| `009e5290` | `DwordKeyedMap_read` | recv | PacketTunneledClientPacket |  |  |
| `009e5370` | `PlacementRecord_read` | recv | PacketTunneledClientPacket |  |  |
| `009e79c0` | `FactoryHandler_NpcTooltipReply` | recv | PacketTunneledClientPacket |  |  |
| `009e8630` | `Tooltip_RequestIfStale` | send | PacketTunneledClientPacket |  |  |
| `009e8aa0` | `FactoryHandler_UpdateFoundation` | recv | PacketTunneledClientPacket |  |  |
| `009e9060` | `FactoryHandler_OpenBlueprintBrowserOnFoundation ⚠downgrade` | recv | PacketTunneledClientPacket |  |  |
| `009e9060` | `FactoryHandler_OpenBlueprintBrowserOnFoundation ⚠downgrade` | send | PacketTunneledClientPacket |  |  |
| `009e9280` | `Tooltip_RefreshTick` | send | PacketTunneledClientPacket |  |  |
| `009e98b0` | `FactoryHandler_UpdatePlot` | recv | PacketTunneledClientPacket |  |  |
| `009e9c50` | `FactoryHandler_UpdateFactoryInfo` | recv | PacketTunneledClientPacket |  |  |
| `009e51c0` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
| `009e82c0` | `— ⚠reject` | recv | PacketTunneledClientPacket |  |  |
