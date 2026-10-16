# Class: SoeUtil::Hash<int,8,6>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098b550` | `Hash8::PopMethodAdjustor` | ✅ | virtual | Adds 0x2c to ECX and jumps to 0098ace0, the pooled-node pop routine; the same slot appears in the derived HashMap<int,int,8,6> vtable. |
| `0098b560` | `Hash8::PushMethodAdjustor` | ✅ | virtual | Adds 0x2c to ECX and jumps to 0098ad50, the pooled-node push routine; the same slot appears in the derived HashMap<int,int,8,6> vtable. |
| `0098ba40` | `Hash8::Constructor` | ✅ | ctor | Constructs the associated eight-bucket map state, then installs the Hash<int,8,6> vtable and returns this. |
| `0098baa0` | `Hash8::ScalarDeletingDestructor` | ✅ | dtor | Calls the eight-bucket map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
