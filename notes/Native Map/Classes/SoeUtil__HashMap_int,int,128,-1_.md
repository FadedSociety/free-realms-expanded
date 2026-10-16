# Class: SoeUtil::HashMap<int,int,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d00120` | `InitializeHashMap128` | ✅ | ctor | Installs the hash-map vtable, clears the count, zeroes all 128 bucket heads, and resets the node-storage pointer. |
| `00d005d0` | `HashMapCleanup` | ✅ | dtor | Restores the hash-map vtable and clears all remaining bucket chains through the shared map cleanup routine. |
| `00d005e0` | `HashMapAllocateNode` | ✅ | virtual | Routes a 12-byte node allocation at alignment four through the current allocator, then fallback allocator, then CRT heap helper. |
| `00d00620` | `HashMapFreeNode` | ✅ | virtual | Frees the supplied hash-map node pointer through the shared allocator at alignment four. |
| `00d007f0` | `HashMapScalarDeletingDtor` | ✅ | dtor | Restores the hash-map vtable, clears its buckets, and frees the containing object only when deleting-flag bit 0 is set. |
| `00d01850` | `HashMapIntInt128Destructor → SoeUtil::HashMap<int,int,128,128>::~HashMap` | ✅ | dtor | Installs the 128-bucket derived vtable, runs its destructor body, frees the derived table's tree storage, clears its fields, installs the base -1-bucket vtable, and runs the base destructor body. |
