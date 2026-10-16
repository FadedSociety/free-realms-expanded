# Class: SoeUtil::List<ClientAccountManager::AccountInfoGameCharacterAndFriendsDetails*,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0070d4d0` | `AccountCharacterFriendList::Constructor` | ✅ | ctor | Installs list vtables, zeros head, tail, and count, then initializes backing storage. |
| `0070def0` | `AccountCharacterFriendList::Destructor` | ✅ | dtor | Installs list vtable, clears storage/nodes, then restores generic list vtable during teardown. |
| `0070dfa0` | `List_Callback_AllocateNodeThunk` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 0070de20. |
| `0070dfb0` | `List_Callback_FreeNodeThunk` | ✅ | virtual | Adjusts ECX by 0x10 and jumps to 0070de90. |
| `0070dfc0` | `AccountFriendListScalarDeletingDtor` | ✅ | dtor | Calls the specialized destructor; returns the original object when (param2 & 1) == 0 and frees it when bit 0 is set. |
