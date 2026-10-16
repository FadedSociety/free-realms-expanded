# Class: SoeUtil::Array<SoeUtil::StrongType<int,BaseItemSearchEngine::SoeUtilSignatureKey,1>,1024,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009901d0` | `InlineSignatureKeyArrayFreeMemory` | ✅ | method | Compares the supplied pointer with the array's aligned inline-buffer address and frees it with four-byte alignment only when they differ. |
| `009906b0` | `InlineSignatureKeyArrayDtor` | ✅ | dtor | Restores the inline array vtable, clears its size, frees dynamic storage only when the buffer is not the object's aligned inline region, then switches to and clears the base dynamic array state. |
| `00990770` | `InlineSignatureKeyArrayCapacityBuffer` | ✅ | method | Returns the array's inline buffer and capacity 0x400 when the requested size is at most 0x400; otherwise delegates capacity calculation to the dynamic helper. |
| `00990a20` | `InlineSignatureKeyArrayCtor` | ✅ | ctor | Installs the dynamic-array vtable, clears data, size, and capacity fields, then installs the 1024-inline-element array vtable. |
| `00990a80` | `InlineSignatureKeyArrayScalarDeletingDtor` | ✅ | dtor | Calls the inline-array destructor, conditionally frees the object according to the low flag bit, and returns the object pointer when retained. |
| `00990f60` | `ArrayRangeOperationsAndClear` | 🟢 | virtual | Reads each input array data pointer and count, calls the shared range-append helper, clears each input count, calls the helper with an empty range, and releases temporary storage. The helper calls omit implicit this arg… |
