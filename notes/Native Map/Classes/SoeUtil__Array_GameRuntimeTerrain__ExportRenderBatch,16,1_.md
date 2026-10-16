# Class: SoeUtil::Array<GameRuntimeTerrain::ExportRenderBatch,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4d6a0` | `FreeArrayStorageUnlessInline` | ✅ | method | Frees the supplied pointer with alignment 4 only when it differs from this object's aligned inline buffer. |
| `00d4da20` | `ArrayDtor` | ✅ | dtor | Clears the auxiliary count, frees heap storage only when it is not the inline buffer, then runs dynamic-array cleanup. |
| `00d4dae0` | `InlineCapacityOrAllocate` | ✅ | method | For requests up to 16, writes capacity 16 and returns the aligned inline buffer; larger requests call the shared allocator. |
| `00d4df10` | `InlineRenderBatchArrayCtor` | ✅ | ctor | Sets the dynamic-array vtable, clears data, size, and capacity, then installs the sixteen-element inline-array vtable. |
| `00d4df90` | `InlineRenderBatchArrayScalarDeletingDtor` | ✅ | dtor | Runs inline ExportRenderBatch-array cleanup, conditionally frees the array object when flag bit 0 is set, and otherwise returns its pointer. |
| `00d4e280` | `AssignRenderBatchArrayState → SwapRenderBatchArrayContents` | ✅ | method | Transfers records through the 16-byte array helper, clears the source and destination size fields, then invokes the storage free helper. |
