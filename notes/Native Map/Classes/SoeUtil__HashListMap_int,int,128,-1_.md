# Class: SoeUtil::HashListMap<int,int,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081c540` | `HashListMapInt128_DtorBody` | ✅ | dtor | Destructor body called from EH unwind funclets (does not free the object): restores the HashListMap<int,int,128,-1> vtable at [0], then unlinks nodes through 0081a300 (128-bucket removal, mask 0x7f) until the head at [2… |
| `0081c8b0` | `HashListMapInt128_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): restores the HashListMap<int,int,128,-1> vtable at [0], unlinks nodes through 0081a300 until the head at [2] is null, and frees this through the _free thunk wh… |
| `0081ca60` | `HashListMapInt128_DtorBodyUnwindCopy` | ✅ | dtor | Destructor body called from EH unwind funclets (does not free the object): restores the HashListMap<int,int,128,-1> vtable at [0] and unlinks nodes through 0081a300 until the head at [2] is null. Same body as 0081c540. |
