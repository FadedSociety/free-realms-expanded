# Class: SoeUtil::List<ClientAccountManager::AccountInfoGameCharacterAndFriendsDetails*,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00709030` | `ListAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 12 bytes aligned to 4. |
| `00709070` | `ListFree` | ✅ | virtual | Passes the supplied pointer and alignment 4 to the memory-manager free routine. |
| `0070aec0` | `ListDtor → AccountDetailsPointerListDtor` | ✅ | dtor | Stores this list's vtable and drains its nodes through 00709d70. |
| `0070b0b0` | `ListScalarDeletingDtor` | ✅ | dtor | Stores the pointer-list vtable, drains nodes through 00709d70, and frees the object when the delete flag bit is set. |
