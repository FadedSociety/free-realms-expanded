# Class: SoeUtil::List<ProxiedFishingSchool,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b634c0` | `ListAllocateNode_0x48` | ✅ | virtual | Vtable slot 2 of the proxied-fishing-school list: recycles a node from the free lists or allocates FUN_006fb950(0x48,8) - 72 bytes with 8-byte alignment, matching the node whose 64-byte payload is followed by links at +… |
| `00b63500` | `ListFreeNode_8` | ✅ | virtual | Vtable slot 3 of the proxied-fishing-school list: 18 bytes, bytes 8b4424046a0850e8f4e689ff83c408c20400 = mov eax,[esp+4]; push 8; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b63870` | `ListProxiedFishingSchool_InsertTail` | 🟢 | method | Allocates a node through the list's vtable slot 2, constructs the element in place with FUN_00b61ed0 from the two arguments, links the node after the tail with node+0x44 (previous tail) and node+0x40, updates the old ta… |
| `00b647e0` | `ListProxiedFishingSchool_EraseNode` | 🟢 | method | Unlinks one node from the list using node+0x44/node+0x40, fixes the neighbours and this+4/this+8, decrements the count at this+0xc, calls FUN_00cd1550 to destroy the ProxiedFishingSchool element, and hands the node to t… |
| `00b654c0` | `ListProxiedFishingSchool_Clear` | 🟢 | method | 27-byte loop: bytes 568bf18b460485c0740f508bcee80ef3ffff8b460485c075f15ec3 = while the head at this+4 is non-zero, call FUN_00b647e0 on it (erase and destroy), rereading the head each iteration. |
| `00b664c0` | `ListProxiedFishingSchool::~ListProxiedFishingSchool` | ✅ | dtor | Non-deleting destructor: stores the list vtable and erases every node by calling FUN_00b647e0 on the head at param_1[1] until the list is empty. |
| `00b665f0` | `ListProxiedFishingSchool::~ListProxiedFishingSchool` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, erases every node through FUN_00b647e0, then frees the list object with _free when bit 0 of the flag argument is set. |
