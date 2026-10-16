# System: engine.asset

114 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (49)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089ec80` | `AssetRequestInterface::~AssetRequestInterface` | AssetRequestInterface | Base-object destructor of the asset-request interface, used as the cleanup for objects that expose the AssetManagement::AssetRequestInterface vtable; both callers are EH unwind funclets (0158fca0, 01644050). | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `0089ecc0` | `AssetRequestInterface::ScalarDeletingDestructor` | AssetRequestInterface | Heap-delete path for asset-request objects exposing the AssetManagement::AssetRequestInterface vtable; the non-deleting body is the 7-byte routine at 0089ec80. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008a2530` | `SoundDefinitionAssetLoader::SoundDefinitionAssetLoader` | SoundDefinitionAssetLoader | Creates the loader object that reads sound-definition assets through the asset-request interface (called from 00915510, 009461a0, 00946a80 and 00c1bb40). | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008b8990` | `BaseClientFileOpener::BaseClientFileOpener` | BaseClientFileOpener | Initializes the client file opener's base and derived vtables. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b8af0` | `DownloadMandatoryAssetsProgress` |  | Reports progress and failure for mandatory asset downloads. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008cf920` | `SensoryEffectDefinitionLoader::SensoryEffectDefinitionLoader` | SensoryEffectDefinitionLoader | Create the asset loader object that registers itself as the sensory-effect definition manager and asset-delivery handler (eleven callers are the actor/effect definition loaders). | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cf9d0` | `SensoryEffectDefinitionLoader::~SensoryEffectDefinitionLoader` | SensoryEffectDefinitionLoader | Tear down the loader object registered as the sensory-effect definition manager (referenced from 22 unwind funclets). | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfa90` | `ActorLightEmitterDefinition::~ActorLightEmitterDefinition` | ActorLightEmitterDefinition | Tear down one actor light-emitter definition asset loaded through the asset-request interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfba0` | `ActorCameraEffectDefinition::~ActorCameraEffectDefinition` | ActorCameraEffectDefinition | Tear down one actor camera-effect definition asset. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfcb0` | `ActorRumbleEffectDefinition::~ActorRumbleEffectDefinition` | ActorRumbleEffectDefinition | Tear down one actor rumble-effect definition asset. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfdb0` | `ActorLightsaberTrailDefinition::~ActorLightsaberTrailDefinition` | ActorLightsaberTrailDefinition | Tear down one actor lightsaber-trail definition asset and its string member. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008d4270` | `AssetHashListMap_ScalarDeletingDtor` | SoeUtil::HashListMap<int,AssetManagemen… | Heap-delete path for the 8192-bucket map keying ints to AssetManagement::Asset pointers; it is the deleting counterpart of the EH-only teardown 008d3eb0. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `0091e8a0` | `HashListMap_AssetReadyHandler_Dtor` | SoeUtil::HashListMap<int,AssetManagemen… | Teardown of the ready-handler map that tracks asset requests; callers 00920d20, 009229a0, 009251e0, 010988d0 plus EH unwinds. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `00989030` | `PendingLoad2_OnAssetReady` | ImageDirectory::PendingLoad2 | Completion callback for the pending image-load request. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00989470` | `ImageAsset2_Constructor` | ImageDirectory::ImageAsset2 | Initializes the ImageDirectory image asset wrapper. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009894d0` | `ImageAsset2_Destructor` | ImageDirectory::ImageAsset2 | Releases the wrapped Scaleform image before base destruction. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00989540` | `ImageAsset2_OnImageBytes` | ImageDirectory::ImageAsset2 | Receives asset bytes, obtains the asset name through its virtual interface, resolves the associated image, and updates it through Scaleform. The descriptor argument layout remains uncertain because Ghidra uses an inferr… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00989740` | `PendingLoad2_CreateImageAsset` | ImageDirectory::PendingLoad2 | Creates the image asset associated with a pending load. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00ae2d50` | `FileRepCtor` | AppUtil::FileRep | Initializes file-representation base state. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2db0` | `FileRepIsComplete` | AppUtil::FileRep | Tests whether the file representation's two reported positions match. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2de0` | `FileRep::ScalarDeletingDtor` | AppUtil::FileRep | Deleting destructor for the shared file representation base. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2e00` | `BaseClientFileCtor` | BaseClientFile | Constructs the client file wrapper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2e70` | `BaseClientFile::SetStatus5` | BaseClientFile | Sets BaseClientFile status to 5. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2ea0` | `BaseClientFile::GetStatus` | BaseClientFile | Returns current file status. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2fa0` | `BaseClientFile::IsOpen` | BaseClientFile | Reports whether the client file has an active readable backing. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae30c0` | `BaseClientFile::Read` | BaseClientFile | Reads bytes from a client file representation. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3190` | `BaseClientFile::Seek` | BaseClientFile | Moves the file cursor. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3280` | `BaseClientFile::ScalarDeletingDtor` | BaseClientFile | Deleting destructor for BaseClientFile. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae32a0` | `CreateBaseClientFile` | BaseClientFileOpener | BaseClientFileOpener virtual object creation method. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3310` | `BaseClientFile::Open` | BaseClientFile | Opens a client file from a path. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae34d0` | `BaseClientFileOpener::OpenForRead` | BaseClientFileOpener | Opens or queries a file through the active opener. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3810` | `TrackedAssetHashListInit` | SoeUtil::HashListMap<int,DelayedAssetLo… | Initializes DelayedAssetLoader tracked-asset lookup storage. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3d70` | `TrackedAssetHashAlloc` | SoeUtil::HashListMap<int,DelayedAssetLo… | Allocator hook for tracked asset nodes. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3db0` | `TrackedAssetHashFree` | SoeUtil::HashListMap<int,DelayedAssetLo… | Free-node hook for tracked asset nodes. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00b0c400` | `MiniMapImageMap::Ctor` | SoeUtil::HashListMap<int,MiniMapImage,1… | Initializes the map used to index minimap images. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c820` | `MiniMapInfoAsset::Ctor` | MiniMapInfoAsset | Initializes minimap metadata asset state. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c880` | `MiniMapInfoAsset::Dtor` | MiniMapInfoAsset | Releases the asset's shared reference and base state. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0c900` | `MiniMapInfoAsset::OnLoaded` | MiniMapInfoAsset | Installs loaded data and notifies the asset request callback. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ca30` | `MiniMapInfoAsset::ScalarDeletingDtor` | MiniMapInfoAsset | Implements scalar deleting destruction for MiniMapInfoAsset. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0cc70` | `MiniMapImages::Dtor` | MiniMapImages | Destroys minimap image request and index state. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0cff0` | `MiniMapImages::Ctor` | MiniMapImages | Initializes minimap image asset request and index state. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0d100` | `MiniMapImages::ScalarDeletingDtor` | MiniMapImages | Implements scalar deleting destruction for MiniMapImages. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b70cb0` | `BattleMagesProcessor::CreateParticleDefinitionAsset` | BattleMagesProcessor | Creates and returns an asset object through the processor?s slot-3 hook. The call site does not show asset registration or a feature-specific request. | [[Native Map/Shards/S0352 00b6cd80-00b71320|S0352]] |
| `010988d0` | `AssetReadyHandler::DtorBody` | AssetManagement::AssetRequestInternal::… | Destructor body of AssetReadyHandler: tears down its deliverable list and ready-handler hash; called from unwind funclets and from 01098940. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098940` | `AssetRequestHandler::DtorBody` | AssetManagement::AssetRequestHandler | Destructor body of AssetRequestHandler; the scalar deleting destructor 00927fd0 calls it (label AssetRequestHandler::ScalarDeletingDtor). | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01099730` | `ReadJobDtorBody` | AssetManagement::AssetDeliveryIndirect:… | Destructor body of the ReadJob asset-delivery job; called from 01099e30 (outside this shard) and its scalar deleting form is not in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `010997f0` | `WriteJobDtorBody` | AssetManagement::AssetDeliveryIndirect:… | Destructor body of the WriteJob asset-delivery job; caller 01099f90 is outside this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `010998b0` | `DecompressJobDtorBody` | AssetManagement::AssetDeliveryIndirect:… | Destructor body of the DecompressJob asset-delivery job; caller 01099fb0 is outside this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01099b80` | `PendingRequestDtorBody` | AssetManagement::AssetDeliveryIndirect:… | Destructor body of PendingRequest; called from four EH unwind funclets and four functions in other shards (01099be0, 0109a170, 0109acf0, 0109b9f0). | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

## 🟢 likely (35)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0089edd0` | `AssetDeliveryHandlerInterface::ScalarDeletingDestructor` | AssetDeliveryHandlerInterface | Heap-delete path for an asset-delivery handler object; the full destructor it reuses is 010935e0, reached from 35 EH funclets, so this wrapper is the delete-side entry of the whole handler family. | [[Native Map/Shards/S0219 0089d990-008a3c80|S0219]] |
| `008cf9c0` | `AdjustorThunk_sub4_to_008cfa70` |  | Interface vtable entry for deleting a SensoryEffectDefinitionLoader through its +4 management interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfa70` | `SensoryEffectDefinitionLoader::ScalarDeletingDestructor` | SensoryEffectDefinitionLoader | Deleting destructor entry of the SensoryEffectDefinitionLoader vtable (slot [0] of 018028dc): run the loader destructor body and optionally free the object. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfb70` | `AdjustorThunk_sub0x90_to_008cfb80` |  | Interface vtable entry for deleting an ActorLightEmitterDefinition through its +0x90 asset-request interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfb80` | `ActorLightEmitterDefinition::ScalarDeletingDestructor` | ActorLightEmitterDefinition | Deleting destructor entry of the ActorLightEmitterDefinition vtable (slot [0] of 01802910): run the destructor body and optionally free the object. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfc80` | `AdjustorThunk_sub0x90_to_008cfc90` |  | Interface vtable entry for deleting an ActorCameraEffectDefinition through its +0x90 asset-request interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfc90` | `ActorCameraEffectDefinition::ScalarDeletingDestructor` | ActorCameraEffectDefinition | Deleting destructor entry of the ActorCameraEffectDefinition vtable (slot [0] of 01802954): run the destructor body and optionally free the object. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfd80` | `AdjustorThunk_sub0x90_to_008cfd90` |  | Interface vtable entry for deleting an ActorRumbleEffectDefinition through its +0x90 asset-request interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfd90` | `ActorRumbleEffectDefinition::ScalarDeletingDestructor` | ActorRumbleEffectDefinition | Deleting destructor entry of the ActorRumbleEffectDefinition vtable (slot [0] of 01802998): run the destructor body and optionally free the object. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfeb0` | `AdjustorThunk_sub0x90_to_008cfec0` |  | Interface vtable entry for deleting an ActorLightsaberTrailDefinition through its +0x90 asset-request interface. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008cfec0` | `ActorLightsaberTrailDefinition::ScalarDeletingDestructor` | ActorLightsaberTrailDefinition | Deleting destructor entry of the ActorLightsaberTrailDefinition vtable (slot [0] of 018029dc): run the destructor body and optionally free the object. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `00987f60` | `ImageDirectory_FindImageId` | ImageDirectory | Finds an image identifier/value by key across the directory's three internal indexes. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00ae31f0` | `BaseClientFileDtor` | BaseClientFile | Destructor body for BaseClientFile. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3740` | `StoreFieldAt18` |  | Research note identifies caller 00a46cd0 as ClientTutorialManager construction; it calls this setter to register the manager. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae48d0` | `RefillDelayedAssetRequests` |  | Delayed asset request queue maintenance; unit meaning is unresolved. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00b0d3e0` | `MiniMapImages::ParseTileInfo` | MiniMapImages | Parses minimap tile metadata into indexed records. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0d7a0` | `MiniMapImages::OnTileInfoLoaded` | MiniMapImages | Handles completion of a minimap metadata request. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b16fd0` | `Thunk_00ae6b90_Adjust10` |  | Interface adjustor that lets a subobject at offset 0x10 be used as the asset-delivery callback object expected by 00ae6b90. | [[Native Map/Shards/S0336 00b150e0-00b1b340|S0336]] |
| `00b663a0` | `FishingProcessor::RequestAsset` | FishingProcessor | Asset-requester entry point of the fishing processor (the class inherits AssetManagement::AssetRequestInterface::AssetRequester); the engine-side asset system allocates a processor, calls this to start its request, and … | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b77760` | `AcquireAssetRefs` |  | Reference acquisition for a resolved asset and its dependencies (mesh/animation/texture/sound by type id); called from FUN_00cda220 in the soccer client. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b778c0` | `ReleaseAssetRefs` |  | Reference release for a resolved asset and its dependencies; called from FUN_00cd8640 in the soccer client. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `010947d0` | `AssetDeliveryInterface::DrainPendingLists` | AssetManagement::AssetDeliveryInterface | Virtual slot [22] of AssetDeliveryInterface (also present in AssetDeliveryDirect): drains and dispatches both pending queues, one after the other. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01095e60` | `AssetRequestHandler::ReleaseAllEntries` | AssetManagement::AssetRequestHandler | Virtual slot [14] of AssetRequestHandler: releases every entry in the handler list and clears the byte total. The caller 01098940 is in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01096830` | `AssetRequestHandler::InsertHashedEntry` | AssetManagement::AssetRequestHandler | Virtual slot [8] of AssetRequestHandler: inserts a request entry into its 8192-bucket table and accumulates its size into the second 64-bit total; caller not identified in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01096900` | `AssetRequestHandler::FindHashedEntryByName` | AssetManagement::AssetRequestHandler | Virtual slot [7] of AssetRequestHandler: name lookup in the same 8192-bucket table that slot [8] (01096830) inserts into. Reached from the lookup wrapper 01096520 through the owner vtable. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `010986f0` | `AssetRequestHandler::PeriodicTick` | AssetManagement::AssetRequestHandler | Virtual slot [4] of AssetRequestHandler: periodic service step that dispatches queued entries within a budget and runs the eviction every countdown period. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098750` | `Thunk_Adjust3c_010984f0` | AssetManagement::AssetRequestHandler | Virtual slot [5] of AssetRequestHandler reached through the 0x3c subobject; it forwards to the keyed list removal of 010984f0. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098b80` | `AssetDeliveryIndirect::ForwardSub62cSlot6` | AssetManagement::AssetDeliveryIndirect | Virtual slot [16] of AssetDeliveryIndirect: forwards to the object held at +0x62c through its slot [6] when that object exists. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098ba0` | `AssetDeliveryIndirect::ReturnThis` | AssetManagement::AssetDeliveryIndirect | Identity virtual of AssetDeliveryIndirect (slot [12]): returns this. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098bd0` | `AssetDeliveryIndirect::TailCallSlot7` | AssetManagement::AssetDeliveryIndirect | Virtual slot [5] of AssetDeliveryIndirect: a tail-call forwarder to the object's own slot [7]. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098bf0` | `AssetDeliveryIndirect::CallSub4DrainPending` | AssetManagement::AssetDeliveryIndirect | Virtual slot [22] of AssetDeliveryIndirect: notifies the subobject at +0x62c, runs a lock-related call on this+0x16b88 and drains the pending queues. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098c20` | `AssetDeliveryIndirect::ForwardSub62cSlot5` | AssetManagement::AssetDeliveryIndirect | Virtual slot [15] of AssetDeliveryIndirect: forwards to the object at +0x62c through its slot [5]. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `010994d0` | `AssetDeliveryIndirect::ClearLinksToObject` | AssetManagement::AssetDeliveryIndirect | Virtual slot [19] of AssetDeliveryIndirect: clears back-references to an object held by the list, then calls FUN_010937d0 on it; callers not identified in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `010995e0` | `PackJobCtor` | AssetManagement::AssetDeliveryIndirect:… | Constructor of the PackJob asset-delivery job; the base initialiser FUN_0083f1e0 is not labelled. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01099b00` | `AssetDeliveryIndirect::PendingRequestCtor` | AssetManagement::AssetDeliveryIndirect:… | Constructor of the PendingRequest record used by the 8192-bucket PendingRequest map (key at +0x6c); caller 0109a2b0 is outside this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008d3eb0` | `AssetHashListMap_Dtor ? ⚠downgrade` | SoeUtil::HashListMap<int,AssetManagemen… | Teardown of the 8192-bucket map that keys ints to AssetManagement::Asset pointers - the asset cache's index container. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `00987f20` | `ImageDirectory_ProcessRequest ? ⚠rename` | ImageDirectory | ImageDirectory virtual callback for processing a nonempty image request descriptor. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00ae4b20` | `AssetRequestStateCleanup ? ⚠rename` |  | Thin cleanup wrapper; resource identity is not apparent here. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae4b70` | `ClearAssetRequestFields ?` |  | Clears string and state fields in an asset request owner. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00b0cd30` | `MiniMapImages::RequestTileInfo ?` | MiniMapImages | Requests tile metadata associated with a minimap image. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `01099500` | `AssetDeliveryIndirect::IsReferencedOrFlagged ? ⚠rename` | AssetManagement::AssetDeliveryIndirect | Virtual slot [20] of AssetDeliveryIndirect: predicate asking whether an object is flagged by FUN_01093820 or still referenced by the list at +0x63c. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

## ⚪ unknown (24)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b8a40` | `— ⚠reject` | BaseClientFileOpener | Releases a BaseClientFileOpener allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `009897a0` | `— ⚠reject` | ImageDirectory::PendingLoad2 | Constructs an ImageDirectory pending-load request and its string member. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00aaef30` | `— ⚠reject` |  | Shared resource visitation helper; housing fixture-asset and item-list paths call it, but the resource categories are not named here. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ae2e80` | `— ⚠reject` | BaseClientFile | Runs a backend operation only when the file is ready. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2f40` | `— ⚠reject` | BaseClientFile | Returns file read position. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae2f70` | `— ⚠reject` | BaseClientFile | Returns file size. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3070` | `— ⚠reject` | BaseClientFile | Closes both shared-memory and backend file state. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae3d30` | `— ⚠reject` |  | Releases tracked-asset hash storage. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae4a90` | `— ⚠reject` |  | Resets a delayed asset request record. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae4e70` | `— ⚠reject` |  | Delayed asset request queue reset/compaction helper. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00b0c990` | `— → MiniMapImages::CreateInfoAsset` |  | Allocates 0x30 bytes and, if allocation succeeds, invokes the MiniMapInfoAsset constructor and returns its result. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0ce10` | `—` |  | Constructs an IString output object with empty storage and initializes a field from object offset 0x204c. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0d140` | `—` |  | Clears an indexed list, computes tile keys from coordinates and scale fields, then inserts each source node into a secondary index. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0d1e0` | `—` |  | Converts a spatial rectangle to integer grid coordinates, searches bucket chains across covered cells, and inserts newly encountered keys into a temporary hash map. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `01096e00` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [11] of AssetRequestHandler: rebuilds the handler ordered list and 32-bucket index from the 8192-bucket table entries. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01096f90` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [15] of AssetRequestHandler: moves an entry out of one index and, when the handler and the entry both accept, back into the 8192-bucket table; caller not identified in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01097720` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [3] of AssetRequestHandler: accepts a keyed request with its callback, indexes new entries in the 8192-bucket table, and queues known keys; the caller set is not identified beyond the handler dispatch. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01097910` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [4] of AssetRequestHandler: removes matching entries from the bucket index and re-queues them through the list-append routine; caller set empty in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098260` | `— ⚠reject` | AssetManagement::AssetRequestInternal::… | Virtual slot [0] of AssetReadyHandler: queues the record with an empty name and reports success; the dispatcher is not identified. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098310` | `— ⚠reject` | AssetManagement::AssetRequestInternal::… | Virtual slot [1] of AssetReadyHandler: queues the record with an empty name without a return value. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098570` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [2] of AssetRequestHandler: keyed asset request that carries a payload copied into a new record; sibling of slot [3] (01097720). | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098760` | `— ⚠reject` | AssetManagement::AssetRequestHandler | Virtual slot [3] of the 01805f08 AssetRequestHandler base: delivers an object to every entry waiting on the key and removes those entries from the bucket table. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01098f80` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect:… | Virtual slot [2] of the AssetDeliveryIndirect WriteJob: stores a decoded asset into the pack database and verifies its CRC against the expected value; the two log strings are its failure reports. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01099540` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect | Virtual slot [21] of AssetDeliveryIndirect: predicate for a pending request by name and object; the table at +0x648 uses the same bucket layout as HashListMap8192FindByIntKey (01098cd0). | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

