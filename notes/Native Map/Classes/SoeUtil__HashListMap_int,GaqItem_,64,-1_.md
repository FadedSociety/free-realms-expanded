# Class: SoeUtil::HashListMap<int,GaqItem*,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a091e0` | `HashListMap_GaqItem64_Construct` | ✅ | ctor | Constructor of the GAQ item map: stores the HashListMap vtable, clears head/tail/count and the extra word and memsets the 64 bucket pointers at +0x14 (0x100 bytes). |
| `00a098c0` | `HashListMap_GaqItem64_NewNode` | ✅ | virtual | Node allocator of the GAQ item map: requests a 0x14-byte node from the thread/global memory manager, falling back to FUN_006fb950(0x14,4). |
| `00a09900` | `HashListMap_GaqItem64_FreeNode` | ✅ | virtual | Node deallocator of the GAQ item map: routes the node pointer to FreeMemory with alignment 4. |
| `00a0a160` | `HashListMap_GaqItem64_Destruct` | ✅ | dtor | Destructor body of the GAQ item map: stores the HashListMap vtable and repeatedly removes the head node with 00a09840 until the list is empty. |
| `00a0a190` | `HashListMap_GaqItem64_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: drains the GAQ item map through 00a09840 and frees this when the delete flag is set. |
| `00a0a1f0` | `HashListMap_GaqItem64_Destruct_B` | ✅ | dtor | Second emission of the GAQ map destructor: stores the HashListMap vtable and drains the map through 00a09840 under an SEH frame. |
| `00a0a330` | `HashListMap_GaqItem64_Construct_B` | ✅ | ctor | Full constructor of the GAQ item map: stores the HashListMap vtable, clears head/tail/count and the 64 bucket pointers (0x100 bytes at +0x14), then installs SoeUtil::HashList<GaqItem*,64,-1>::vftable as the base constru… |
