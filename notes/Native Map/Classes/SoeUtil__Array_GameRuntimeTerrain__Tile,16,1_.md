# Class: SoeUtil::Array<GameRuntimeTerrain::Tile,16,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4d550` | `FreeArrayStorageUnlessInline` | ✅ | method | Frees the supplied pointer with alignment 4 only when it differs from this object's aligned inline buffer. |
| `00d4d8f0` | `ArrayDtor` | ✅ | dtor | Destroys live Tile elements, frees heap storage only when it is not the inline buffer, then runs dynamic-array cleanup. |
| `00d4d9c0` | `InlineCapacityOrAllocate` | ✅ | method | For requests up to 16, writes capacity 16 and returns the aligned inline buffer; larger requests jump to 00d4d420. |
| `00d4deb0` | `InlineTileArrayCtor` | ✅ | ctor | Sets the dynamic-array vtable, clears data, size, and capacity, then installs the sixteen-element inline-array vtable. |
| `00d4df70` | `InlineTileArrayScalarDeletingDtor` | ✅ | dtor | Runs inline Tile-array cleanup, conditionally frees the array object when flag bit 0 is set, and otherwise returns its pointer. |
