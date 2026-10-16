# Class: SoeUtil::Array<Deep::FontFace*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d007d0` | `InlineArrayFreeStorage` | ✅ | virtual | Frees the supplied storage only when its pointer differs from the object's aligned inline buffer address. |
| `00d00a20` | `InlineArrayCleanupStorage` | ✅ | dtor | Restores the inline-array vtable, frees heap storage only when it differs from the aligned inline buffer, then restores and clears the zero-inline-capacity base array. |
| `00d00ae0` | `InlineArrayAllocateStorage` | ✅ | virtual | Returns the aligned inline buffer with capacity four for requests of four or fewer elements; larger requests delegate to the dynamic array allocator. |
| `00d00ff0` | `InlineArrayInitialize` | ✅ | ctor | Initializes the zero-inline-capacity array base with null pointer, size and capacity fields, then installs the four-element inline-array vtable. |
| `00d010b0` | `InlineArrayScalarDeletingDtor` | ✅ | dtor | Runs inline-array cleanup and frees the array object only when deleting-flag bit 0 is set; otherwise returns the original pointer. |
| `00d01130` | `AppendArrayContentsAndClearCounts ? ⚠rename` | 🟡 | method | Appends the supplied array's live dwords, clears its logical count, appends the receiver's live range through the same helper, resets that count, and releases a null storage pointer. |
