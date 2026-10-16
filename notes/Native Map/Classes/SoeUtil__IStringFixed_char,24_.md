# Class: SoeUtil::IStringFixed<char,24>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7950` | `SoeUtil_IStringFixed24_Dtor` | ✅ | dtor | Stores IStringFixed<char,24> vftable at this. If count +0xc is positive, decrements the refcount at buffer-4; when the decremented count is nonpositive, calls this object vslot +8 with the refcount-header pointer as its… |
| `008c7a10` | `SoeUtil_IStringFixed24::VSlot1_InlineOrHeap` | ✅ | virtual | Bytes: cmp eax,0x1c; jg heap; otherwise writes 0 to the byte at param_4, writes 0x1c to *param_3 and returns this+0x10 (inline storage). Heap path: tail-jumps to FUN_00401cb0 with the same stack arguments. Returns with … |
| `008c7a40` | `SoeUtil_IStringFixed24::FreeIfNotInline` | 🟢 | virtual | Bytes: if the pointer argument is not this+0x10 (the inline buffer), calls FUN_00401c00(pointer, 0) with alignment 0. Returns with ret 4. |
| `008c9330` | `SoeUtil_IStringFixed24_ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_008c7950 on this, then frees this with _free when bit 0 of the flag byte is set. Returns this. |
| `008d3fa0` | `Ctor_IStringFixed24 ?` | 🟡 | ctor | Constructor of a fixed 24-byte string: initializes the SoeUtil::IString<char> base (vtable, empty buffer 01be48fc, zero length/capacity), overwrites the vtable with SoeUtil::IStringFixed<char,24>::vftable and assigns th… |
| `00b284b0` | `SoeUtil::IStringFixed24Ctor` | ✅ | ctor | Constructor of the 24-byte fixed inline string: writes SoeUtil::IString<char>::vftable, points the buffer field (this+4) at the global empty/inline buffer DAT_01be95dc, zeroes this+8 and this+0xc, then writes SoeUtil::I… |
| `00b2b080` | `IStringFixed24::Constructor` | ✅ | ctor | Initializes the IString base and writes the fixed 24-byte string specialization vtable. |
