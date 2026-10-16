# Class: FriendRemovePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7dff0` | `FriendRemovePacket::Construct` | ✅ | ctor | Calls the common friend packet constructor with sub-opcode 7, installs FriendRemovePacket::vftable, and copies two global default fields into object+0x10/+0x14. |
| `00c7e060` | `FriendRemovePacket::DeletingDtor` | ✅ | dtor | Restores BasePacket::vftable, frees the packet if flag bit 0 is set, and otherwise returns the original object pointer. |
