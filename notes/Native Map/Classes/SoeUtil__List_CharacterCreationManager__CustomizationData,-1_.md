# Class: SoeUtil::List<CharacterCreationManager::CustomizationData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097ddf0` | `List_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00980280` | `List_CustomizationData_RemoveNode` | 🟢 | method | Removes node param_2 from a List<CustomizationData> header: unlinks node+0xa4/node+0xa0 from the list (header +4/+8), decrements the count at +0xc, destroys the node with 0097eab0 and returns it through vtable slot 3 (F… |
| `00981210` | `List_CustomizationData_Clear` | 🟢 | method | Clear helper: repeatedly removes the head node (header +4) with 00980280 until the list is empty. |
| `00981580` | `List_CustomizationData_Dtor` | ✅ | dtor | Destructor body: installs the List<CustomizationData,-1> vtable and removes every node through 00980280. |
| `00981670` | `List_CustomizationData_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the List<CustomizationData,-1> vtable, removes every node with 00980280 and frees the object when the delete flag is set. |
| `009816b0` | `List_CustomizationData_CtorWithArg` | 🟢 | ctor | Constructor of the List<CustomizationData,-1> used as a hash-list value: installs the vtable, zeroes head/tail/count (+4/+8/+0xc) and stores *param_2 into the extra field at +0x18. |
| `00981820` | `List_CustomizationData_Dtor_EH` | 🟢 | dtor | Destructor body with SEH frame: installs the List<CustomizationData,-1> vtable and removes every node through 00980280. |
| `00982690` | `List_CustomizationData_ScalarDeletingDtor2` | 🟢 | dtor | Deleting destructor wrapper: runs the EH destructor body 00981820 and frees the object with _free when the delete flag is set. |
