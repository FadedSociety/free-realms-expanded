# Class: SoeUtil::HashListMap<int,PromotionalBundleDatasource*,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a05cd0` | `HashListMap_PromotionalBundleDatasource256_Construct` | ✅ | ctor | Constructor of the SoeUtil hash-list-map instantiation: stores the HashListMap vtable, zeroes head/tail/count at +8/+0xc/+0x10 and the extra word at +4, then memsets the 256 bucket pointers at +0x14 (0x400 bytes). |
| `00a06330` | `HashListMap_PromotionalBundleDatasource256_NewNode` | ✅ | virtual | Node allocator of the datasource map: requests a 0x14-byte node from the thread/global memory manager, falling back to FUN_006fb950(0x14,4). |
| `00a06370` | `HashListMap_PromotionalBundleDatasource256_FreeNode` | ✅ | virtual | Node deallocator of the datasource map: routes the node pointer to FreeMemory with alignment 4. |
| `00a070b0` | `HashListMap_PromotionalBundleDatasource256_Destruct` | ✅ | dtor | Destructor body of the datasource map: stores the HashListMap vtable and repeatedly removes the head node with 00a06290 until the list is empty. |
| `00a070e0` | `HashListMap_PromotionalBundleDatasource256_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: drains the datasource map through 00a06290 and frees this when the delete flag is set. |
| `00a07490` | `HashListMap_PromotionalBundleDatasource256_Destruct_B` | ✅ | dtor | Second emission of the datasource-map destructor: stores the HashListMap vtable and drains the map through 00a06290 under an SEH frame. |
| `00a07660` | `HashListMap_PromotionalBundleDatasource256_Construct_B` | ✅ | ctor | Full constructor of the datasource map: stores the HashListMap vtable, clears head/tail/count and the 256 bucket pointers (0x400 bytes at +0x14), then reinstalls SoeUtil::HashList<PromotionalBundleDatasource*,256,-1>::v… |
