# Class: InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>::IdEntry<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b22160` | `InGamePurchaseSessionStats_IdEntry_StoreBundleIdDtor` | ✅ | dtor | Destructor body: stores the IdEntry vtable first, then the Loggable base vtable, into *this (own-then-base order identifies a destructor), and returns. |
| `00b25e60` | `— ⚠reject` | ⚪ | ctor | Constructor: writes Loggable::vftable then the IdEntry<StoreBundleId> vtable, copies the typed key from *param_2 into this+4, zeroes this+8, seeds this+0x10/0x14 from two globals and then overwrites them with the 8-byte… |
| `00b26180` | `InGamePurchaseSessionStats_IdEntry_StoreBundleIdDeletingDtor` | ✅ | dtor | Scalar deleting destructor: writes the IdEntry<StoreBundleId> vtable then Loggable::vftable to *this and frees this via _free when the flags low bit is set. |
| `00b27b60` | `InGamePurchaseSessionStats::IdEntryStoreBundleIdDefaultCtor` | ✅ | ctor | Default constructor: writes Loggable::vftable then the IdEntry<StoreBundleId> vtable and zeroes this+4 and this+8; no timestamp is loaded. |
| `00b28f50` | `IdEntryStoreBundleIdInitSetField20` | 🟢 | ctor | Inside an SEH frame, calls FUN_00b25e60 with the incoming this pointer and param_3 as the key pointer, copies *param_2 to this+0x20, and returns this. |
| `00b29240` | `HashListRemoveIdEntryStoreBundleIdNode` | 🟢 | function | Removes node param_2 from the 32-bucket chain (bucket = param_2[8] & 0x1f, table this+0x14, chain param_2[9]) and from the list (next/prev at +0x18/+0x1c, head this+8, tail this+0xc), decrements the count at this+0x10, … |
