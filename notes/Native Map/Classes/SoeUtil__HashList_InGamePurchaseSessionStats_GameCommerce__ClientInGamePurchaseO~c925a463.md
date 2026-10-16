# Class: SoeUtil::HashList<InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>,6,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b314a0` | `PurchaseOrderIntHashList6::Ctor` | ✅ | ctor | Zeros list pointers, count, and six bucket-head fields, then writes the specialized six-bucket HashList vtable. |
| `00b31620` | `PurchaseOrderIntHashList6::ScalarDeletingDtor` | ✅ | dtor | Calls list cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the pointer. |
