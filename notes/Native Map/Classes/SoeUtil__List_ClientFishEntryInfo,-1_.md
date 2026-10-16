# Class: SoeUtil::List<ClientFishEntryInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b63280` | `ListAllocateNode_0x38` | ✅ | virtual | Vtable slot 2 of the client-fish-entry list: recycles a node from the caller's free list (FUN_006fb860 + slot 1) or the global list DAT_01bdf280, otherwise allocates FUN_006fb950(0x38,4). |
| `00b632c0` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the client-fish-entry list: 18 bytes, bytes 8b4424046a0450e834e989ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b692a0` | `ClearClientFishEntryInfoList` | ✅ | dtor | Installs the list vtable and repeatedly erases the current head node until no nodes remain. |
| `00b69310` | `ClearAndDeleteClientFishEntryList` | ✅ | dtor | Clears the list, returns the original pointer when flag bit zero is clear, and frees it when that bit is set. |
