# Class: InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleGroupId,1>>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b221b0` | `InGamePurchaseSessionStats_IdEntry_StoreBundleGroupIdDtor` | ✅ | dtor | Destructor body: stores the IdEntry vtable first, then the Loggable base vtable, into *this, and returns. |
| `00b25ef0` | `— ⚠reject` | ⚪ | ctor | Constructor: writes Loggable::vftable then the IdEntry<StoreBundleGroupId> vtable, copies the typed key from *param_2 into this+4, zeroes this+8, seeds this+0x10/0x14 from two globals and overwrites them from FUN_007026… |
| `00b261f0` | `InGamePurchaseSessionStats_IdEntry_StoreBundleGroupIdDeletingDtor` | ✅ | dtor | Scalar deleting destructor: writes the IdEntry<StoreBundleGroupId> vtable then Loggable::vftable and frees this via _free when the flags low bit is set. |
| `00b27bc0` | `InGamePurchaseSessionStats::IdEntryStoreBundleGroupIdDefaultCtor` | ✅ | ctor | Default constructor: writes Loggable::vftable then the IdEntry<StoreBundleGroupId> vtable and zeroes this+4 and this+8. |
| `00b28fb0` | `IdEntryStoreBundleGroupIdInitSetField20` | 🟢 | ctor | Inside an SEH frame, calls FUN_00b25ef0 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b29300` | `HashListRemoveIdEntryStoreBundleGroupIdNode` | 🟢 | function | Same removal as 00b29240 but with bucket = param_2[8] & 0xf and the payload destroyed by writing the IdEntry<StoreBundleGroupId> vtable then Loggable::vftable before the node is freed through virtual slot 2. |
