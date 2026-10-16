# Class: SoeUtil::List<ProxiedFishingUnderwaterFish,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b63530` | `ListAllocateNode_0x1A0` | ✅ | virtual | Vtable slot 2 of the proxied-underwater-fish list: recycles a node from the free lists or allocates FUN_006fb950(0x1a0,0x10) - 416 bytes with 16-byte alignment, matching the node whose links are at +0x190/+0x194. |
| `00b63570` | `ListFreeNode_0x10` | ✅ | virtual | Vtable slot 3 of the proxied-underwater-fish list: 18 bytes, bytes 8b4424046a1050e884e689ff83c408c20400 = mov eax,[esp+4]; push 0x10; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b637d0` | `ListProxiedFishingUnderwaterFish_InsertTail` | 🟢 | method | Allocates a node through the list's vtable slot 2, constructs the element in place with FUN_00b61e70 (when allocation succeeded), then links the node after the tail using node+0x194 (previous tail) and node+0x190, updat… |
| `00b64a90` | `ListProxiedFishingUnderwaterFish_EraseNode` | 🟢 | method | Unlinks one node from the list using node+0x194/node+0x190, fixes the neighbours and this+4/this+8, decrements the count at this+0xc, calls FUN_00cd1950 to destroy the ProxiedFishingUnderwaterFish element, and hands the… |
| `00b68740` | `ClearProxiedFishingUnderwaterFishList` | ✅ | dtor | Installs the list vtable and repeatedly erases the current head node until the list is empty. |
| `00b68770` | `ClearAndDeleteProxiedFishList` | ✅ | dtor | Clears the list, returns the object when flag bit zero is clear, and frees the object when that bit is set. |
