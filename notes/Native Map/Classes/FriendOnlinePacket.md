# Class: FriendOnlinePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e910` | `FriendOnlinePacket::Construct` | ✅ | ctor | Calls friend-family initializer with sub-opcode 2, installs FriendOnlinePacket vtable, initializes embedded IString/default fields and clears the boolean at +0x28. |
| `00c7ed60` | `FriendOnlinePacket::DeletingDtor` | ✅ | dtor | Calls string-member destructor 00c7e9a0, conditionally frees on flag bit 0, and otherwise returns the object pointer. |
