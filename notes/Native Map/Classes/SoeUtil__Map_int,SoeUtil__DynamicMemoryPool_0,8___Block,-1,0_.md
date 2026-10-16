# Class: SoeUtil::Map<int,SoeUtil::DynamicMemoryPool<0,8>::Block,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010800c0` | `MapSlot2AllocNode0x24` | 🟢 | virtual | Virtual slot 2 (no stack args): calls 006fb9c0 once, then 006fb860 for the thread memory manager; if present calls its vslot 1; else the global manager DAT_01bdf280 vslot 1; else calls 006fb950(0x24, 4). Allocates a 0x2… |
| `01080100` | `MapSecondarySlot0Free4 ?` | 🟡 | virtual | Virtual slot 0 of the sub-vtable at vftable[3]: stdcall-style (ret 4), pushes alignment 4 and the pointer argument, calls 00401c00 (FreeMemory), and returns. Bytes decoded. |
| `010812c0` | `MapDtorBodyLayoutB` | 🟢 | dtor | Destructor body: stores the SoeUtil::Map<int,DynamicMemoryPool<0,8>::Block,-1,0> vftable, destroys the tree rooted at this+0x4 through 01080c40 on the root children, frees the external buffer through 010806a0, calls the… |
| `010813b0` | `MapScalarDeletingDtorLayoutB` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the Map<int,DynamicMemoryPool<0,8>::Block,-1,0> vftable, destroys the tree (children through 01080c40, buffer through 010806a0, container virtual slot 3 on the root), ze… |
