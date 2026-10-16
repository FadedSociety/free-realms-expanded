# Class: SoeUtil::HashList<SoeUtil::IString<char>,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081fa00` | `HashListIString256_Ctor` | ✅ | ctor | Fastcall constructor (ecx=this): stores the HashListMap<int,SoeUtil::IString<char>,256,-1> vtable, zeroes [2], [3] and [4], calls _memset on the 256-entry bucket array at [5] (0x400 bytes), zeroes [1], then stores the H… |
| `0081fa80` | `HashListIString256_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, one stack byte flag): calls 0081f990 (the 256-bucket IString HashListMap destructor body) on this, and frees this through the _free thunk when flag bit 0 is set. Returns this. |
