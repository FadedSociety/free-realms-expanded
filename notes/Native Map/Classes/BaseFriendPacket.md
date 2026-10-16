# Class: BaseFriendPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7d870` | `BaseFriendPacket::Construct` | ✅ | ctor | Installs BasePacket then BaseFriendPacket vtables, stores family id 0x4a and the supplied friend sub-opcode at object+8. |
| `00c7dec0` | `BaseFriendPacket::DeletingDtor` | ✅ | dtor | Restores BasePacket::vftable, frees the object if flag bit 0 is set, and otherwise returns the original pointer. |
