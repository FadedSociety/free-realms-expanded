# Class: SoeUtil::List<ProxiedFishingPlayer,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b63450` | `ListAllocateNode_0x150` | ✅ | virtual | Vtable slot 2 of the proxied-fishing-player list: recycles a node from the free lists or allocates FUN_006fb950(0x150,0x10) - 336 bytes with 16-byte alignment, matching the node whose links sit at +0x140/+0x144. |
| `00b63490` | `ListFreeNode_0x10` | ✅ | virtual | Vtable slot 3 of the proxied-fishing-player list: 18 bytes, bytes 8b4424046a1050e864e789ff83c408c20400 = mov eax,[esp+4]; push 0x10; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b63910` | `ListProxiedFishingPlayer_InsertTail` | 🟢 | method | Allocates a node through the list's vtable slot 2, constructs the element in place with FUN_00b61f30, links the node after the tail with node+0x144 (previous tail) and node+0x140, updates the old tail's +0x140, and incr… |
| `00b64740` | `ListProxiedFishingPlayer_EraseNode` | 🟢 | method | Unlinks one node from the list (node+0x144 prev, node+0x140 next, fixing the neighbours and this+4/this+8, decrementing the count at this+0xc), calls FUN_00ccf9c0 to destroy the ProxiedFishingPlayer element, and finally… |
| `00b654a0` | `ListProxiedFishingPlayer_Clear` | 🟢 | method | 27-byte loop: bytes 568bf18b460485c0740f508bcee88ef2ffff8b460485c075f15ec3 = while the head at this+4 is non-zero, call FUN_00b64740 on it (erase and destroy), rereading the head each iteration. |
| `00b66490` | `ListProxiedFishingPlayer::~ListProxiedFishingPlayer` | ✅ | dtor | Non-deleting destructor: stores the list vtable and erases every node by calling FUN_00b64740 on the head at param_1[1] until the list is empty. |
| `00b665b0` | `ListProxiedFishingPlayer::~ListProxiedFishingPlayer` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, erases every node through FUN_00b64740, then frees the list object with _free when bit 0 of the flag argument is set. |
