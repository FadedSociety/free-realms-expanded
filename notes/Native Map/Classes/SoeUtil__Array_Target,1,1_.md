# Class: SoeUtil::Array<Target,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bea50` | `InlineTargetArray::FreeBuffer` | ✅ | virtual | Preserves inline object storage at this+0x13 aligned to 4; frees non-inline buffers with alignment 4. |
| `008c7fe0` | `SoeUtil_ArrayTarget1_Dtor` | ✅ | dtor | Stores Array<Target,1,1> vftable and calls FUN_008bedb0 with this as receiver; that helper visits counted elements at buffer +4 (stride 8), invokes each element vtable entry 0 with one zero argument, and resets count +8… |
| `008c80b0` | `SoeUtil_ArrayTarget1::VSlot1_InlineOrDelegate` | ✅ | virtual | Bytes: cmp eax,1; jle inline; otherwise tail-jumps to FUN_008bdbd0 with the same stack arguments. Inline path writes 1 to *param_3 and returns (this+0x13) rounded down to a multiple of 4. Returns with ret 0xc. |
| `008d3a20` | `ArrayTarget1::ArrayTarget1` | ✅ | ctor | Initializes the array with the empty Array<Target,0,1> vtable, zeroes its data pointer, count, and capacity, then installs the Array<Target,1,1> vtable. |
| `008d3b60` | `ArrayTarget1_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the one-element Target-array vtable): runs FUN_008c7fe0 (the array's non-deleting record teardown) and frees the object through the CRT _free thunk when bit 0 of the flag arg… |
| `008e1dd0` | `ArrayTarget1_SwapContents` | ✅ | virtual | Copy-constructs a temporary from the argument, assigns the argument from this Target array, assigns this array from the temporary, then frees temporary state. |
