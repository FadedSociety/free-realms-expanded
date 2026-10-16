# Class: SoeUtil::Array<OverHeadElementSet*,64,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4b4c0` | `SoeUtil::Array_OverHeadElementSet64_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): calls FUN_00a49fd0 to release the array storage, frees the object via the _free thunk when (param_2 & 1) is set, and returns this; bytes 56 8b f1 e8 .. confirm the standa… |
| `00a4b700` | `SoeUtil::Array_OverHeadElementSet64_Swap` | ✅ | virtual | Vtable slot [3] swap method for the 64-entry OverHeadElementSet-pointer array: same temporary append/clear/append/clear/append sequence through FUN_00a49f80, ending with FreeMemory of the temporary storage. |
