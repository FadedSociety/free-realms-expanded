# Class: SoeUtil::HashList<InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>_>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b31420` | `StoreOrderBundleGroupHashList::Ctor` | ✅ | ctor | Initializes list state, clears 0x40 bytes for bucket heads, and writes the specialized 16-bucket HashList vtable. |
| `00b31600` | `StoreOrderBundleGroupHashList::ScalarDeletingDtor` | ✅ | dtor | Calls list cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the pointer. |
