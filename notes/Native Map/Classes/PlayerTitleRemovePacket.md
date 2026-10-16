# Class: PlayerTitleRemovePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b49410` | `PlayerTitleRemovePacket::Ctor` | ✅ | ctor | Constructs the base with sub-opcode 2 through FUN_00b491a0 and stores PlayerTitleRemovePacket::vftable (01821264); no payload fields are initialised beyond the base. |
| `00b49470` | `PlayerTitleRemovePacket::Dtor` | ✅ | dtor | Destructor body: restores the vtable chain PlayerTitleRemovePacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b494d0` | `PlayerTitleRemovePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |
