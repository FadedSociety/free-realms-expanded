# Class: SoeUtil::List<CharacterCreationManager::CharacterNamesDataSourceElement,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097dd30` | `List_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00980f80` | `List_DestroyAllNameElementNodes` | 🟢 | method | Clears a List<CharacterNamesDataSourceElement>: repeatedly takes the head at +4, unlinks node+4/node+8 links, decrements the count at +0xc, releases the element with 0097e780 and returns the node through vtable slot 3. |
| `00981160` | `List_CharacterNamesElement_Dtor` | ✅ | dtor | Container destructor: installs the CharacterNamesDataSourceElement list vtable and drains all nodes with 00980f80. |
| `009811a0` | `List_CharacterNamesElement_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the name-element list vtable, drains nodes with 00980f80 and frees the object when the flag bit 0 is set. |
