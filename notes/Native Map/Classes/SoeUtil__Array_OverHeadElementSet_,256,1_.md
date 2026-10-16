# Class: SoeUtil::Array<OverHeadElementSet*,256,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4b4a0` | `SoeUtil::Array_OverHeadElementSet256_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): calls FUN_00a49e90 to release the array storage, frees the object via the _free thunk when (param_2 & 1) is set, and returns this. Raw bytes 56 8b f1 e8 .. match the stan… |
| `00a4b610` | `SoeUtil::Array_OverHeadElementSet256_Swap` | ✅ | virtual | Vtable slot [3] swap method for the 256-entry OverHeadElementSet-pointer array: appends the other array's elements into a temporary (element copies through FUN_00a49f80), clears the other array, appends this array's ele… |
