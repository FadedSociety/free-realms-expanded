# Class: InGamePurchaseMerchantListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1fec0` | `InGamePurchaseMerchantListPacketCleanup` | ✅ | dtor | Writes InGamePurchaseMerchantListPacket::vftable and then BasePacket::vftable at the object start, then returns. |
| `00b23ae0` | `InGamePurchaseMerchantListPacket::InGamePurchaseMerchantListPacket ? ⚠rename` | 🟡 | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 0x2a (42), writes InGamePurchaseMerchantListPacket::vftable and zeroes this+0x18. |
| `00b23b40` | `InGamePurchaseMerchantListPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: writes its own vftable, seeds try level -1, writes BasePacket::vftable and frees this via _free when the flags low bit is set. |
