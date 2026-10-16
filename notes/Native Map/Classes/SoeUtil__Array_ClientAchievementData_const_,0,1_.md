# Class: SoeUtil::Array<ClientAchievementData_const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce3dd0` | `ArrayClientAchievementPtrGrowAlloc` | ✅ | virtual | Virtual slot [1] (returns the data pointer in eax): computes the new element count from param_2 and the capacity at +0xc with the same grow and shrink rules as the other arrays; writes the count to *param_3; returns thi… |
| `00ce3e70` | `ArrayClientAchievementPtrFreeBlock` | ✅ | virtual | Virtual slot [2] (one stack argument, ret 8): pushes alignment 4 and the pointer argument, calls FUN_00401c00 (FreeMemory), cleans its pushes and returns with ret 8. |
| `00ce3e90` | `ArrayClientAchievementPtrSwapFields ?` | 🟡 | virtual | Virtual slot [3] (one stack pointer and one byte flag): when the flag is nonzero it calls vtbl+0xc of the pointed-to object; otherwise it exchanges the dwords at this+4, +8 and +0xc with those of the pointed-to object (… |
| `00ce40f0` | `ArrayClientAchievementPtrDtorBody` | ✅ | dtor | Thiscall destructor body (ret 0): stores the Array<ClientAchievementData_const*> vtable at this, zeroes the count at +8, frees the data at +4 with alignment 4 via FUN_00401c00, and nulls +4. |
| `00ce42a0` | `ArrayClientAchievementPtrScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (ret 4): stores the array vtable, zeroes the count at +8, frees the data at +4 with FUN_00401c00 (alignment 4), nulls +4, then calls _free(this) when bit 0 of its stack argument is se… |
