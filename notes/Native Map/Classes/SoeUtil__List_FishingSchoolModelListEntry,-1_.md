# Class: SoeUtil::List<FishingSchoolModelListEntry,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b631f0` | `ListAllocateNode_0x14` | ✅ | virtual | Vtable slot 2 of the model-list-entry list: takes a node from the caller's container free-list through FUN_006fb860/its vtable slot 1, falls back to the global free list DAT_01bdf280, and otherwise allocates FUN_006fb95… |
| `00b63230` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the model-list-entry list: 18 bytes, bytes 8b4424046a0450e8c4e989ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4, freeing the node with 4-byte alignment. |
| `00b63d70` | `ListFishingSchoolModelListEntry_InsertTail` | 🟢 | method | Allocates a node through the list's vtable slot 2, initialises the payload to {0xffffffff, 0, float constant _DAT_0175b420}, links the node after the tail with node+0x10 (previous tail) and node+0xc, updates the old tai… |
| `00b655c0` | `ListFishingSchoolModelListEntry_Clear` | 🟢 | method | Loop that repeatedly takes the head at this+4, unlinks the node using node+0x10 and node+0xc, decrements the count at this+0xc and hands the node to the list's free slot, until the list is empty. |
| `00b66480` | `ListFishingSchoolModelListEntry::~ListFishingSchoolModelListEntry` | ✅ | dtor | 11-byte tail-call destructor: stores the list vtable (mov dword[ecx],0x018227dc) and jumps to FUN_00b655c0, which frees every node. |
| `00b66580` | `ListFishingSchoolModelListEntry::~ListFishingSchoolModelListEntry` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, calls FUN_00b655c0 to free every node, then frees the list object itself with _free when bit 0 of the flag argument is set. |
