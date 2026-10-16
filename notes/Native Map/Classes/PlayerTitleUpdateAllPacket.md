# Class: PlayerTitleUpdateAllPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b497b0` | `PlayerTitleUpdateAllPacket::Ctor` | ✅ | ctor | Constructs the base with sub-opcode 3 through FUN_00b491a0 and stores PlayerTitleUpdateAllPacket::vftable (0182127c); no further payload is initialised. |
| `00b49810` | `PlayerTitleUpdateAllPacket::Dtor` | ✅ | dtor | Destructor body: restores the vtable chain PlayerTitleUpdateAllPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b49870` | `PlayerTitleUpdateAllPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |
