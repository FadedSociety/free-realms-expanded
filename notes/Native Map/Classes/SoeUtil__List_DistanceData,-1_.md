# Class: SoeUtil::List<DistanceData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00996090` | `ListDistanceDataSlot2Dispatch ?` | 🟡 | virtual | Virtual slot [2] of SoeUtil::List<DistanceData,-1>, no stack args, returns void. Calls 006fb9c0, then 006fb860. If that returns a nonzero object, calls its vtable slot +4 and returns. Else, if DAT_01bdf280 is nonzero, c… |
| `009960d0` | `ListDistanceDataSlot3FreeArg` | 🟢 | virtual | Virtual slot [3] of SoeUtil::List<DistanceData,-1>. One stack argument (undefined4) is passed with alignment 4 to 00401c00 (free routine), then returns void. |
| `009966a0` | `ListDistanceDataDtorBody` | 🟢 | dtor | Fastcall (ECX=this). Stores the SoeUtil::List<DistanceData,-1> vtable (0180cbc8) to [this], then tail-jumps to 00996390 (list clear, ECX=this). 11 bytes: mov dword [ecx], vtable; jmp rel32. Does not free the object. |
| `009966b0` | `ListDistanceDataScalarDeletingDtor` | ✅ | dtor | Thiscall (ECX=this) with one stack arg (byte flags). Stores the List<DistanceData,-1> vtable to [this], calls 00996390 (clear, ECX=this), then if bit 0 of the flags arg is set pushes this and calls the free thunk (00d82… |
