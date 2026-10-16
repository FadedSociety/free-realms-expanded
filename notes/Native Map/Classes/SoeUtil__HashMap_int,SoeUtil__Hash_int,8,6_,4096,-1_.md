# Class: SoeUtil::HashMap<int,SoeUtil::Hash<int,8,6>,4096,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00987af0` | `HashMap4096Hash_Constructor` | ✅ | ctor | Stores the map vtable, zeros its scalar state, clears 0x4000 bytes of buckets, and returns this. |
| `009882e0` | `HashMap4096Hash_AllocateNode` | ✅ | virtual | Uses the allocator vtable/global fallback sequence and requests 0x94 bytes aligned to 4 on the global path. |
| `00988320` | `HashMap4096Hash_FreeNode` | ✅ | virtual | Forwards its pointer with alignment 4 to 00401c00, then returns. |
| `0098cbc0` | `HashMap4096Nested::Destructor` | ✅ | dtor | Installs the nested-map vtable and clears the 4096 buckets and each contained eight-bucket map. |
| `0098cbd0` | `HashMap4096Nested::ScalarDeletingDestructor` | ✅ | dtor | Installs the nested-map vtable, clears the 4096 buckets and inner maps, frees this only when deleting flag bit 0 is set, and otherwise returns this. |
