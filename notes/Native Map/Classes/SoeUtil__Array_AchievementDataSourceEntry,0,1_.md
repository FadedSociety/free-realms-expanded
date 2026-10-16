# Class: SoeUtil::Array<AchievementDataSourceEntry,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce3c90` | `ArrayAchievementEntryGrowAlloc` | ✅ | virtual | Virtual slot [1] (returns the data pointer in eax): computes the new element count from param_2 and the capacity at +0xc (the same grow and shrink rules as the dsRaceResults array), writes the count to *param_3, returns… |
| `00ce3d30` | `ArrayAchievementEntryFreeBlock` | ✅ | virtual | Virtual slot [2] (one stack argument, ret 8): pushes alignment 4 and the pointer argument, calls FUN_00401c00 (FreeMemory), cleans its pushes and returns with ret 8. |
| `00ce3d50` | `ArrayAchievementEntrySwapFields ?` | 🟡 | virtual | Virtual slot [3] (one stack pointer and one byte flag): when the flag is nonzero it calls vtbl+0xc of the pointed-to object; otherwise it exchanges the dwords at this+4, +8 and +0xc with those of the pointed-to object (… |
| `00ce4920` | `ArrayAchievementEntryDtorBody` | ✅ | dtor | Thiscall destructor body (ret 0): stores the Array<AchievementDataSourceEntry> vtable at this, destroys the elements through FUN_00ce45a0 with the count at +8, frees the data at +4 with FUN_00401c00 (alignment 4), and n… |
| `00ce4950` | `ArrayAchievementEntryScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): stores the Array<AchievementDataSourceEntry> vtable, truncates the elements through FUN_00ce45a0, frees the data block with FUN_00401c00 (alignment 4) and nulls +4, then call… |
