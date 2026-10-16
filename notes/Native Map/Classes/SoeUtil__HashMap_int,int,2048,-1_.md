# Class: SoeUtil::HashMap<int,int,2048,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c31020` | `IntHashMapCtor` | ✅ | ctor | Installs the HashMap vtable, zeros count and allocator fields, clears 0x2000 bytes of buckets, and returns this. |
| `00c311c0` | `IntHashMapDtor` | ✅ | dtor | Restores the hash-map vtable, then tail-jumps to FUN_00c310e0 to dispose all entries. |
| `00c311d0` | `HashMapAllocator` | ✅ | method | Initializes allocation state, tries the thread and global memory managers, then requests 0xc bytes aligned to four. |
| `00c31210` | `— ⚠reject` | ⚪ | method | Calls FUN_00401c00(pointer,4) and returns with eight bytes callee-cleaned. |
| `00c31230` | `HashMapScalarDeletingDtor` | ✅ | dtor | Restores the map vtable, clears entries through FUN_00c310e0, and frees this object if flag bit 0 is set. |
| `00c31380` | `IntHashMapDtor` | ✅ | dtor | Restores the hash-map vtable and invokes FUN_00c310e0 to dispose its entries. |
