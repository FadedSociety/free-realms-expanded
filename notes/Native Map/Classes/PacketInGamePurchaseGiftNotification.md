# Class: PacketInGamePurchaseGiftNotification

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1fdf0` | `— ⚠reject` | ⚪ | dtor | Destroys the gift-item array and string through FUN_01078020, then stores BasePacket::vftable at [param_1]. It is called by the gift-notification scalar deleting destructor and serialization/unwind code. |
| `00b23980` | `— ⚠reject` | ⚪ | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 0, writes PacketInGamePurchaseGiftNotification::vftable and then calls FUN_01077f70 with this to construct the embedded members (that callee's body references SoeUtil::ISt… |
| `00b239e0` | `PacketInGamePurchaseGiftNotification::ScalarDeletingDestructor` | ✅ | dtor | 27-byte scalar deleting destructor: pushes esi, moves ECX (this) to esi, calls FUN_00b1fdf0 (the class destructor body, which resets the BasePacket vtable and destroys members), then, if bit 0 of the flags argument is s… |
