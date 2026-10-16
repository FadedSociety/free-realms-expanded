# Class: SoeUtil::HashList<ClientFirstTimeEventDefinitionManager::Entry,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009209c0` | `HashListClientFirstTimeEntry_Slot1Thunk` | ✅ | virtual | Adjustor thunk: adds 0x114 to ECX and jumps to 00872620. |
| `009209d0` | `HashListClientFirstTimeEntry_Slot2Thunk` | ✅ | virtual | Adjustor thunk: adds 0x114 to ECX and jumps to 00872690. |
| `009250a0` | `HashList_ClientFirstTimeEntry_Ctor` | ✅ | ctor | Constructor of the HashList base: calls 009211b0 on this (HashListMap_ClientFirstTimeEntry_Ctor, ECX = this), then stores the HashList<ClientFirstTimeEventDefinitionManager::Entry,64,32> vtable at this and returns this. |
| `009252c0` | `HashList_ClientFirstTimeEntry_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 009208e0 on this (bytes: call after the SEH frame setup), then frees the object through _free when bit 0 of the second argument is set; returns this. |
