# Class: SoeUtil::HashListMap<int,SoeUtil::List<CharacterCreationManager::CustomizationData,-1>,9,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0097db30` | `HashListMap_intSoeUtil_List_CharacterCreationManager_CustomizationDatam19m1_All…` | ✅ | virtual | Virtual node allocator of the container: ensures the local CRT heap is initialised (FUN_006fb9c0), asks the current thread's memory manager (FUN_006fb860) vtable slot 1 for the node and, when neither manager is availabl… |
| `00982510` | `HashList9_InsertCustomizationDataListNode` | 🟢 | method | Insert for the list-valued 9-bucket container: allocates a node through vtable slot 1, constructs it with 009816b0 (which embeds a List<CustomizationData>), appends at the list tail (node+0x14 previous, node+0x10 next, … |
| `00982e70` | `HashList9_DestroyCustomizationDataLists` | 🟢 | method | Clears the list-valued 9-bucket container: repeatedly takes the head node at +8, unlinks it from bucket (key +0x18, %9, chain +0x1c) and list (node+0x10 next, node+0x14 previous), decrements the count, destroys the node… |
| `00983290` | `CustomizationDataMap_DestructBody` | ✅ | dtor | Destructor body of the HashListMap<int,List<CustomizationData>,9,-1> instantiation: the 11 bytes are 'mov dword ptr [ecx],0x0180b764' (store this class's vftable) followed by a 5-byte 'jmp 00982e70' (tail jump to the co… |
| `009832a0` | `CustomizationDataMap_DeletingDestruct` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of the HashListMap<int,List<CustomizationData>,9,-1> instantiation: stores the class vftable into [esi], calls the teardown 00982e70, then tests bit 0 of the flag argument; w… |
| `009832f0` | `CustomizationDataMap_Destruct` | ✅ | dtor | SEH-framed destructor body of the HashListMap<int,List<CustomizationData>,9,-1> instantiation: installs an exception frame (LAB_01599ad8), stores the class vftable into [param_1], parks the object in a stack slot and ca… |
