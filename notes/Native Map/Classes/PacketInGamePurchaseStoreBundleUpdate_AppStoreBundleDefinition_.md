# Class: PacketInGamePurchaseStoreBundleUpdate<AppStoreBundleDefinition>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24090` | `PacketInGamePurchaseStoreBundleUpdate::Ctor ?` | 🟡 | ctor | Calls FUN_00b22f00 with inner tag 2; that base constructor builds family 66/sub-opcode 5 and stores the tag at +0x18. Then writes its own vtable and stores the caller?s 4-byte value at +0x20. |
| `00b24100` | `PacketInGamePurchaseStoreBundleUpdate::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor (slot 0): writes BasePacket::vftable, conditionally frees this via _free, returns this. |
