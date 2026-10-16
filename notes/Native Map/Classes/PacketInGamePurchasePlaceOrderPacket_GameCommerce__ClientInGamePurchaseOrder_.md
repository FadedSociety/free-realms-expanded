# Class: PacketInGamePurchasePlaceOrderPacket<GameCommerce::ClientInGamePurchaseOrder>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b243d0` | `PacketInGamePurchasePlaceOrderPacket::Ctor ⚠downgrade` | 🟢 | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 3, writes PacketInGamePurchasePlaceOrderPacket vftable and stores the 4-byte argument (the order object) at this+0x18. |
| `00b24440` | `PacketInGamePurchasePlaceOrderPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
