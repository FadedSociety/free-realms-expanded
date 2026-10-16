# Class: SoeUtil::HashList<int,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081da40` | `HashListInt128_Ctor` | ✅ | ctor | Fastcall constructor (ecx=this): stores the HashListMap<int,int,128,-1> vtable, zeroes [2], [3] and [4], calls _memset on the 128-entry bucket array at [5] (0x200 bytes), zeroes [1], then stores the HashList<int,128,-1>… |
| `0081db20` | `HashListInt128_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081ca60 (the 128-bucket int HashListMap destructor body) on this, and frees this through the _free thunk when flag bit 0 is set. Returns this. |
