# Class: SoeUtil::Array<SoeUtil::Array<int,9,1>,3,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4b4e0` | `SoeUtil::Array_ArrayInt9_3_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): calls FUN_00a4a0c0 to destroy the nested Array<int,9,1> elements, frees the object via the _free thunk when (param_2 & 1) is set, and returns this. |
| `00a4d090` | `SoeUtil::Array_ArrayInt9_3_Swap` | ✅ | virtual | Vtable slot [3] swap method for the 3x9-int nested array: copies the argument's elements into a temporary through FUN_00a4c090, pops the argument through FUN_00a474d0, copies this array's elements into the argument, pop… |
