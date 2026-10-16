# Class: SoeUtil::Array<int,6,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbbe0` | `FreeHeapStorage → SoeUtil::Array<int,6,1>::FreeHeapStorage` | ✅ | virtual | Frees param_2 with alignment 4 only when it differs from the inline address obtained by rounding this+0x13 down to a four-byte boundary. |
| `008c6e60` | `SoeUtil_ArrayInt6::Dtor` | ✅ | dtor | Stores Array<int,6,1> vftable at this, zeroes count at +8, and frees buffer +4 with alignment 4 only when it is not inline ((this+0x13) rounded down to 4). Clears +4, stores Array<int,0,1> vftable, zeroes count again, t… |
| `008c6ef0` | `SoeUtil_ArrayInt6::VSlot1_InlineOrDelegate` | ✅ | virtual | Bytes: cmp eax,6; jle inline; otherwise tail-jumps to FUN_0073a540 with the same stack arguments. Inline path: writes 6 to *param_3 and returns (this+0x13) rounded down to a multiple of 4 (the inline element buffer). Re… |
| `008d38a0` | `ArrayInt6::ArrayInt6` | ✅ | ctor | Initializes the array object with the empty Array<int,0,1> base vtable, zeroes its data pointer, count, and capacity fields, then installs the Array<int,6,1> vtable. |
| `008d3a80` | `ArrayInt6_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the fixed-capacity int-array vtable): runs FUN_008c6e60 (the array's non-deleting record teardown) and frees the object through the CRT _free thunk when bit 0 of the flag arg… |
| `008e0540` | `ArrayInt6_SwapContents` | ✅ | virtual | Copy-constructs a temporary from the argument array, assigns the argument from this array, assigns this array from the temporary, then frees temporary storage with four-byte alignment. |
