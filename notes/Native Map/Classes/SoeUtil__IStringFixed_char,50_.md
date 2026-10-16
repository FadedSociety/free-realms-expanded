# Class: SoeUtil::IStringFixed<char,50>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7c90` | `SoeUtil_IStringFixed50_Dtor` | ✅ | dtor | Stores IStringFixed<char,50> vftable at this. If count +0xc is positive, decrements the refcount at buffer-4; when the decremented count is nonpositive, calls this object vslot +8 with the refcount-header pointer as its… |
| `008c7d50` | `SoeUtil_IStringFixed50::VSlot1_InlineOrHeap` | ✅ | virtual | If param_2 is below 0x37 writes 0 to the byte at param_4, writes 0x36 to *param_3 and returns this+0x10 (inline storage). Otherwise calls FUN_00401cb0 (heap path) and returns its result. |
| `008c7d80` | `SoeUtil_IStringFixed50::FreeIfNotInline` | 🟢 | virtual | Bytes: if the pointer argument is not this+0x10, calls FUN_00401c00(pointer, 0). Returns with ret 4. |
| `008c9410` | `SoeUtil_IStringFixed50_ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_008c7c90 on this, then frees this with _free when bit 0 of the flag byte is set. Returns this. |
| `00ae27f0` | `StringFixed50BaseInit` | ✅ | ctor | Installs IString then IStringFixed vtables, points data at the fixed empty buffer, and clears string length and reference count. |
