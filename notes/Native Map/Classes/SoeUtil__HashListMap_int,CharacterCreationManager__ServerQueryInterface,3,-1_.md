# Class: SoeUtil::HashListMap<int,CharacterCreationManager::ServerQueryInterface,3,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097dd90` | `HashListMap_intCharacterCreationManager_ServerQueryInterface3m1_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `0097ffa0` | `HashListMap_ServerQueryInterface_Dtor` | ✅ | dtor | Container destructor: writes the HashListMap<int,ServerQueryInterface,3,-1> vtable over this and drains every node with 0097f2b0; referenced from four unwind funclets. |
| `00980050` | `HashListMap_ServerQueryInterface_ScalarDeletingDtor` | ✅ | dtor | Deleting destructor: installs the HashListMap<int,ServerQueryInterface,3,-1> vtable, drains all nodes with 0097f2b0 and frees the object with _free when the flag bit 0 is set; sits in vtable slot 0 of that container. |
| `00980320` | `HashListMap_ServerQueryInterface_Dtor_EH` | ✅ | dtor | Destructor body with explicit SEH frame: installs the ServerQueryInterface container vtable and drains all nodes with 0097f2b0; referenced from two unwind funclets (the EH-wrapped twin of 0097ffa0). |
