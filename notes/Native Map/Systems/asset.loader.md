# System: asset.loader

130 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (66)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0071b5b0` | `AssetRequesterScalarDeletingDtor` | AssetManagement::AssetRequestInterface:… | Scalar deleting destructor for AssetRequester. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071bb80` | `AssetConstructor` | AssetManagement::Asset | Initializes common Asset base fields. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071bc20` | `AssetDestructor` | AssetManagement::Asset | Destroys common Asset base fields. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071bcb0` | `AssetScalarDeletingDtor` | AssetManagement::Asset | Scalar deleting destructor for Asset. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `0071c0c0` | `Asset::SharedVirtualMethod2` | AssetManagement::Asset | Shared slot-two implementation across Asset, texture, material, camera, water, and definition classes; semantic name unknown. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00840b80` | `FlashLoaderJobScalarDeletingDtor` | FlashLoaderJob | virtual destructor of FlashLoaderJob (vftable slot 0) that frees the object when the flag says so | [[Native Map/Shards/S0202 0083ea10-00844c00|S0202]] |
| `00927e10` | `AssetReadyHandler::Ctor` | AssetManagement::AssetRequestInternal::… | Constructs the ready handler embedded in asset delivery. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927e80` | `AssetRequestHandler::Ctor` | AssetManagement::AssetRequestHandler | Constructs the asset request and delivery handler. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927f90` | `AssetRequestHandler::CallVirtual1cAsBool` | AssetManagement::AssetRequestHandler | Calls the method at vtable byte offset 0x1c with the argument and converts its result to bool; underlying semantics are unknown. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927fb0` | `AssetRequestHandler::SetStateByte` | AssetManagement::AssetRequestHandler | Sets the handler byte at object offset 0x0c. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927fc0` | `AssetRequestHandler::GetStateByte` | AssetManagement::AssetRequestHandler | Returns the handler byte at object offset 0x0c. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927fd0` | `AssetRequestHandler::ScalarDeletingDtor` | AssetManagement::AssetRequestHandler | Implements the handler scalar deleting destructor. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00927ff0` | `AssetRequestHandler::DispatchByNonzeroValue` | AssetManagement::AssetRequestHandler | Selects between two helper calls based on whether the second argument is nonzero; helper semantics are unknown. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928030` | `AssetRequestHandler::ScalarDeletingDtor` | AssetManagement::AssetRequestHandler | Provides a secondary-vtable destructor entry. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928040` | `AssetRequestHandler::GetBufferField359c` | AssetManagement::AssetRequestHandler | Exposes one handler field through a virtual getter. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928050` | `AssetRequestHandler::GetBufferFieldb5a4` | AssetManagement::AssetRequestHandler | Exposes another handler field through a virtual getter. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00928600` | `InitAssetDelivery` |  | Initializes client asset delivery and sizes the request handler buffers. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `009461a0` | `DeferredPlayListLoader::DeferredPlayListLoader` | DeferredPlayListLoader | Constructor of the deferred play-list asset loader, derived from SoundDefinitionAssetLoader. | [[Native Map/Shards/S0250 00942c80-00947180|S0250]] |
| `009cd8e0` | `FileInfoList::AllocateNode` | SoeUtil::List<ClientStartingZoneLoader:… | Allocates nodes for the ClientStartingZoneLoader file-info list. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cd920` | `FileInfoList::FreeNode` | SoeUtil::List<ClientStartingZoneLoader:… | Releases a node from the ClientStartingZoneLoader file-info list. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cdcd0` | `LoadStartingZoneLoadRecords` |  | Loads client StartingZoneLoad records from the local text file. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ce0d0` | `ClientStartingZoneLoader::Ctor` | ClientStartingZoneLoader | Initializes a starting-zone file loader instance. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `009ce180` | `ClientStartingZoneLoader::Dtor` | ClientStartingZoneLoader | Releases nested loader state and list nodes. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a5c5d0` | `LoadingScreenDataCtor` | LoadingScreenData | Initialize loading-screen data and delivery-handler state. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00a5c680` | `LoadingScreenDataDtorAdjustorThunk` | LoadingScreenData | Adjust the this pointer before dispatching the deleting destructor. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00a5c730` | `LoadingScreenData::ScalarDeletingDtor` | LoadingScreenData | Scalar deleting destructor for LoadingScreenData. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00ae51e0` | `DelayedAssetLoader::FileCreateJob::Destroy` | DelayedAssetLoader::FileCreateJob | Destroys a delayed asset file-creation job and its base state. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae56f0` | `DelayedAssetLoader::FileCreateJob::Ctor` | DelayedAssetLoader::FileCreateJob | Constructs a delayed asset file-creation job and retains its referenced object. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae5790` | `DelayedAssetLoader::FileCreateJob::ScalarDeletingDtor` | DelayedAssetLoader::FileCreateJob | Implements scalar deletion for FileCreateJob. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae5a00` | `CreateRequestFromNextPooledRecord` |  | Initializes the next available delayed-asset request record. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6870` | `OnDelayedAssetRequestError → DelayedAssetLoader::OnAssetRequestError` |  | Handles an asset-delivery failure and updates the matching loader request. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6ab0` | `BeginAssetFileListRequest` |  | Starts asynchronous retrieval of an asset file list. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6b90` | `PumpDelayedAssetRequests` |  | Advances asynchronous deferred asset delivery and reports completion or request failures. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7ae0` | `ParseDeferredAssetsManifest` |  | Parses DeferredAssets.txt entries into delayed asset requests. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7f00` | `OnAssetDataOrManifestReceived → DelayedAssetLoader::OnAssetDataOrManifestReceiv…` |  | Processes an asset-delivery response as a manifest or tracked asset. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae8100` | `DelayedAssetLoader::OnFileCreateJobComplete` | DelayedAssetLoader | Completes a delayed asset file job and reports its result. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae8360` | `DelayedAssetLoader::Ctor` | DelayedAssetLoader | Constructs the delayed asset loader and its embedded request-tracking structures. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae8450` | `DelayedAssetLoader::AdjustThisMinus8ThenScalarDelete` | DelayedAssetLoader | Adjusts the secondary base pointer before scalar deletion. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae8460` | `DelayedAssetLoader::Destroy` | DelayedAssetLoader | Destroys the delayed asset loader and all embedded state. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae85b0` | `DelayedAssetLoader::ScalarDeletingDtor` | DelayedAssetLoader | Implements scalar deletion for DelayedAssetLoader. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00caa1a0` | `DateWindowCheck` |  | Date-window filter for a loading-screen entry; called by the entry filter 00caa9e0. | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00caa9e0` | `LoadingScreenEntry_IsEligible` |  | Eligibility filter for loading-screen XML entries: membership, country, locale, required age and date window. | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00caae20` | `LoadingScreenXml_Load` |  | Loads the loading-screen definition file that supplies the splash art entries and their text. | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00ce2320` | `MiniGamePreloadLoad` |  | Loads a mini game preload file list: rejects duplicate preload entries and dispatches each entry type (CompositeEffect, ParticleEmitter, SoundEmitter, ModelMaterial, LightEmitter, CameraEffect, MeshEmitter) to its handl… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00d4a9e0` | `ReadTerrainTileRecord` |  | Parses terrain tile records; the FloraData array vtable and direct chunk-loader caller anchor this interpretation. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4c2c0` | `PrepareChunkDecodeBuffers` | GameRuntimeTerrain::DecompressChunkData… | Prepares the decompression job's buffers and decode state. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4c330` | `LogUnavailableChunkAsset` | GameRuntimeTerrain::Chunk | Chunk virtual method for the AssetUnavailable result path. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4d720` | `DecompressChunkDataJobCtor` | GameRuntimeTerrain::DecompressChunkData… | Constructs the async terrain chunk decompression job. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4d7e0` | `DecompressChunkDataJobDtor` | GameRuntimeTerrain::DecompressChunkData… | Destroys buffers and reference state owned by the decompression job. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4d8a0` | `DecompressChunkDataJobScalarDeletingDtor` | GameRuntimeTerrain::DecompressChunkData… | Scalar deleting destructor for the decompression job. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e3a0` | `CopyStringRecordFields` |  | Copies one raw terrain record layout; the precise record type is inferred from its list callers. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e470` | `CopyStringRecordFields` |  | Copies one raw terrain record layout; the precise record type is inferred from its list callers. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e560` | `CopyStringRecordFields` |  | Copies one raw terrain record layout; the precise record type is inferred from its list callers. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e610` | `ConfigureRuntimeTerrainAssets` |  | Configures runtime terrain asset availability for the chunk-loading path. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e6b0` | `ParseChunkHeaderAndCreateDecompressionJob` |  | The chunk asset path distinguishes GCNK and GCK2 and prepares a DecompressChunkDataJob; the original server is not involved. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e9a0` | `InstallFallbackTerrainResource` |  | Handles the named fallback terrain resource after the secondary job completes. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4ead0` | `FormatGcnkFilename` |  | Builds the GCNK asset filename for a terrain chunk. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4eb30` | `ChunkBuildAssetPaths` | GameRuntimeTerrain::Chunk | Chunk virtual method that prepares and forwards both supported chunk asset paths. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4ec80` | `HandleChunkAssetResult` | GameRuntimeTerrain::Chunk | Chunk virtual method for processing an incoming asset result and dispatching its parse failure status. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `01048570` | `ItemClassDefinitionManager::LoadDefinitions` | ItemClassDefinitionManager | Virtual slot 1 of ItemClassDefinitionManager: loads ItemClasses.txt into the item-class table, one record per non-comment line. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01097160` | `AssetLoadFailureDispatch` |  | Asset load failure path: logs the failure to AssetFailure.log and dispatches it to every waiting entry with the same name; caller set 010983f0 and 010984f0 is in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `01097c10` | `AssetDumpToFile → AssetDumpByExtensionToFile` |  | Writes the asset dump report (all assets or those matching a filter) to AssetDump.txt; the caller is not identified in this shard. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |
| `0123bbc0` | `Streaming::SOEAssetLoaderHandler::Ctor` | Streaming::SOEAssetLoaderHandler | Constructor of the SOE asset loader handler; it installs the base interface vtable, then its own. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123bd80` | `Streaming::SOEAssetLoaderHandler::ScalarDeletingDtor` | Streaming::SOEAssetLoaderHandler | Vtable slot [0] of Streaming::SOEAssetLoaderHandler: the scalar deleting destructor. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123bde0` | `Streaming::StreamingRequestStreamedAssets::Ctor` | Streaming::StreamingRequestStreamedAsse… | Constructor of the streamed-assets request handler; it installs the base action-handler vtable, then its own. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123be80` | `Streaming::StreamingRequestStreamedAssets::ScalarDeletingDtor` | Streaming::StreamingRequestStreamedAsse… | Vtable slot [0] of Streaming::StreamingRequestStreamedAssets: the scalar deleting destructor. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## 🟢 likely (42)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0071af10` | `AssetRequester::Destructor` | AssetManagement::AssetRequestInterface:… | Destructor body stores the AssetRequester vtable and invokes a member virtual at byte offset 0x18 with (this, 0) when the member exists; interpreting it as releasing the asset request is inferred from the class context. | [[Native Map/Shards/S0149 00719de0-0071efe0|S0149]] |
| `00840ab0` | `FlashLoaderJobCreateMovie` | FlashLoaderJob | creates the Scaleform movie for a FlashLoaderJob and keeps the handle at +0x120 | [[Native Map/Shards/S0202 0083ea10-00844c00|S0202]] |
| `00840ad0` | `FlashLoaderJobDtorBody` | FlashLoaderJob | destructor body of FlashLoaderJob: releases the movie and the name string, then the SyncJob/Job base | [[Native Map/Shards/S0202 0083ea10-00844c00|S0202]] |
| `009cdbc0` | `AdvanceQueueUnlessState1Or2` |  | Called by WaitForWorldReady while polling loader readiness. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cdbe0` | `AppendStringPairNodeAtTail` |  | Appends one parsed record to the loader entry list. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cdee0` | `UnlinkAndDestroyListNode` |  | Used by the FileInfo list drain/destructor; this function's body itself is generic list-node cleanup. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009d25a0` | `VehiclePartProcessor::GetOrCreateShipCombatAssetProcessor` | VehiclePartProcessor | Lazily creates and caches the ShipCombat asset processor used by VehiclePartProcessor. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a5c690` | `LoadingScreenDataBaseDtor → LoadingScreenData::DtorBody` | LoadingScreenData | Destroy loading-screen data members and its reference-counted base. | [[Native Map/Shards/S0300 00a59220-00a5da20|S0300]] |
| `00ae4fa0` | `NextChunkedRequestRecord` |  | Provides the next request record from the chunked pool. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae5560` | `PopChunkedTreeRecord` |  | Retrieves a record from a tree-tracked pool. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae5bc0` | `InitializePendingRequestStorage` |  | Initializes backing storage used by the delayed request queue. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae5c20` | `PopPendingDequeRecord` |  | Retrieves a pending record and conditionally runs deque cleanup. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6250` | `DtorWrapperFor51E0` |  | Provides an unwind-safe FileCreateJob cleanup wrapper. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae62a0` | `ReturnRecordToDequePool` |  | Returns a pending record while conditionally holding the deque lock. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6380` | `ConsumePendingDequeTail` |  | Consumes completed pending requests and releases spare deque blocks. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6460` | `InitializePendingRequestDeque` |  | Initializes the embedded pending-request deque state. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6540` | `ConsumePendingDequeHead` |  | Consumes completed pending requests and releases spare deque blocks. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae65e0` | `AppendTrackedHashListNode` |  | Adds a newly allocated tracked entry to both list and bucket chains. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6690` | `AllocateFileCreateJobNode` |  | Allocates a FileCreateJob node from the pending-request pool. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6720` | `ResetDelayedAssetLoaderState` |  | Resets delayed asset loading state and reports the new status. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae67e0` | `AllocateTrackedAssetRequestId` |  | Allocates a unique tracked request identifier within the current bucket chain. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6940` | `InitializeAndReleaseJobNode` |  | Allocates and releases a short-lived FileCreateJob node around a helper call. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6d00` | `PumpUntilElapsedThreshold` |  | Waits for a bounded interval while advancing delayed asset loading. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae6d80` | `PumpUntilLoaderIdle` |  | Waits until delayed asset loading becomes idle. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7900` | `TrackedAssetMapDtorWrapper` |  | Provides an unwind-safe wrapper for map cleanup. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7950` | `BeginDeferredLoadMode3` |  | Starts one deferred asset-loading mode or reports that the loader is busy. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae79b0` | `BeginDeferredLoadMode1` |  | Starts deferred asset loading in state 1 or reports that the loader is busy. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7a10` | `BeginDeferredLoadMode2` |  | Starts deferred asset loading in state 2 or reports that the loader is busy. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae7a60` | `ShutdownDelayedAssetLoader` |  | Stops delayed loading and releases its owned resources and callback registration. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00d4b390` | `PrepareTileResourceSlots` |  | Called by Tile::CreateAll to prepare the tile resource state; the tile association comes from that caller. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4bc20` | `DownsamplePackedPixels32` |  | Called by the terrain texture conversion routines while building lower-resolution levels. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4be60` | `DownsamplePackedPixels16` |  | Called by terrain texture conversion routines while building lower-resolution levels. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4c0c0` | `CopyCountedWordsAndAdvanceSource` |  | Used by the GCNK/GCK2 chunk-header reader to copy fixed-width values from its input buffer. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4c360` | `ConvertTerrainTextureMipLevels` |  | Called by the chunk asset reader while converting texture data. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4c750` | `ConvertTerrainTextureMipLevelsPacked` |  | Selected by the chunk reader for a packed-texture branch. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4cc20` | `ConvertTerrainTextureMipLevelsExpanded` |  | Selected by the chunk reader for a terrain texture format branch. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4d020` | `ConvertTerrainTextureMipLevels16` |  | Selected by the chunk reader for a terrain texture format branch. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4edc0` | `CopyStringRecordFields` |  | Record-copy helper called by the linked-list append path. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4ee10` | `CopyStringRecordFields` |  | Record-copy helper called by the linked-list append path. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `0123bd00` | `CreateStaticLoaderHandler` |  | Creates the SOE asset-loader handler object and publishes it in the global DAT_01cbcc80. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123be50` | `Streaming::StreamingRequestStreamedAssets::GetActionName` | Streaming::StreamingRequestStreamedAsse… | Returns the action name of the streamed-assets request handler. | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |
| `0123be70` | `Streaming::StreamingRequestStreamedAssets::GetTraceLabel` | Streaming::StreamingRequestStreamedAsse… | Diagnostic label of the streamed-assets request handler (action, file and line). | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## 🟡 speculative (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ca9a20` | `SpaceListMatchCountry ?` |  | Tokenises the string argument on spaces (00793e30 with separator 0x20) and compares each token with the string at *(this+4)+0x714 through 006fc5b0, with a substring test (004019e0) and the literal at DAT_018365b0 as a f… | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00ca9de0` | `SpaceListMatchLocale ?` |  | Tokenises the string argument on spaces (00793e30 with separator 0x20) and compares each token with the value returned by 0097a180(1) through 006fc5b0, with a substring test (004019e0) and the literal DAT_018365b0 fallb… | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00ce1e60` | `EnsureSubobject4ec_CallFn_010c2cd0 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1ef0` | `EnsureSubobject4ec_CallFn_010c29d0 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce1f80` | `EnsureSubobject4ec_CallFn_010c2a50 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce2010` | `EnsureSubobject4ec_CallFn_010c2ad0 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce20a0` | `EnsureSubobject4ec_CallFn_010c2b50 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce2130` | `EnsureSubobject4ec_CallFn_010c2bd0 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce21c0` | `EnsureSubobject4ec_CallFn_010c2c50 ?` |  | Thiscall with one stack argument: when that argument is nonzero, and this+0x14 is null, allocates 0x4ec bytes with FUN_00d833a6 and constructs it with FUN_010c28a0 into this+0x14 (0 when allocation fails); then calls FU… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `0123bf70` | `Streaming::StreamingRequestStreamedAssets::BuildRequestFields ? ⚠rename` | Streaming::StreamingRequestStreamedAsse… | Virtual slot [2] of Streaming::StreamingRequestStreamedAssets::vftable (904 bytes): assembles several named string fields (msgName, msgData, imgData, locale, defaults, initial) through FUN_012eb250 and FUN_012eb980/FUN_… | [[Native Map/Shards/S0654 012393a0-0123f010|S0654]] |

