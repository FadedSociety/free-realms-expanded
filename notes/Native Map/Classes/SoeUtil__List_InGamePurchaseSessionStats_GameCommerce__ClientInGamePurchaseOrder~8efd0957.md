# Class: SoeUtil::List<InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::TransactionTracker,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b3f780` | `— ⚠reject` | ⚪ | virtual | Eight-byte virtual thunk (bytes 83 c1 10 e9 18 ff ff ff = add ecx,0x10; jmp 00b3f6a0): slot 2 of the list vtable adjusts this to the embedded node pool at +0x10 and tail-jumps to the pool block allocator. |
| `00b3f790` | `ListPoolFreeThunk` | ✅ | virtual | Eight-byte virtual thunk (bytes 83 c1 10 e9 78 ff ff ff = add ecx,0x10; jmp 00b3f710): slot 3 of the list vtable adjusts this to the embedded node pool at +0x10 and tail-jumps to the pool block release. |
| `00b40250` | `ListTransactionTracker_Destruct` | ✅ | dtor | Destructor body of the transaction-tracker list: writes the ,1 vtable at +0, drains the element list at +8 (FUN_00b366a0 on each head until empty), tears down the span tree at +0x10 (recursive frees of the +8/+0xc child… |
| `00b40320` | `ListTransactionTracker_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the destructor body 00b40250 and, when bit 0 of the flag byte argument is set, releases this with _free; otherwise returns this unchanged. |
