# Class: PlayerTitleAddPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b492d0` | `PlayerTitleAddPacket::Ctor` | ✅ | ctor | Constructs the base with sub-opcode 1 through FUN_00b491a0, stores PlayerTitleAddPacket::vftable (0182125c) and zeroes the four payload words at +0xc, +0x10, +0x14 and +0x18. |
| `00b49340` | `PlayerTitleAddPacket::Dtor` | ✅ | dtor | Destructor body: restores the vtable chain PlayerTitleAddPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b493a0` | `PlayerTitleAddPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set; otherwise returns this. |
