# Class: PlayerTitleRefreshRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b49680` | `PlayerTitleRefreshRequestPacket::Ctor` | ✅ | ctor | Constructs the base with sub-opcode 5 through FUN_00b491a0 and stores PlayerTitleRefreshRequestPacket::vftable (01821274); no further payload is initialised. |
| `00b496e0` | `PlayerTitleRefreshRequestPacket::Dtor` | ✅ | dtor | Destructor body: restores the vtable chain PlayerTitleRefreshRequestPacket::vftable, BasePlayerTitlePacket::vftable, BasePacket::vftable; no members are released. |
| `00b49740` | `PlayerTitleRefreshRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the full vtable chain and calls _free(this) when the low bit of the deleting flag is set, otherwise returns this. |
