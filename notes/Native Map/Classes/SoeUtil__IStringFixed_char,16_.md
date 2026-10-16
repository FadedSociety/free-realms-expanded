# Class: SoeUtil::IStringFixed<char,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00709b30` | `FixedStringAllocate → Char16FixedStringBufferAlloc` | ✅ | virtual | For requested capacity below 0x15 returns the inline buffer at this+0x10 with capacity 0x14; otherwise calls the heap allocator. |
| `00709b80` | `IStringDtor → IStringFixedChar16Dtor` | ✅ | dtor | Releases the reference-counted character buffer when owned, resets pointer and length fields to static empty storage, and restores the base IString vtable. |
| `00709fd0` | `IStringFixedScalarDeletingDtor → IStringFixedChar16ScalarDeletingDtor` | ✅ | dtor | Runs the fixed-string destructor and frees the object only when the delete flag bit is set. |
