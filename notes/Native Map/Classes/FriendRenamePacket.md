# Class: FriendRenamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e810` | `FriendRenamePacket::Construct` | ✅ | ctor | Calls friend-family initializer with sub-opcode 10, installs FriendRenamePacket vtable, initializes an embedded IString and copies two global defaults. |
| `00c7ed40` | `FriendRenamePacket::DeletingDtor` | ✅ | dtor | Calls string-member destructor 00c7e890, frees the object when flag bit 0 is set, otherwise returns the pointer. |
