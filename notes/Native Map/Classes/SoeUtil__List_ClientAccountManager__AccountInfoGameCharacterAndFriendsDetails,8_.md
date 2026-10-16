# Class: SoeUtil::List<ClientAccountManager::AccountInfoGameCharacterAndFriendsDetails,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071a420` | `ListRemoveHead` | ✅ | virtual | Adjusts this by sixteen bytes and jumps to 0071a340, which removes and returns the free-list head. |
| `0071a430` | `ListInsertHead` | ✅ | virtual | Adjusts this by sixteen bytes and jumps to 0071a3b0, which links a slot at free-list head. |
| `0071a510` | `ListScalarDeletingDtor` | ✅ | dtor | Calls the list destructor and frees the object when the deleting flag bit is set. |
