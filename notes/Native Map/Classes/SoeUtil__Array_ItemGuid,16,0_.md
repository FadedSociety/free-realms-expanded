# Class: SoeUtil::Array<ItemGuid,16,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc2f0` | `FreeHeapStorage` | ✅ | virtual | Frees param_2 with alignment 4 only when it differs from the inline address obtained by rounding this+0x13 down to a four-byte boundary. |
| `008c70f0` | `SoeUtil_ArrayItemGuid16_Dtor` | ✅ | dtor | Stores Array<ItemGuid,16,0> vftable at this, zeroes count at +8, and frees buffer +4 with alignment 4 only when it is not inline ((this+0x13) rounded down to 4). Clears +4, stores Array<ItemGuid,0,1> vftable, zeroes cou… |
| `008c71b0` | `SoeUtil_ArrayItemGuid16::VSlot1_InlineBuffer` | ✅ | virtual | Bytes: mov eax,[esp+8]; mov dword [eax],0x10; lea eax,[ecx+0x13]; and eax,-4; ret 0xc. Writes capacity 16 to *param_3 and returns the inline buffer at this+0x13 rounded to 4. |
| `008d3900` | `ArrayItemGuid16::ArrayItemGuid16` | ✅ | ctor | Initializes the array with the empty Array<ItemGuid,0,1> vtable, zeroes its data pointer, count, and capacity, then installs the Array<ItemGuid,16,0> vtable. |
| `008d3aa0` | `ArrayItemGuid16_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the 16-element ItemGuid-array vtable): runs FUN_008c70f0 (the array's non-deleting record teardown) and frees the object through the CRT _free thunk when bit 0 of the flag ar… |
| `008e06d0` | `ArrayItemGuid16_SwapContents` | ✅ | virtual | Copy-constructs a temporary from the argument array, assigns the argument from this array, assigns this array from the temporary, then frees temporary storage with four-byte alignment. |
