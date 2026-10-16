# Class: PacketBaseInGamePurchase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b22e10` | `PacketBaseInGamePurchase::PacketBaseInGamePurchase` | ✅ | ctor | Writes BasePacket::vftable, stores 0x42 at this+4, writes PacketBaseInGamePurchase::vftable, and stores its caller-supplied sub-opcode at this+8. |
| `00b22e80` | `PacketBaseInGamePurchase::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): writes BasePacket::vftable to *this and, when the low bit of the flags argument is set, calls _free(this); returns this. |
