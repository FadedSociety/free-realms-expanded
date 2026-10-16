# Class: SoeUtil::Array<SoeUtil::StringFixed<64>,2,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095afa0` | `InlineArrayFreeStorage` | ✅ | virtual | Frees param_2 at alignment 4 only when it differs from the object's inline-buffer address (this+0x13 rounded down to four-byte alignment). |
| `0095e260` | `InlineArrayDestructor → SoeUtil::Array<SoeUtil::StringFixed<64>,2,1>::~Array` | ✅ | dtor | Installs the inline-two array vtable and calls FUN_009594c0 on this+8. If the data pointer at this+4 differs from the aligned inline-buffer address, calls FUN_00401c00 with that pointer and alignment 4; then clears this… |
| `0095e330` | `InlineArrayAllocateStorage → SoeUtil::Array<SoeUtil::StringFixed<64>,2,1>::Allo…` | ✅ | virtual | Returns the aligned inline buffer and capacity 2 for requests up to two elements, otherwise forwards to FUN_0095ae70. |
| `009626d0` | `ArrayStringFixed64Inline2Ctor` | ✅ | ctor | Installs the dynamic array vtable, zeros its three bookkeeping words, then installs the inline-capacity-two array vtable. |
| `009627a0` | `ArrayStringFixed64ScalarDeletingDestructor` | ✅ | dtor | Calls FUN_0095e260 to destroy the inline-two array; if flag bit 0 of param_2 is set, frees param_1; otherwise returns the original pointer. |
