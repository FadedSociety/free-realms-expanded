# Class: SoeUtil::HashListMap<int,ClientAchievementData,256,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bac860` | `HashListMap_AllocNodeThunk` | ✅ | virtual | Adjustor thunk: adds 0x414 to ECX and jumps to NodePool_AllocNode (00bac780). Entry is vtable slot [1] of the HashListMap<int,ClientAchievementData,256,8> and HashList<ClientAchievementData,256,8> vtables. |
| `00bac870` | `HashListMap_FreeNodeThunk` | ✅ | virtual | Adjustor thunk: adds 0x414 to ECX and jumps to NodePool_FreeNode (00bac7f0). Entry is vtable slot [2] of the HashListMap<int,ClientAchievementData,256,8> and HashList<ClientAchievementData,256,8> vtables. |
| `00bad3a0` | `HashListMap_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the base destructor body 00babdd0 on ECX, then calls _free on the object when the delete flag bit 0 is set, and returns the object pointer. |
| `00bad3e0` | `HashListMap_ClientAchievementData_256_8_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,ClientAchievementData,256,-1> vtable, zeroes the fields at +8, +0xc, +0x10, clears the 0x400-byte bucket array at +0x14 with memset, zeroes +4, stores the HashListMap<...,256,8> v… |
