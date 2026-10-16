# Class: SoeUtil::WideStringFixed<64>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00709f80` | `WideFixedStringAllocate` | ✅ | virtual | Returns the inline wide-character buffer with capacity 0x84 for requests below 0x85; otherwise calls the heap allocator. |
| `00709fb0` | `WideFixedStringFree` | ✅ | virtual | Frees the supplied wide buffer with alignment zero unless it equals the inline buffer at this+0x10. |
| `0070c020` | `WideStringScalarDeletingDtor → WideStringFixed64ScalarDeletingDtor` | ✅ | dtor | Runs the 64-element wide-string destructor and frees the object when the delete flag bit is set. |
