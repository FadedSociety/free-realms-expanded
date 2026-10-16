# Class: SoeUtil::HashListMap<int,CharacterCreationManager::CharacterCreateDataSource,9,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097dba0` | `HashListMap_intCharacterCreationManager_CharacterCreateDataSource9m1_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00980ef0` | `HashList9_DestroyAllCharacterCreateDataSourceNodes` | 🟢 | method | Clears a 9-bucket CharacterCreateDataSource container: repeatedly takes the head node at +8, unlinks it from bucket (key +0xc, %9, chain +0x10) and list (node+4 next, node+8 previous), decrements the count at +0x10, rel… |
| `00981150` | `HashListMap_CharacterCreateDataSource_Dtor` | ✅ | dtor | Container destructor: installs the CharacterCreateDataSource vtable and drains all nodes with 00980ef0. |
| `00981170` | `HashListMap_CharacterCreateDataSource_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the CharacterCreateDataSource vtable, drains all nodes with 00980ef0 and frees the object with _free when the flag bit 0 is set. |
| `00981230` | `HashListMap_CharacterCreateDataSource_Dtor_EH` | ✅ | dtor | Destructor body with SEH frame: installs the CharacterCreateDataSource vtable and drains all nodes with 00980ef0. |
| `00981470` | `HashListMap_CharacterCreateDataSource_ScalarDeletingDtor2` | ✅ | dtor | Deleting destructor (second copy): installs the CharacterCreateDataSource vtable, drains with 00980ef0 and frees the object when the delete flag is set. |
