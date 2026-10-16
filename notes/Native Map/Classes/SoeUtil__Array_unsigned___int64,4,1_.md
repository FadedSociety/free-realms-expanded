# Class: SoeUtil::Array<unsigned___int64,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008c77e0` | `SoeUtil_ArrayUInt64x4_Dtor` | ✅ | dtor | Stores Array<unsigned__int64,4,1> vftable at this, zeroes count at +8, and frees buffer +4 with alignment 8 only when it is not inline ((this+0x17) rounded down to 8). Clears +4, stores Array<unsigned__int64,0,1> vftabl… |
| `008c7900` | `SoeUtil_ArrayUInt64x4::VSlot1_InlineOrDelegate` | ✅ | virtual | Bytes: cmp eax,4; jle inline; otherwise tail-jumps to FUN_008c76d0 with the same stack arguments. Inline path: writes 4 to *param_3 and returns (this+0x17) rounded down to a multiple of 8 (the inline element buffer). Re… |
| `008e0d40` | `ArrayUInt64_4_SwapContents` | ✅ | virtual | Copy-constructs a temporary from the argument array, assigns the argument from this array, assigns this array from the temporary, then frees temporary storage with eight-byte alignment. |
