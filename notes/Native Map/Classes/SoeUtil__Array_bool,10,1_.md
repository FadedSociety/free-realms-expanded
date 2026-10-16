# Class: SoeUtil::Array<bool,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009960f0` | `ArrayBoolSlot2FreeOutOfLine` | 🟢 | virtual | Thiscall (ECX=this) with one stack arg param_2, virtual slot [2] of SoeUtil::Array<bool,10,1>. If param_2 is not this+0x10, calls 00401c00(param_2,1) (free with alignment 1). Returns void. |
| `009963f0` | `ArrayBool10DtorBody` | 🟢 | dtor | Fastcall (ECX=this, data pointer at [+4], count at [+8]). Sets [this] to the SoeUtil::Array<bool,10,1> vtable (0180cbdc), zeroes [+8], frees the buffer at [+4] with alignment 1 through 00401c00 when it is not the inline… |
| `00996480` | `ArrayBoolSlot1InlineOrTailGrow` | 🟢 | virtual | Virtual slot [1] of SoeUtil::Array<bool,10,1>, thiscall with two stack args (count param_2, out-pointer param_3), ret 0xc. Bytes: if param_2 > 10 it tail-jumps to 007e84f0 with the same stack args (its result is returne… |
| `009965a0` | `ArrayBoolCtorZeroFields` | 🟢 | ctor | Fastcall (ECX=this, no stack args). Stores the SoeUtil::Array<bool,0,1> vtable (017f1184) to [this], zeroes [+4], [+8] and [+0xc], then stores the SoeUtil::Array<bool,10,1> vtable (0180cbdc) to [this] and returns. |
| `00996660` | `ArrayBool10ScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags). Bytes: calls 009963f0 (dtor body, ECX=this); if bit 0 of the flags arg is set, pushes this and calls the free thunk (00d82463 to _free), then returns this with ret 4. |
| `009967a0` | `ArrayBoolSlot3SwapContents` | 🟢 | virtual | Virtual slot [3] of SoeUtil::Array<bool,10,1>, thiscall (ECX=this) with one stack arg (the other array, param_2). Builds a temporary copy of the argument with 009966e0 (ECX=temporary, argument pushed). Resets the argume… |
