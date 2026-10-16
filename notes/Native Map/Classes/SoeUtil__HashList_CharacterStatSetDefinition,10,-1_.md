# Class: SoeUtil::HashList<CharacterStatSetDefinition,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102bff0` | `HashList::AllocNode36` | ✅ | virtual | Virtual with no stack args: calls 006fb9c0, gets the current thread memory manager (006fb860); if present pushes (0x24,4) and calls its slot [1]; else global manager slot [1] (DAT_01bdf280); else calls 006fb950(0x24,4). |
| `0102c030` | `HashList::FreeNode` | ✅ | virtual | Thiscall virtual with one stack argument (ret 4): calls FreeMemory 00401c00 on the argument with alignment 4 and returns. |
| `0102ce90` | `HashList::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte): calls the HashListMap destructor body 0102cd50 on this, then calls _free(this) when bit 0 of the flag is set; returns this. |
