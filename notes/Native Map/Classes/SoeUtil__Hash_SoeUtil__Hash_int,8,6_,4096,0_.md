# Class: SoeUtil::Hash<SoeUtil::Hash<int,8,6>,4096,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098d740` | `HashNested4096::PopMethodAdjustor` | ✅ | virtual | Adds 0x400c to ECX and jumps to the large-tree node-pool pop routine at 0098d660; the same slot is listed in the derived HashMap vtable. |
| `0098d750` | `HashNested4096::PushMethodAdjustor` | ✅ | virtual | Adds 0x400c to ECX and jumps to the large-tree node-pool push routine at 0098d6d0; the same slot is listed in the derived HashMap vtable. |
| `0098d9f0` | `HashNested4096::Constructor` | ✅ | ctor | Constructs the pooled 4096-bucket nested-map state, installs the Hash<Hash<int,8,6>,4096,0> vtable, and returns this. |
| `0098da50` | `HashNested4096::ScalarDeletingDestructor` | ✅ | dtor | Calls the pooled nested-map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
