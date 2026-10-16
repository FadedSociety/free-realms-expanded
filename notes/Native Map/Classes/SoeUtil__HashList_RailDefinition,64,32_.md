# Class: SoeUtil::HashList<RailDefinition,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009208c0` | `HashListRailDefinition_Slot1Thunk` | ✅ | virtual | Adjustor thunk: adds 0x114 to ECX and jumps to 009207e0 (ChunkPoolAllocate_3). |
| `009208d0` | `HashListRailDefinition_Slot2Thunk` | ✅ | virtual | Adjustor thunk: adds 0x114 to ECX and jumps to 00920850 (ChunkPoolFree_3). |
| `00925040` | `HashList_RailDefinition_Ctor` | ✅ | ctor | Constructor of the HashList base: calls 00921130 on this (HashListMap_RailDefinition_Ctor, ECX = this), then stores the HashList<RailDefinition,64,32> vtable at this and returns this. |
| `00925260` | `HashList_RailDefinition_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091fe30 on this (bytes: call after the SEH frame setup), then frees the object through _free when bit 0 of the second argument is set; returns this. |
