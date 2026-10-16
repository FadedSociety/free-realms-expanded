# Class: SoeUtil::List<CharacterCreationManager::CharacterType,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097da10` | `List_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `009800a0` | `List_CharacterType_RemoveNode` | 🟢 | method | Removes node param_2 from a List<CharacterType> header: unlinks it from the doubly linked list (node+0x74 previous, node+0x70 next, header +4/+8), decrements the count at +0xc, destroys the node's two StringFixed<32> me… |
| `00981130` | `List_CharacterType_Clear ? ⚠downgrade` | 🟡 | method | Clear helper: repeatedly removes the head node (header +4) with 009800a0 until the list is empty. |
| `009814f0` | `List_CharacterType_Dtor` | ✅ | dtor | Destructor body of the CharacterType list: installs the List<CharacterType,-1> vtable and removes every node through 009800a0. |
| `009815b0` | `List_CharacterType_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the CharacterType list): installs the vtable, removes every node through 009800a0 and frees the object with _free when the delete flag is set. |
