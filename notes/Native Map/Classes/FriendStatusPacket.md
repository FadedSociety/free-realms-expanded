# Class: FriendStatusPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7ea20` | `FriendStatusPacket::Construct` | ✅ | ctor | Calls friend-family initializer with sub-opcode 9, installs FriendStatusPacket vtable, initializes embedded IString/default fields and zeroes trailing status/profile scalars. |
| `00c7eea0` | `FriendStatusPacket::DeletingDtor` | ✅ | dtor | Calls string-member destructor 00c7eac0 and frees the object only when flag bit 0 is set. |