## ⚪ unknown (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00840a20` | `— ⚠reject` | FlashLoaderJob | constructs a FlashLoaderJob, a SyncJob-based loader job that holds a movie name string at +0x110 and a movie handle at +0x120 | [[Native Map/Shards/S0202 0083ea10-00844c00|S0202]] |
| `009cda80` | `— ⚠reject` |  | Selects and dispatches the next loader entry from its queue. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cdf80` | `— ⚠reject` | ClientStartingZoneLoader | Tears down ClientStartingZoneLoader-owned nested state. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00ae57b0` | `— ⚠reject` | DelayedAssetLoader::FileCreateJob | Runs the delayed file-creation job and stores its check result. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ca97e0` | `— ⚠reject` |  | Copies the source text (IString at param_2+4/+8) into a local IString, then walks it token by token with 006fc270 into a StringFixed<1024>: a token that is empty or starts with '#' is skipped; for every other token the … | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00caa4a0` | `— ⚠reject` |  | Advances one loading-screen entry's state at this+0x18 unless it is -1 or 2: builds the entry's asset path from its Filename (00402c50 truncation on the last path separator), asks the resource interface at this+8 whethe… | [[Native Map/Shards/S0410 00ca6401-00cab4e0|S0410]] |
| `00d4ed70` | `— ⚠reject` |  | Record-copy helper called by the linked-list append path. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `010456c0` | `— ⚠reject` | ItemCategoryManager | Virtual slot 1 of ItemCategoryManager: loads item category and category group definitions from ItemCategories.txt and ItemCategoryGroups.txt. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010479f0` | `— ⚠reject` |  | Reads a definition source in 0x800-byte chunks and appends a 0x34-byte record per chunk. Called from 01047e20, which is labelled ContentPackManager::LoadDefinitionsFromBuffer in nmq (not verified here). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049520` | `— ⚠reject` | StringHashToValueManager | Virtual slot 1 of StringHashToValueManager: loads a prefixed text table that maps a hashed name to a numeric value and string record. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `0104a2b0` | `— ⚠reject` | RailDefinitionManager | Virtual slot 1 of RailDefinitionManager: loads the rail definitions from Rails.txt and their points from RailPoints.txt, logging invalid rows to RailErrors.log. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01097a90` | `— ⚠reject` |  | Validates an asset record's length: zero-length assets are logged to AssetErrors.log and queued as errors; non-empty assets are handed to the object's slot [3]. | [[Native Map/Shards/S0584 01093ee0-01099b80|S0584]] |

