# Class: SoeUtil::Array<int,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00996110` | `ArrayIntSlot2FreeOutOfLine` | 🟢 | virtual | Thiscall (ECX=this) with one stack arg param_2, virtual slot [2] of SoeUtil::Array<int,10,1>. If param_2 differs from (this+0x13) rounded up to a multiple of 4, calls 00401c00(param_2,4) (free with alignment 4). Returns… |
| `009964b0` | `ArrayInt10DtorBody` | 🟢 | dtor | Fastcall (ECX=this; data pointer at [+4], count at [+8]). Sets [this] to the SoeUtil::Array<int,10,1> vtable (0180cbf0), zeroes [+8], frees the buffer at [+4] with alignment 4 through 00401c00 when it is not the inline … |
| `00996540` | `ArrayIntSlot1InlineOrTailGrow` | 🟢 | virtual | Virtual slot [1] of SoeUtil::Array<int,10,1>, thiscall with two stack args (count param_2, out-pointer param_3), ret 0xc. Bytes: if param_2 > 10 it tail-jumps to 0073a540 with the same stack args (its result is returned… |
| `00996600` | `ArrayIntCtorZeroFields` | 🟢 | ctor | Fastcall (ECX=this, no stack args). Stores the SoeUtil::Array<int,0,1> vtable (017e99b0) to [this], zeroes [+4], [+8] and [+0xc], then stores the SoeUtil::Array<int,10,1> vtable (0180cbf0) to [this] and returns. |
| `00996680` | `ArrayInt10ScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags). Bytes: calls 009964b0 (dtor body, ECX=this); if bit 0 of the flags arg is set, pushes this and calls the free thunk (00d82463 to _free), then returns this with ret 4. |
| `009968a0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] of SoeUtil::Array<int,10,1>, thiscall (ECX=this) with one stack arg param_2. Calls 0073c310(param_2), then 0073c370 with ECX=this, then 0073c370 on a local object (its address passed). Sets the local's … |
