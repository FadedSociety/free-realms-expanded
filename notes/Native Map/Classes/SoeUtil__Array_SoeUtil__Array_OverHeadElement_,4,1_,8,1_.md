# Class: SoeUtil::Array<SoeUtil::Array<OverHeadElement*,4,1>,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4b480` | `SoeUtil::Array_ArrayOverHeadElement4_8_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): calls FUN_00a49d60 to destroy the nested-array elements and buffers, then, when the deletion flag (param_2 & 1) is set, frees the object through the _free thunk and retur… |
| `00a4cf50` | `SoeUtil::Array_ArrayOverHeadElement4_8_Swap` | ✅ | virtual | Vtable slot [3] swap method: copy-constructs a temporary of the base Array<...0,1> type from the argument array through FUN_00a4bf70 (vtable 01814470 stored in the temporary), pops the argument's elements (FUN_00a47280 … |
