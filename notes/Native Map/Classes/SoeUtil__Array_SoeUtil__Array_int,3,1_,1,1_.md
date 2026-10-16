# Class: SoeUtil::Array<SoeUtil::Array<int,3,1>,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4b500` | `SoeUtil::Array_ArrayInt3_1_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): calls FUN_00a4a200 to destroy the nested Array<int,3,1> elements, frees the object via the _free thunk when (param_2 & 1) is set, and returns this. |
| `00a4d1c0` | `SoeUtil::Array_ArrayInt3_1_Swap` | ✅ | virtual | Vtable slot [3] swap method for the 1x3-int nested array: same copy-append/pop sequence through FUN_00a4c1b0 and FUN_00a47520, with the temporary buffer released at the end. |
