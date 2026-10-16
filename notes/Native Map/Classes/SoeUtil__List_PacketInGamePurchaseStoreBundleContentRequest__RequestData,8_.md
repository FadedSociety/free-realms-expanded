# Class: SoeUtil::List<PacketInGamePurchaseStoreBundleContentRequest::RequestData,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b42140` | `ListRequestDataCtor → SoeUtil::List<PacketInGamePurchaseStoreBundleContentReque…` | ✅ | ctor | Initializes the list base fields at +4/+8/+0xc to zero, switches the list vtable from the -1 base to the capacity-8 specialization, and initializes its eight inline hash buckets through 00b24800. |
| `00b42b70` | `ListPool68AllocAdjustor` | ✅ | virtual | Eight-byte x86 thunk bytes 83 c1 10 e9 18 ff ff ff add ECX,0x10 and jump to 00b42a90; it adjusts this to the embedded pool then transfers control to the 0x68-byte block allocator. |
| `00b42b80` | `ListPool68FreeAdjustor` | ✅ | virtual | Eight-byte x86 thunk bytes 83 c1 10 e9 78 ff ff ff add ECX,0x10 and jump to 00b42b00; it adjusts this to the embedded pool then transfers control to the 0x68-byte block release. |
| `00b42b90` | `ListRequestData_DestructWithPool` | ✅ | dtor | Writes the capacity-8 vtable, drains the node list through 00b3a8a0, recursively frees both branches of the allocation tree with 00b28e80 using alignment 4, clears tree fields +0x18/+0x1c, then writes the -1 base vtable… |
| `00b42c60` | `ListRequestData_ScalarDeletingDtor` | ✅ | dtor | The 27-byte x86 wrapper saves this, calls destructor body 00b42b90, tests bit 0 of the flag byte, conditionally calls _free(this), restores the saved register and returns this when not deleted. |
