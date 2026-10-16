# Class: Engine::PendingFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `013c0e20` | `Engine::PendingFriendRequest::PendingFriendRequest` | ✅ | ctor | Constructor body (fastcall, this in ecx): stores the Engine::PendingFriendRequest vtable at +0, then initialises four std::string objects at +0x04, +0x20, +0x3c and +0x58 (size 0, capacity 15, first byte 0 at +0x08, +0x… |
| `013c0eb0` | `Engine::PendingFriendRequest::~PendingFriendRequest` | ✅ | dtor | Destructor body: re-stores the Engine::PendingFriendRequest vtable at +0, then for each of the four string objects (capacity at +0x70, +0x54, +0x38, +0x1c) releases the heap buffer when the capacity exceeds 15 and reset… |
| `013c0f90` | `Engine::PendingFriendRequest::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of the class (bytes 56 8b f1 / e8 / f6 44 24 08 01 / 74 09 / 56 / e8 / 83 c4 04 / 8b c6 / 5e c3): saves esi, calls the destructor body FUN_013c0eb0, calls _free(this) only when bit 0 of the argument is … |
| `013c1350` | `Engine::PendingFriendRequest::SetField04` | 🟢 | method | Assigns the argument std::string to the string member at this+0x04 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 04 / e8 / c2 04 00, this in ecx). |
| `013c1370` | `Engine::PendingFriendRequest::SetField20` | 🟢 | method | Assigns the argument std::string to the string member at this+0x20 (assign(src, 0, npos)). |
| `013c1390` | `Engine::PendingFriendRequest::SetField3C` | 🟢 | method | Assigns the argument std::string to the string member at this+0x3c (assign(src, 0, npos)). |
| `013c13b0` | `Engine::PendingFriendRequest::SetField58` | 🟢 | method | Assigns the argument std::string to the string member at this+0x58 (assign(src, 0, npos)). |
