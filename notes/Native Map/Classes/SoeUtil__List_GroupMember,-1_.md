# Class: SoeUtil::List<GroupMember,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102d400` | `List::AllocNode216` | ✅ | virtual | Virtual with no stack args: calls 006fb9c0, gets the current thread memory manager (006fb860); if present pushes (0xd8,8) and calls its slot [1]; else global manager slot [1] (DAT_01bdf280); else calls 006fb950(0xd8,8). |
| `0102d440` | `List::FreeNode` | ✅ | virtual | Thiscall virtual with one stack argument (ret 4): calls FreeMemory 00401c00 on the argument with alignment 8 and returns. |
| `0102e0b0` | `List::DtorBody → SoeUtil::List<GroupMember,-1>::DtorBody` | ✅ | dtor | Fastcall destructor body (ECX = list): stores the GroupMember list vftable, then while list+0x4 is non-null calls 008faac0 on the first node. |
| `0102e110` | `List::DeletingDtor → SoeUtil::List<GroupMember,-1>::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte): stores the GroupMember list vftable, removes nodes through 008faac0 until the list is empty, and calls _free(this) when bit 0 of the flag is set; returns this. |
