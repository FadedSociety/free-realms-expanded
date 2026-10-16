# Class: SoeUtil::List<AchievementData,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bac880` | `— ⚠reject` | ⚪ | ctor | Constructor: stores the List<AchievementData,-1> vtable, zeroes the fields at +4, +8 and +0xc, constructs the sub-object at +0x10 through 00baa850, then stores the List<AchievementData,32> vtable (01824be0). |
| `00bad2b0` | `— ⚠reject` | ⚪ | virtual | Adjustor thunk: adds 0x11 to ECX and jumps to NodePool_AllocNode copy 00bad1d0. Entry is vtable slot [2] of SoeUtil::List<AchievementData,32>. |
| `00bad2c0` | `— ⚠reject` | ⚪ | virtual | Adjustor thunk: adds 0x11 to ECX and jumps to NodePool_FreeNode copy 00bad240. Entry is vtable slot [3] of SoeUtil::List<AchievementData,32>. |
| `00bad2d0` | `List_AchievementData_32_DtorBody` | 🟢 | dtor | Destructor body: stores the List<AchievementData,32> vtable (01824be0), clears the list by calling 00bab670 on the head at +4 until empty, releases the pointer at +0x10 when non-null (00baacb0 on its +8 and +0xc, then 0… |
| `00bad3c0` | `List_AchievementData_32_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00bad2d0 on ECX, then _free on the object when the delete flag bit 0 is set, and returns the object pointer. |
