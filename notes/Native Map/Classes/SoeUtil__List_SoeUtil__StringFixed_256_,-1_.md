# Class: SoeUtil::List<SoeUtil::StringFixed<256>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009195a0` | `SoeUtil::ListStringFixed256DtorBody` | ✅ | dtor | Destructor body: stores the List<SoeUtil::StringFixed<256>,-1> vtable and walks the chain at +4 calling FUN_008fba20 on each node until the chain is empty. |
| `0091a280` | `SoeUtil::ListStringFixed256ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_008fba20 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
