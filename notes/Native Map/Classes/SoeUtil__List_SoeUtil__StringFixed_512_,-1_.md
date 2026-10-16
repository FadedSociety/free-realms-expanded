# Class: SoeUtil::List<SoeUtil::StringFixed<512>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009195d0` | `SoeUtil::ListStringFixed512DtorBody` | ✅ | dtor | Destructor body: stores the List<SoeUtil::StringFixed<512>,-1> vtable and walks the chain at +4 calling FUN_008fbac0 on each node until the chain is empty. |
| `0091a2c0` | `SoeUtil::ListStringFixed512ScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_008fbac0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
