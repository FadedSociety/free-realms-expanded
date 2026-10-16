# Class: SoeUtil::HashListMap<int,CharacterCreationManager::NameWheelCategory,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097dcc0` | `HashListMap_intCharacterCreationManager_NameWheelCategory4m1_AllocNode` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00983420` | `NameWheelCategoryMap_DestructBody` | ✅ | dtor | Destructor body of the HashListMap<int,CharacterCreationManager::NameWheelCategory,4,-1> instantiation: caches the node pointer at [this+8], stores the class vftable into [this], then loops while that head pointer is no… |
| `00983450` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (slot [0]) of the NameWheelCategory map: stores the class vftable, drains the container by repeatedly calling 009828e0 on the head node at [this+8] until empty, then calls the _free thunk 00d8… |
| `00983490` | `NameWheelCategoryMap_Destruct` | ✅ | dtor | SEH-framed destructor body of the NameWheelCategory map (frame LAB_01599b68): stores the class vftable into [param_1] and drains the container by calling 009828e0 on the head node at [param_1+8] until the list is empty,… |
