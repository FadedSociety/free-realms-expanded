# Class: SoeUtil::HashListMap<int,DelayedAssetLoader::TrackedAsset,256,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae7840` | `HashListMap::AdjustThis414ThenPopNode` | ✅ | virtual | Adds 0x414 to ECX and jumps to the shared pooled-node pop implementation at 00ae7760. |
| `00ae7850` | `HashListMap::AdjustThis414ThenPushNode` | ✅ | virtual | Adds 0x414 to ECX and jumps to the shared pooled-node push implementation at 00ae77d0. |
| `00ae7860` | `SoeUtil::HashListMap::TrackedAssetScalarDeletingDtor` | ✅ | dtor | Calls the map destructor, frees the object when flag bit 0 is set, and returns its pointer otherwise. |
| `00ae7880` | `SoeUtil::HashListMap::TrackedAssetCtor` | ✅ | ctor | Installs the base map vtable, clears the count and 256 bucket heads, installs the bounded-map vtable, and initializes the embedded deque. |
