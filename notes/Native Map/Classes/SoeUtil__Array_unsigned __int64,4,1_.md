# Class: SoeUtil::Array<unsigned __int64,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd9d0` | `UInt64InlineArray::FreeBuffer` | ✅ | virtual | Does not free the inline buffer at this+0x17 aligned to 8; frees any other buffer with alignment 8. |
| `008d3ae0` | `ArrayUInt64_4_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the 4-element unsigned-64-bit-array vtable): runs the array record teardown FUN_008c77e0 (rewrites the array vtable, frees the heap buffer through the engine FreeMemory routi… |
