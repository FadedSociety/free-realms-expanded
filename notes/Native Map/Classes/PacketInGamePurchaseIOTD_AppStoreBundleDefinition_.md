# Class: PacketInGamePurchaseIOTD<AppStoreBundleDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24160` | `PacketInGamePurchaseIOTD::Ctor` | 🟢 | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 0x20 (32), writes PacketInGamePurchaseIOTD vftable, stores the first 4-byte argument at this+0x18 and the second at this+0x1c. |
| `00b241d0` | `PacketInGamePurchaseIOTD::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
