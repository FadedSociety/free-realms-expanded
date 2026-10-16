# Class: SoeUtil::Array<ClaimItem,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00700` | `— ⚠reject` | ⚪ | virtual | Capacity policy of a 0x168-byte-element array: grows the request by 5/4 when it exceeds the current capacity, shrinks it by 6/5 when it is below three quarters of it, writes the chosen capacity to *param_3 and returns e… |
| `00a007a0` | `Array_ClaimItem_0_FreeBackingStore` | 🟢 | virtual | Free wrapper (18 bytes: mov eax,[esp+4]; push 8; push eax; call 00401c00; add esp,8; ret 8): releases the array buffer with alignment 8. |
| `00a007c0` | `Array_ClaimItem_0_Swap` | 🟢 | virtual | Array swap slot: when the flag param_3 is non-zero it tail-jumps into virtual slot 3 (offset 0xc) of the object in param_2 passing this object as the argument and 0 as the flag (bytes: cmp byte [esp+8],0; mov eax,ecx; m… |
| `00a01a30` | `Array_ClaimItem_0_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::Array<ClaimItem,0,1>: stores the class vtable, truncates the elements, frees the backing store and then frees this when bit 0 of the flag is set. |
