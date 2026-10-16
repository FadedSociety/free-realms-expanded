# Class: SoeUtil::HashList<CharacterCreationManager::ServerQueryInterface,3,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00980e20` | `HashList_ServerQueryInterface_Ctor` | ✅ | ctor | Constructor: installs first the HashListMap and finally the HashList<CharacterCreationManager::ServerQueryInterface,3,-1> vtable, zeroing head/tail/count and the three bucket heads at +4..+0x1c. |
| `00980e90` | `HashList_ServerQueryInterface_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the ServerQueryInterface container vtable, drains all nodes with 0097f2b0, frees the object with _free when the flag bit 0 is set. |
