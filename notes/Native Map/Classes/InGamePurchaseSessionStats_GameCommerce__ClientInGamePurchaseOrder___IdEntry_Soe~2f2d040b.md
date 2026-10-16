# Class: InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20ea0` | `IdEntryBundleIdDtor` | ✅ | dtor | Destructor body that stores the IdEntry<StoreBundleId> vtable at [param_1] and then Loggable::vftable; the two stores are the whole body and it is reached from five EH unwind funclets. |
| `00b213c0` | `IdEntryBundleIdDeletingDtor` | ✅ | dtor | Deleting destructor for IdEntry<StoreBundleId>: stores its vtable and Loggable::vftable, then frees the object through thunk 00d82463 when bit 0 of param_2 is set; returns the object pointer. |
| `00b355c0` | `StoreBundleIdSessionStat::Log` | ✅ | virtual | Formats entry timestamp then logs entry id, count and last-occurrence string. |
