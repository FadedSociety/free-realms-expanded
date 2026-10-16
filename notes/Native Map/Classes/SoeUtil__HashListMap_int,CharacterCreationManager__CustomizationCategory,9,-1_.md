# Class: SoeUtil::HashListMap<int,CharacterCreationManager::CustomizationCategory,9,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097dad0` | `HashListMap_intCharacterCreationManager_CustomizationCategory9m1_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00980130` | `HashList9_RemoveCategoryNode` | 🟢 | method | Removes category node param_2 from its container: unlinks it from the bucket chain via 0097d390, unlinks the sorted-list links (node+0x80 previous, node+0x7c next, header +8/+0xc), decrements the count at +0x10, destroy… |
| `009811d0` | `HashList9_CustomizationCategory_Clear` | 🟢 | method | Clear helper: repeatedly removes the head node (header +8) with 00980130 until the container is empty. |
| `00981520` | `HashListMap_CustomizationCategory_Dtor` | ✅ | dtor | Destructor body: installs the CustomizationCategory vtable and removes every node through 00980130. |
| `009815f0` | `HashListMap_CustomizationCategory_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor (second vtable of the category cache): installs the vtable, removes every node with 00980130 and frees the object when the delete flag is set. |
| `00981890` | `HashListMap_CustomizationCategory_Dtor_EH` | 🟢 | dtor | Destructor body with SEH frame: installs the CustomizationCategory vtable and removes every node through 00980130. |
| `009824f0` | `HashListMap_CustomizationCategory_ScalarDeletingDtor2` | 🟢 | dtor | Deleting destructor wrapper: runs the EH destructor body 00981890 and frees the object with _free when the delete flag is set. |
