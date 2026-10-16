# Class: SoeUtil::Array<unsigned_char,128,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008831e0` | `ReleaseHeapBuffer` | ✅ | virtual | Thiscall virtual, slot [2]. Takes the buffer pointer from [esp+4]; if it differs from this+0x10 (the inline storage) calls FUN_00401c00(ptr, 1) to free it. Epilogue is ret 8. |
| `00883640` | `Array::~Array` | ✅ | dtor | Fastcall (ECX=this), EH frame. Stores the SoeUtil::Array<unsigned_char,128,1> vtable, sets count (+8) to 0, frees the buffer at +4 with FUN_00401c00(ptr,1) unless it equals this+0x10 (inline), sets buffer to 0, then sto… |
| `008836d0` | `ReserveInlineOrForward` | ✅ | virtual | Thiscall virtual, slot [1]. Args: size [esp+4], out-capacity pointer [esp+8]. If size is at most 0x80 it writes 0x80 to *out and returns this+0x10 (inline storage). Otherwise it tail-jumps to FUN_00702f80 with the same … |
| `00883f40` | `Array::ctor` | ✅ | ctor | Fastcall (ECX=this), no arguments. Stores the Array<unsigned char,0,1> vtable, zeroes the three header dwords at +4, +8 and +0xc, then stores the Array<unsigned char,128,1> vtable. Returns void. |
| `00884000` | `Array::ScalarDeletingDtor` | ✅ | dtor | Thiscall with flag byte [esp+8]. Calls FUN_00883640 on this (the Array<unsigned char,128,1> destructor body; ECX preserved), then if flag bit 0 is set calls _free(this). Returns this. Ret 4. |
| `00884880` | `SwapContentsViaTemp` | 🟢 | virtual | Thiscall virtual, slot [3], stack arg peer (param_2). Builds a stack temporary through FUN_00703790 (ECX = temporary, argument = peer), then appends this array's bytes to the peer (reserving through FUN_00701cc0 when ne… |
