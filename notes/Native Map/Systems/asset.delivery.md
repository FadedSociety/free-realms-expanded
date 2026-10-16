# System: asset.delivery

69 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (32)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00989810` | `AssetRequester::Destructor` | AssetManagement::AssetRequestInterface:… | Tears down the AssetRequester portion embedded in pending image loads. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `00989b70` | `ImageAsset2::ScalarDeletingDestructor` | ImageDirectory::ImageAsset2 | Provides scalar deletion for an ImageAsset2 instance. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `00989dc0` | `ImageDirectory::PendingLoad2::ScalarDeletingDestructor` | ImageDirectory::PendingLoad2 | Provides scalar deletion for a PendingLoad2 object. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098ae70` | `BuildIdi2KeyAndQueryAssetManager` |  | Combines an idi2-prefixed key with an Asset-to-ImageAsset2 manager query. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098af60` | `ImageAsset2::Method2` | ImageDirectory::ImageAsset2 | Virtual slot 2 implementation for ImageAsset2; the exact semantic meaning of the produced string is not established. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098b0d0` | `ImageDirectory::PendingLoad2::ReportUnavailableAsset` | ImageDirectory::PendingLoad2 | Reports a failed image asset request and clears its associated references and pending-map entry. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098d860` | `ImageDirectory::Destructor` | ImageDirectory | Releases ImageDirectory's lookup and pending-request state during teardown. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098dab0` | `ImageDirectory::Constructor` | ImageDirectory | Initializes image request tracking and image-definition lookup state. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `01099be0` | `PendingRequest::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryIndirect:… | Deleting destructor for AssetDeliveryIndirect::PendingRequest objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099da0` | `ReadJob::ReadJob` | AssetManagement::AssetDeliveryIndirect:… | Creates the read job that fetches one asset named by param_3 for owner param_2. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099e30` | `ReadJob::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryIndirect:… | Deleting destructor for ReadJob objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099f90` | `WriteJob::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryIndirect:… | Deleting destructor for WriteJob objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099fb0` | `DecompressJob::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryIndirect:… | Deleting destructor for DecompressJob objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099fd0` | `DecompressJob::Decompress` | AssetManagement::AssetDeliveryIndirect:… | Decompresses a delivered asset envelope (A1B2C3D4 header plus zlib body) into the job buffer before the write job. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109a390` | `AssetDeliveryIndirect::InitPackStorage` | AssetManagement::AssetDeliveryIndirect | One-time local pack storage setup for indirect asset delivery: directory, transport by URL scheme, and pack file groups. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109ae60` | `RemoveDeadAssets` |  | Removes local assets whose names no longer appear in the manifest, logging each removal to DeadAssets.txt. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109afc0` | `AssetDeliveryIndirect::GetAssetData` | AssetManagement::AssetDeliveryIndirect | Asset request entry point: rejects unknown names, rejects duplicates already pending, then either queues a local ReadJob or forwards the request to the remote handler. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109b2a0` | `AssetDeliveryIndirect::RetryRequest` | AssetManagement::AssetDeliveryIndirect | Retries a pending asset request once more through the remote handler, or logs that the asset is missing from the manifest. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109b400` | `AssetDeliveryIndirect::AssetReceived` | AssetManagement::AssetDeliveryIndirect | Delivery callback for a received asset: a compressed body with a manifest record goes to the decompress job; any other body goes straight to the write job. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109bb00` | `AssetDeliveryIndirect::AssetErrorReceived` | AssetManagement::AssetDeliveryIndirect | Error callback for a delivered asset: fails every pending request for that name and logs the error. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109bd00` | `AssetDeliveryIndirect::OnManifestCrcReceived` | AssetManagement::AssetDeliveryIndirect | Manifest CRC callback: parses a local manifest whose size matches the announced CRC size, otherwise requests the manifest again. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109bea0` | `AssetDeliveryIndirect::ManifestReceived` | AssetManagement::AssetDeliveryIndirect | Manifest body callback: verifies the downloaded manifest CRC, saves it, and parses it. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109c000` | `AssetDeliveryIndirect::OnFinished` | AssetManagement::AssetDeliveryIndirect | Job completion dispatcher: delivers, retries or fails the pending requests when a read, write or decompress job finishes. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109c720` | `AssetDeliveryInterface::~AssetDeliveryInterface` | AssetManagement::AssetDeliveryInterface | Tears down the AssetDeliveryInterface base and its two pending-entry lists. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109c800` | `AssetDeliveryInterface::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryInterface | Deleting destructor for AssetDeliveryInterface objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109c820` | `AssetDeliveryInterface::AssetDeliveryInterface` | AssetManagement::AssetDeliveryInterface | Constructs the AssetDeliveryInterface base with its two pending-entry lists. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d960` | `AssetDeliveryIndirect::AssetDeliveryIndirect` | AssetManagement::AssetDeliveryIndirect | Creates an AssetDeliveryIndirect object for the indirect (manifest and HTTP) asset delivery path. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109db20` | `AssetDeliveryIndirect::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryIndirect | Deleting destructor for AssetDeliveryIndirect objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109dfa0` | `AssetDeliveryDirect::FileLoader::FileLoader` | AssetManagement::AssetDeliveryDirect::F… | Creates one file loader owned by a FileRequestQueue for AssetDeliveryDirect. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e030` | `AssetDeliveryDirect::FileLoader::~FileLoader` | AssetManagement::AssetDeliveryDirect::F… | Tears down a FileLoader object. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e080` | `AssetDeliveryDirect::FileLoader::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryDirect::F… | Deleting destructor for FileLoader objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e900` | `AssetDeliveryDirect::FileRequestQueue::ScalarDeletingDestructor` | AssetManagement::AssetDeliveryDirect::F… | Deleting destructor for FileRequestQueue objects. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0098b1e0` | `CreatePendingImageLoadForId` |  | Creates and registers a pending image load for a known image id. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0109a810` | `AssetDeliveryIndirect::FindManifestEntry` | AssetManagement::AssetDeliveryIndirect | Manifest record lookup for the AssetDeliveryIndirect methods: callers test the result against 0 to reject unknown names. The class binding rests on the caller 0109afc0 (a vtable method of AssetDeliveryIndirect). | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e3d0` | `AssetDeliveryDirect::InitFileLoaders` | AssetManagement::AssetDeliveryDirect | Builds the file request queue and the configured number of file loaders for AssetDeliveryDirect. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e590` | `FileRequestQueue::FindItemByNameAndRef` | AssetManagement::AssetDeliveryDirect::F… | Finds a queued file-load entry by its name and reference tag. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |

