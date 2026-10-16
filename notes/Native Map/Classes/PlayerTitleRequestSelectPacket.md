# Class: PlayerTitleRequestSelectPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b49540` | `PlayerTitleRequestSelectPacket::Ctor` | ✅ | ctor | Constructs the base with sub-opcode 4 through FUN_00b491a0, stores PlayerTitleRequestSelectPacket::vftable (0182126c) and writes its argument into the payload word at +0xc. |
| `00b495b0` | `PlayerTitleRequestSelectPacket::Dtor` | ✅ | dtor | Destructor body: restores the vtable chain PlayerTitleRequestSelectPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b49610` | `PlayerTitleRequestSelectPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |
