# Class: SoeUtil::HashListMap<int,InGamePurchaseGiftRecord,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c33c90` | `InGamePurchaseGiftHashMapCtor` | ✅ | ctor | Installs the hash-list-map vtable, zeros count/pointer fields, clears 0x80 bucket bytes, and returns this. |
| `00c33f20` | `GiftHashMapAllocator` | ✅ | method | Initializes allocator state, tries current then global managers, and falls back to allocating 0x58 bytes aligned to eight. |
| `00c33f60` | `GiftHashMapFreeBuffer` | ✅ | method | Calls FUN_00401c00 on the supplied buffer using alignment 8. |
| `00c34250` | `HashListMapGiftRecord32Dtor` | ✅ | dtor | Stores the HashListMap vtable and repeatedly removes the node at offset 8 through 00c34110 until the container is empty. |
| `00c34280` | `HashListMapGiftRecord32ScalarDtor` | ✅ | dtor | Clears nodes, returns the object when the low flag bit is clear, and otherwise frees the object allocation. |
