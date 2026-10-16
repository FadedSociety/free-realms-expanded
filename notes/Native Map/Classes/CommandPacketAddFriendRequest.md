# Class: CommandPacketAddFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d14c0` | `CommandPacketAddFriendRequest::~CommandPacketAddFriendRequest` | 🟢 | dtor | Destructor body of a command packet: rewrites the embedded SoeUtil::IString<char> member at +0x10 (param_1[4]) to its vtable, LOCK-decrements the shared refcount stored before the character buffer at +0x14 and calls the… |
| `008eb5c0` | `CommandPacketAddFriendRequestCtor` | ✅ | ctor | Constructs command packet with sub-opcode 14, installs derived vtable, initializes string member and copies argument. |
| `008eb650` | `CommandPacketAddFriendRequest::ScalarDeletingDtor` | ✅ | dtor | Calls packet destructor, frees object if flag bit zero is set, otherwise returns original pointer. |
