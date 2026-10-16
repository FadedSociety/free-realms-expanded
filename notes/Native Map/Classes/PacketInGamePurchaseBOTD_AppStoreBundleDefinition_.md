# Class: PacketInGamePurchaseBOTD<AppStoreBundleDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24230` | `PacketInGamePurchaseBOTD::Ctor` | 🟢 | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 0x29 (41), writes PacketInGamePurchaseBOTD vftable and stores the two 4-byte arguments at this+0x18 and this+0x1c. |
| `00b242a0` | `PacketInGamePurchaseBOTD::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
