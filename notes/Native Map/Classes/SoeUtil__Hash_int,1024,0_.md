# Class: SoeUtil::Hash<int,1024,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098b480` | `Hash1024::Method1Adjustor → Hash1024::PopMethodAdjustor` | ✅ | virtual | Adds 0x100c to ECX and jumps to 00751950; the vtable also lists the derived HashMap<int,int,1024,0> implementation. |
| `0098b490` | `Hash1024::Method2Adjustor → Hash1024::PushMethodAdjustor` | ✅ | virtual | Adds 0x100c to ECX and jumps to 007519c0; the vtable also lists the derived HashMap<int,int,1024,0> implementation. |
| `0098be20` | `Hash1024::Constructor` | ✅ | ctor | Constructs the 1024-bucket map state, then installs the Hash<int,1024,0> vtable and returns this. |
| `0098c080` | `Hash1024Int::ScalarDeletingDestructor` | ✅ | dtor | Calls the 1024-bucket integer-map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
