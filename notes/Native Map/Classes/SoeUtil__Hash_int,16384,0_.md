# Class: SoeUtil::Hash<int,16384,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098b3b0` | `Hash16384::Method1Adjustor → Hash16384::PopMethodAdjustor` | ✅ | virtual | Adds 0x1000c to ECX and jumps to 00751950; the vtable also lists the derived HashMap<int,int,16384,0> implementation. |
| `0098b3c0` | `Hash16384::Method2Adjustor → Hash16384::PushMethodAdjustor` | ✅ | virtual | Adds 0x1000c to ECX and jumps to 007519c0; the vtable also lists the derived HashMap<int,int,16384,0> implementation. |
| `0098bdc0` | `Hash16384::Constructor` | ✅ | ctor | Constructs the 16384-bucket map state, then installs the Hash<int,16384,0> vtable and returns this. |
| `0098c020` | `Hash16384Int::ScalarDeletingDestructor` | ✅ | dtor | Calls the 16384-bucket integer-map cleanup body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
