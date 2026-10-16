# Class: SoeUtil::HashListMap<int,ItemInstance,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf5ee0` | `HashListMapItemInstance20Ctor` | ✅ | ctor | Installs the HashListMap vtable, clears its metadata fields, zeroes the 20-entry bucket array at +0x14, and returns the object pointer. |
| `00cf6360` | `HashListMapItemInstance20AllocateNode` | ✅ | virtual | Calls allocator setup 006fb9c0, obtains an allocator through 006fb860 and calls its allocation slot; if absent, falls back to DAT_01bdf280 and finally 006fb950 with size 0x40 and alignment 8. |
| `00cf63a0` | `HashListMapItemInstance20FreeNode ? ⚠rename` | 🟡 | virtual | Passes the supplied node to 00401c00 with alignment 8. |
| `00cf63c0` | `— ⚠reject` | ⚪ | dtor | Calls 0103dd90 to destroy the map body, frees the object when the low bit of the flag is set, and otherwise returns this. |
| `00cf6d90` | `HashListMapItemInstanceDtor ?` | 🟡 | dtor | Restores the HashListMap vtable and repeatedly destroys its node at offset 8 through FUN_00cf66d0. |
| `00cf6dc0` | `HashListMapItemInstanceScalarDeletingDtor ?` | 🟡 | dtor | Restores the map vtable, destroys linked nodes, and frees the object when the delete flag's low bit is set. |
| `00cf6e60` | `HashListMapItemInstanceCleanup ?` | 🟡 | dtor | Restores the map vtable and repeatedly destroys nodes while the linked-list head at offset 8 is nonzero. |
| `00cf6fb0` | `HashListMapItemInstanceInit ?` | 🟡 | init | Installs the map vtable, zeroes its size and list pointers, clears 20 bucket pointers, then installs the HashList vtable. |
