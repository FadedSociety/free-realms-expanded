# Class: SoeUtil::HashMap<int,PromotionalBundleData,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a05c50` | `HashMap_PromotionalBundleData64_Construct` | ✅ | ctor | Constructor of the SoeUtil hash map instantiation: stores the HashMap vtable, zeroes the count at +8 and the extra word at +4, and memsets the 64 bucket pointers at +0xc (0x100 bytes). |
| `00a06220` | `HashMap_PromotionalBundleData64_NewNode` | ✅ | virtual | Node allocator of the bundle map: asks the thread or global memory manager for a 0x134-byte block (falling back to the CRT allocator with FUN_006fb950(0x134,4)) and returns it. |
| `00a06260` | `HashMap_PromotionalBundleData64_FreeNode` | ✅ | virtual | Node deallocator of the bundle map: routes the node pointer to FreeMemory with alignment 4. |
| `00a06450` | `— ⚠reject` | ⚪ | method | Drains the bundle map: for each of the 64 buckets, walks the node chain through the link at node+0x130, runs the node-data destructor FUN_01061530 on node+4 and frees the node through vtable slot 2, decrementing the cou… |
| `00a06b40` | `HashMap_PromotionalBundleData64_Destruct` | ✅ | dtor | Eleven-byte destructor body: stores the HashMap vtable and calls the drain routine 00a06450 to free every node. |
| `00a06c00` | `HashMap_PromotionalBundleData64_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the HashMap vtable, drains the map with 00a06450 and frees this when the delete flag is set. |
| `00a06f10` | `HashMap_PromotionalBundleData64_Destruct_B` | ✅ | dtor | Second emission of the bundle-map destructor: stores the HashMap vtable, keeps the object pointer in an SEH local and calls the drain routine 00a06450. |
