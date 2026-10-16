# Class: FriendOfflinePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7df20` | `FriendOfflinePacket::Construct` | ✅ | ctor | Calls the common friend packet constructor with sub-opcode 3, installs FriendOfflinePacket::vftable, and stores two arguments at object+0x10 and +0x14. |
| `00c7df90` | `FriendOfflinePacket::DeletingDtor` | ✅ | dtor | Restores BasePacket::vftable, conditionally frees the object when flag bit 0 is set, and otherwise returns its pointer. |
