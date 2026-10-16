# Class: SoeUtil::Array<GameRuntimeTerrain::Tile,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4d420` | `AllocateTileArrayStorage → AllocateTileArrayStorageWithGrowthPolicy` | ✅ | method | Reuses storage in its capacity branches or allocates param_2 times 0x4CC bytes at alignment 4, writing the selected capacity through param_3. |
| `00d4d4c0` | `FreeArrayStorage` | ✅ | method | Passes the supplied pointer to the memory free helper with alignment 4 and returns. |
| `00d4d4e0` | `ArraySwapOrInvokeCleanup → ArraySwap` | ✅ | method | When param_3 is nonzero, calls param_2's vtable slot +0x0C; otherwise swaps the data pointer, size, and capacity fields between the two objects. |
| `00d4d8c0` | `ArrayDtor` | ✅ | dtor | Destroys live Tile elements through the array element callback, frees backing storage with alignment 4, and clears the data pointer. |
| `00d4db10` | `ArrayScalarDeletingDtor` | ✅ | dtor | Runs dynamic Tile-array cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |
