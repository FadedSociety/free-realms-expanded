# Class: SoeUtil::IStringFixed<char,255>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7e00` | `SoeUtil_IStringFixed255::Ctor` | ✅ | ctor | Stores SoeUtil::IString<char>::vftable at this+0, points this+4 at the static empty buffer DAT_01be48fc, zeroes this+8 and this+0xc, then stores SoeUtil::IStringFixed<char,255>::vftable at this+0. Returns void (fastcall… |
| `008c7e60` | `SoeUtil_IStringFixed255::VSlot1_InlineOrHeap` | ✅ | virtual | Bytes: cmp eax,0x103; jg heap. For requests up to 0x103 writes 0 to the byte at param_4, writes 0x103 to *param_3 and returns this+0x10 (inline storage). Otherwise tail-jumps to FUN_00401cb0 with the same stack argument… |
| `008c7e90` | `SoeUtil_IStringFixed255::FreeIfNotInline` | 🟢 | virtual | Bytes: if the pointer argument is not this+0x10, calls FUN_00401c00(pointer, 0). Returns with ret 4. |
| `008c7eb0` | `SoeUtil_IStringFixed255_Dtor` | ✅ | dtor | Stores IStringFixed<char,255> vftable at this. If count +0xc is positive, decrements the refcount at buffer-4; when the decremented count is nonpositive, calls this object vslot +8 with the refcount-header pointer as it… |
| `008c9490` | `SoeUtil_IStringFixed255_ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_008c7eb0 on this, then frees this with _free when bit 0 of the flag byte is set. Returns this. |
| `008d4180` | `Ctor_IStringFixed255` | 🟢 | ctor | Constructor of a fixed 255-byte string: initializes the SoeUtil::IString<char> base (vtable, empty buffer 01be48fc, zero length/capacity), overwrites the vtable with SoeUtil::IStringFixed<char,255>::vftable and assigns … |
