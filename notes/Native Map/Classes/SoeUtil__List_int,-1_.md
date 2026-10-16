# Class: SoeUtil::List<int,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ebef0` | `ListAndInlineVectorDtor` | ✅ | dtor | Destroys two vector elements, restores and cleans one integer-list specialization, then cleans a second list base. |
| `009fa650` | `— ⚠reject` | ⚪ | ctor | Installs SoeUtil::List<int,-1>::vftable, clears its four node/count fields, and stores the constructor argument at field 0. |
| `009fa6c0` | `— ⚠reject` | ⚪ | dtor | Installs the integer-list vtable and invokes FUN_007798c0 twice to release list storage under an exception frame. |
