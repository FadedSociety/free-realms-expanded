# Class: SoeUtil::List<UnderwaterFishSpawnInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b632f0` | `ListAllocateNode_0x68` | ✅ | virtual | Vtable slot 2 of the underwater-fish-spawn list: recycles a node from the free lists or allocates FUN_006fb950(0x68,4). |
| `00b63330` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the underwater-fish-spawn list: 18 bytes, bytes 8b4424046a0450e8c4e889ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b6a090` | `ClearUnderwaterFishSpawnInfoList` | ✅ | dtor | Installs the list vtable and repeatedly erases the current head node until the list is empty. |
| `00b6a130` | `ClearAndDeleteUnderwaterFishSpawnInfoList` | ✅ | dtor | Clears the list, returns the original pointer when flag bit zero is clear, and frees it when that bit is set. |
