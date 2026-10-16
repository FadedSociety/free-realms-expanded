# Class: SoeUtil::List<ClientAccountManager::AccountInfoGameCharacterAndFriendsDetails,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708d30` | `ListAlloc → AccountDetailsListAlloc` | ✅ | virtual | Uses the current memory manager or global fallback to allocate 0x62b8 bytes aligned to 8. |
| `00708d70` | `ListFree → AccountDetailsListFree` | ✅ | virtual | Passes the supplied pointer and alignment 8 to the memory-manager free routine. |
| `0071a440` | `ListDestructor` | ✅ | dtor | Installs list vtable, repeatedly releases nodes, frees both tree branches, and clears counts and roots. |
