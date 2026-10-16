# Class: SoeUtil::List<CustomTintEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007991b0` | `ListAllocBufferCustomTint` | 🟢 | virtual | Vtable slot [2] of SoeUtil::List<CustomTintEntry,-1>, no stack arguments. Same routing as 00799120 (006fb9c0, 006fb860, the manager's slot [1], global manager at DAT_01bdf280, fallback 006fb950) but with size 0x1c and a… |
| `007991f0` | `ListFreeBuffer` | 🟢 | virtual | Vtable slot [3] of SoeUtil::List<CustomTintEntry,-1>, one stack argument (ret 4). Calls 00401c00(argument, 4) to free the buffer with 4-byte alignment. |
| `007998b0` | `ListCustomTintEntryDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores SoeUtil::List<CustomTintEntry,-1>::vftable at +0, then repeatedly unlinks and frees the head node through 00799670 until the head is NULL. |
