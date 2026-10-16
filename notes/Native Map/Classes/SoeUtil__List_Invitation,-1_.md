# Class: SoeUtil::List<Invitation,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102d500` | `List::AllocNode304` | ✅ | virtual | Virtual with no stack args: calls 006fb9c0, gets the current thread memory manager (006fb860); if present pushes (0x130,8) and calls its slot [1]; else global manager slot [1] (DAT_01bdf280); else calls 006fb950(0x130,8… |
| `0102d540` | `List::FreeNode` | ✅ | virtual | Thiscall virtual with one stack argument (ret 4): calls FreeMemory 00401c00 on the argument with alignment 8 and returns. |
| `0102de40` | `— ⚠reject` | ⚪ | method | Thiscall (ECX = list, param_2 = node): unlinks the node from the list (list+0x4 first and list+0x8 last, node+0x128 next and node+0x12c prev), decrements the count at list+0xc, calls the record member destructor 008d155… |
| `0102e090` | `List::ClearRemove ? ⚠downgrade` | 🟡 | function | Fastcall (ECX = list): while list+0x4 (first) is non-null, calls 0102de40 (List::RemoveNode) on the first node with ECX = list. |
| `0102e0e0` | `List::DtorBody → SoeUtil::List<Invitation,-1>::DtorBody` | ✅ | dtor | Fastcall destructor body (ECX = list): stores the Invitation list vftable, then while list+0x4 is non-null calls 0102de40 (RemoveNode) on the first node; no free. |
| `0102e150` | `List::DeletingDtor → SoeUtil::List<Invitation,-1>::DeletingDtor` | ✅ | dtor | Scalar deleting destructor (thiscall, flag byte): stores the Invitation list vftable, removes nodes through 0102de40 until the list is empty, and calls _free(this) when bit 0 of the flag is set; returns this. |
