# Class: SoeUtil::HashListMap<int,DelayedAssetLoader::TrackedAsset,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae3810` | `TrackedAssetHashListInit` | ✅ | ctor | Installs the 256-bucket map vtable, clears head/tail/count and zeroes 256 bucket heads, then returns this. |
| `00ae3d70` | `TrackedAssetHashAlloc` | ✅ | virtual | Initializes memory management, asks thread/global managers for a node allocation, and falls back to requesting 0x24 bytes aligned to 4. |
| `00ae3db0` | `TrackedAssetHashFree` | ✅ | virtual | Pushes pointer and alignment 4 to FreeMemory (00401c00), then returns. |
| `00ae69b0` | `SoeUtil::HashListMap::TrackedAssetDtor` | ✅ | dtor | Installs the HashListMap vtable and repeatedly removes the node at offset +8 until the map list is empty. |
| `00ae6a50` | `SoeUtil::HashListMap::TrackedAssetScalarDeletingDtor` | ✅ | dtor | Calls the map destructor body, frees the map when flag bit 0 is set, and returns its pointer otherwise. |
| `00ae6db0` | `SoeUtil::HashListMap::TrackedAssetBaseDtor` | ✅ | dtor | Installs the bounded-map vtable, removes all linked nodes, clears and frees its embedded alignment-4 tree, installs the base HashListMap vtable, and removes the remaining list nodes. |