## 🟡 speculative (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01099e50` | `ReadJob::ReadIntoBuffer ?` | AssetManagement::AssetDeliveryIndirect:… | Virtual slot [2] of ReadJob. Allocates a 0x18-byte object (00d833a6 then the 0083fe90 constructor), stores it at this+0xb0, calls 010fb8e0 with this+0x60 and the new buffer; on failure releases the buffer reference at +… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109a1c0` | `StringFixed64KeyedCtor ?` |  | Constructs a StringFixed<64> at offset 0 of this through 00402a10, stores the int at *param_2 into the field at +0x64, and returns this. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109a6d0` | `PackFileSequenceProbe ?` |  | Builds the path <dir>/<name>_manifest.dat, tests it with 00704840 and calls 00704750 on it when present; then builds <dir>/<name>_NNN.dat for NNN = 0, 1, 2, ... and repeats the test and call until a numbered file is abs… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109a9a0` | `AssetDeliveryIndirect::ManifestLookupFromRequest ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [11] of AssetDeliveryIndirect. Converts the name at param_1+4 to an IString through 01093aa0, calls 0109a810 (manifest lookup) with it, and returns with no value. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109aa60` | `AssetDeliveryIndirect::HasManifestEntry ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [23] of AssetDeliveryIndirect, one stack argument (ret 4). Calls 0109a810 with the argument and returns 1 when the result is nonzero, else 0. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109acf0` | `ScalarDeletingDtor_Via01099b80 ?` |  | Scalar deleting destructor shape: calls the body 01099b80 on this, then calls _free on the object when bit 0 of the stack argument is set; returns the object pointer. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109b310` | `AssetDeliveryIndirect::RemoveLocalAsset ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [7] of AssetDeliveryIndirect. Converts the name at param_1+4 to an IString, looks it up with 0109a810, and when a manifest record exists calls 010fbb00 on the name and, if that result is not -1, calls 010fb… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d820` | `AssetDeliveryIndirect::GetByte624 ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [8] of AssetDeliveryIndirect (bytes 8a 81 24 06 00 00 c3): returns the byte at this+0x624. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d850` | `Thunk_0109db20_AdjustM61c ?` | AssetManagement::AssetDeliveryIndirect | Adjustor thunk (bytes 81 e9 1c 06 00 00 then jmp rel32): subtracts 0x61c from ECX and jumps to 0109db20. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d860` | `AssetDeliveryIndirect::GetByte625 ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [9] of AssetDeliveryIndirect (bytes 8a 81 25 06 00 00 c3): returns the byte at this+0x625. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d870` | `AssetDeliveryIndirect::GetDword644 ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [14] of AssetDeliveryIndirect (bytes 8b 81 44 06 00 00 c3): returns the dword at this+0x644. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d880` | `AssetDeliveryIndirect::ForwardTo16b78 ?` | AssetManagement::AssetDeliveryIndirect | Virtual slot [10] of AssetDeliveryIndirect, one stack argument (ret 4): adds 0x16b78 to ECX, pushes the result as a stack argument, reloads ECX from the caller's argument and calls 006f9e60. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109dbc0` | `AssetDeliveryDirect::ReturnThis ?` | AssetManagement::AssetDeliveryDirect | Virtual slot [13] of AssetDeliveryDirect (bytes 8b c1 c3): returns its this pointer unchanged. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109dbd0` | `AssetDeliveryDirect::SubobjectAt640 ?` | AssetManagement::AssetDeliveryDirect | Virtual slot [17] of AssetDeliveryDirect (bytes 8d 81 40 06 00 00 c3): returns this+0x640 as the address of the subobject. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109dbe0` | `AssetDeliveryDirect::TestOrForwardToQueue ?` | AssetManagement::AssetDeliveryDirect | Virtual slot [20] of AssetDeliveryDirect, one stack argument: returns 1 when 01093820 accepts the argument; otherwise returns the result of virtual slot [2] of the object at this+0x638 (the FileRequestQueue) called with… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e140` | `ClearItemRefInLists ?` |  | Takes the lock 00fc45f0, clears the word at +0x28 of every node in the lists with heads at this+8 and this+0x14 whose +0x28 word equals param_2, and releases the lock 00fc4610 unless this is the -0x20 sentinel. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e1e0` | `FileRequestQueue::ContainsItemRef ?` | AssetManagement::AssetDeliveryDirect::F… | Virtual slot [2] of FileRequestQueue, one stack argument: under the lock 00fc45f0 searches the lists at +8 and +0x14 for a node whose word at +0x28 equals the argument, releases the lock, and returns 1 when found, else … | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e360` | `AssetDeliveryDirect::ReleaseLoadersAndQueue ?` | AssetManagement::AssetDeliveryDirect | Releases the loader list and the file request queue of AssetDeliveryDirect. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e570` | `AssetDeliveryDirect::ClearItemRefs ?` | AssetManagement::AssetDeliveryDirect | Virtual slot [19] of AssetDeliveryDirect, one stack argument: calls 010937d0 on this with the argument, then calls 0109e140 on the FileRequestQueue at this+0x638 with the same argument. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109ea50` | `DirectFileRequestStep ?` |  | Per-request file step: when the callback pointer at this+0x28 is set, allocates a 0x18-byte RefArray buffer (00d833a6 and 0083fe90), probes the request through 00703e10, sizes and fills the buffer through 00702dd0 and 0… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |

