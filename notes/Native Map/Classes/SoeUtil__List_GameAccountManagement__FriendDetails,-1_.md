# Class: SoeUtil::List<GameAccountManagement::FriendDetails,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708760` | `ListAlloc` | ✅ | virtual | Initializes the list allocator path, asks the current memory manager for storage, falls back to the global manager, then requests 0xf8 bytes aligned to 8. |
| `007087a0` | `ListFree` | ✅ | virtual | Passes the supplied pointer and alignment 8 to the memory-manager free routine. |
| `00711ff0` | `FriendDetailsListDestructor` | ✅ | dtor | Installs friend-details list vtable and removes its head through 0070e530 until empty. |
