# Class: SoeUtil::IStringFixed<unsigned_short,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c7a60` | `SoeUtil_IStringFixed32W_Dtor` | ✅ | dtor | Stores IStringFixed<unsigned_short,32> vftable at this. If count +0xc is positive, decrements the refcount at wide-buffer-4; when the decremented count is nonpositive, calls this object vslot +8 with the refcount-header… |
| `008c7b20` | `SoeUtil_IStringFixedW32::VSlot1_InlineOrHeap` | ✅ | virtual | Bytes: cmp eax,0x44; jg heap; otherwise writes 0 to the byte at param_4, writes 0x44 to *param_3 and returns this+0x10 (inline storage). Heap path tail-jumps to FUN_006facc0 with the same stack arguments. Returns with r… |
| `008c7b50` | `SoeUtil_IStringFixedW32::FreeIfNotInline` | 🟢 | virtual | Bytes: if the pointer argument is not this+0x10, calls FUN_00401c00(pointer, 0). Returns with ret 4. |
| `008c9350` | `SoeUtil_IStringFixedW32_ScalarDeletingDtor` | ✅ | dtor | Calls the destructor body FUN_008c7a60 on this, then frees this with _free when bit 0 of the flag byte is set. Returns this. |
| `00cf2aa0` | `IStringFixedWide32Ctor` | ✅ | ctor | Installs the IString<unsigned_short> vtable, sets the shared empty-buffer pointer, clears length/capacity fields, then installs the fixed-capacity-32 wide-string vtable. |