## ⚪ unknown (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00799a30` | `— ⚠reject` |  | Thiscall (ecx = this, one stack argument). Scans the list at arg+0x30 for entries whose +0x60 word equals 1, then for each entry builds two local SoeUtil::IString<char> values plus a List<IString<char>,-1> and calls vir… | [[Native Map/Shards/S0172 00798490-0079d890|S0172]] |
| `0098b000` | `— ⚠reject` |  | Builds a prefixed string and invokes the associated virtual method. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `01099c00` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect | Blocking asset load: uses the local copy when it is current, otherwise loads from the pack file and logs failures. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `01099ef0` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect:… | Creates a write job holding one received asset's data buffer under its name. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109a170` | `— ⚠reject` |  | Calls the PendingRequest body destructor 01099b80 on a stack-local object and returns; no free is performed. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109aa80` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect:… | Creates the decompress job that turns one received compressed asset body into its raw buffer. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109b650` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect | Manifest text parser: builds the manifest records from the comma-separated text, then removes dead assets and marks the manifest loaded. Its callers 0109bd00 and 0109bea0 pass the downloaded or local manifest text. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109d640` | `— ⚠reject` | AssetManagement::AssetDeliveryIndirect | Tears down an AssetDeliveryIndirect object: its handler, job queue, manifest and pending maps, and its AssetDeliveryInterface base. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e0e0` | `— ⚠reject` |  | Takes the lock 00fc45f0, pops the head node of the list at this+0 through 0109dcb0, appends it to the tail of the list with head at this+0x14, tail at +0x18 and count at +0x1c (becoming the head when that list is empty)… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e2e0` | `— ⚠reject` | AssetManagement::AssetDeliveryDirect::F… | Constructs the file request queue that AssetDeliveryDirect uses for its file loaders. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e4d0` | `— ⚠reject` | AssetManagement::AssetDeliveryDirect | Virtual slot [2] of AssetDeliveryDirect. Takes the lock 00fc45f0 when the counter at +0x628 is positive. When the pair at +0x9978/+0x997c is set and 008bb840 accepts the id at +0x61c, it calls 01538790 instead of forwar… | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e7c0` | `— ⚠reject` |  | Takes the lock 00fc45f0, calls 0109e750 twice on this (one drain per list), and releases the lock 00fc4610 unless this is the -0x20 sentinel. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |
| `0109e890` | `— ⚠reject` | AssetManagement::AssetDeliveryDirect::F… | Tears down the file request queue of AssetDeliveryDirect. | [[Native Map/Shards/S0585 01099be0-0109ea50|S0585]] |

