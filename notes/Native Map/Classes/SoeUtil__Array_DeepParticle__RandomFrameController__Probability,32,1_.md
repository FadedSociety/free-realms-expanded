# Class: SoeUtil::Array<DeepParticle::RandomFrameController::Probability,32,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00883330` | `ReleaseHeapBuffer` | ✅ | virtual | Virtual slot [2]. Takes the pointer from [esp+4]; if it differs from the aligned inline address (this+0x13 rounded down to a multiple of 4, i.e. this+0x10 for 4-aligned this) calls FUN_00401c00(ptr, 4). Epilogue is ret … |
| `008837c0` | `Array::~Array` | ✅ | dtor | Fastcall (ECX=this), EH frame. Stores the Array<Probability,32,1> vtable, sets count to 0, frees the buffer at +4 with FUN_00401c00(ptr,4) unless it equals the aligned inline address (this+0x13 rounded down to a multipl… |
| `00883880` | `ReserveInlineOrForward` | ✅ | virtual | Thiscall virtual, slot [1]. Args: count [esp+4], out-capacity pointer [esp+8]. If count is at most 0x20 it writes 0x20 to *out and returns the aligned inline address (this+0x13 rounded down to a multiple of 4). Otherwis… |
| `00883fa0` | `Array::ctor` | ✅ | ctor | Fastcall (ECX=this), no arguments. Stores the Array<Probability,0,1> vtable, zeroes the three header dwords at +4, +8 and +0xc, then stores the Array<Probability,32,1> vtable. Returns void. |
| `00884020` | `Array::ScalarDeletingDtor` | ✅ | dtor | Thiscall with flag byte [esp+8]. Calls FUN_008837c0 on this (the Array<Probability,32,1> destructor body; ECX preserved), then if flag bit 0 is set calls _free(this). Returns this. Ret 4. |
| `00884040` | `SwapContentsViaTemp` | ✅ | virtual | Thiscall virtual, slot [3], stack args: peer array (param_2) and a second dword that is never read (ret 8). Stores a stack temporary with the Array<Probability,0,1> vtable, appends the peer's elements into the temporary… |
