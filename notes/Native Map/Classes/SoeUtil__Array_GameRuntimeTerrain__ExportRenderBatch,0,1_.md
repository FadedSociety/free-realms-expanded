# Class: SoeUtil::Array<GameRuntimeTerrain::ExportRenderBatch,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4d570` | `AllocateRenderBatchArrayStorage → AllocateRenderBatchArrayStorageWithGrowthPoli…` | ✅ | method | Reuses storage in its capacity branches or allocates param_2 times 16 bytes at alignment 4, writing the selected capacity through param_3. |
| `00d4d610` | `FreeArrayStorage` | ✅ | method | Passes the supplied pointer to the memory free helper with alignment 4 and returns. |
| `00d4d630` | `ArraySwapOrInvokeCleanup → ArraySwap` | ✅ | method | When param_3 is nonzero, calls param_2's vtable slot +0x0C; otherwise swaps the data pointer, size, and capacity fields between the two objects. |
| `00d4d9f0` | `ArrayDtor` | ✅ | dtor | Clears the auxiliary count, frees the backing storage with alignment 4, and clears the data pointer. |
| `00d4db50` | `ArrayScalarDeletingDtor` | ✅ | dtor | Runs dynamic-array cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |
