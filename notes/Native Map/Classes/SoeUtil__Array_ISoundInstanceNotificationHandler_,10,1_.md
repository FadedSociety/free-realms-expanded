# Class: SoeUtil::Array<ISoundInstanceNotificationHandler*,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007be500` | `SmallArrayFreeIfHeap` | ✅ | virtual | Virtual slot [2] (thiscall, one stack arg, ret 8): if the pointer argument differs from (this+0x13) & ~3 it frees it through 00401c00(ptr, 4); otherwise it does nothing. |
| `007bf230` | `SmallArrayDtorBody` | ✅ | dtor | Fastcall (ECX = array), no stack args: sets the vtable to the 10-element array, zeroes the count, frees the buffer at +4 through 00401c00(ptr, 4) only when it is not the inline address (this+0x13)&~3, zeroes +4; then se… |
| `007bf2f0` | `SmallArrayPlanCapacity` | ✅ | virtual | Virtual slot [1] (thiscall, three stack args, ret 0xc): if the requested count is 10 or less, writes 10 to *out and returns the inline storage address (this+0x13)&~3; otherwise tail-jumps to the generic planner 007be3d0… |
| `007bfac0` | `SmallArrayCtor` | ✅ | ctor | Fastcall (ECX = array, no stack args, ret 0): installs the base SoeUtil::Array<ISoundInstanceNotificationHandler*,0,1> vtable, zeroes the three state dwords at +4, +8 and +0xc, then installs the SoeUtil::Array<ISoundIns… |
| `007bfb20` | `SmallArrayScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] (thiscall, one byte flag arg, ret 4): calls the destructor body 007bf230, then calls _free on the object when flag bit 0 is set; returns the object pointer. |
| `007bfe10` | `— ⚠reject` | ⚪ | virtual | Virtual slot [3] (thiscall, ret 8 by the bytes): reads the other array's count (+8) and buffer (+4) and appends them to this array through 007bf590, zeroes the other count, appends this array's buffer and count to itsel… |
