# Class: SoeUtil::HashList<InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<int>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b31510` | `PurchaseOrderIntHashList16::Ctor` | ✅ | ctor | Installs map state, clears 0x40 bytes of bucket heads, and writes the specialized 16-bucket HashList vtable. |
| `00b31640` | `PurchaseOrderIntHashList16::ScalarDeletingDtor` | ✅ | dtor | Calls list cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the pointer. |
