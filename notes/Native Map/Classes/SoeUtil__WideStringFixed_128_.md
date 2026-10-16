# Class: SoeUtil::WideStringFixed<128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00709ed0` | `WideFixedStringAllocate → WideString128BufferAlloc` | ✅ | virtual | Returns the inline wide-character buffer with capacity 0x104 for requests below 0x105; otherwise calls the heap allocator. |
| `00709f00` | `WideFixedStringFree` | ✅ | virtual | Frees the supplied wide buffer with alignment zero unless it equals the inline buffer at this+0x10. |
| `0070bfc0` | `WideStringScalarDeletingDtor` | ✅ | dtor | Runs the 128-element wide-string destructor and frees the object when the delete flag bit is set. |
