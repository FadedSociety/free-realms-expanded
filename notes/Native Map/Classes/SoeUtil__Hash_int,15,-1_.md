# Class: SoeUtil::Hash<int,15,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a729f0` | `Hash_AllocNode0xc` | ✅ | virtual | Virtual slot [1] of Hash<int,15,-1> and HashMap<int,int,15,-1>. Calls 006fb9c0, then gets the current thread's memory manager from 006fb860; when present it calls that manager's virtual slot 1 with (size 0xc, alignment … |
| `00a72db0` | `Hash_Int15_Ctor` | ✅ | ctor | Installs the HashMap<int,int,15,-1> vtable, sets the count at +8 to 0, calls _memset(this+0xc, 0, 0x3c) for the 15-entry bucket table, sets +4 to 0, then installs the Hash<int,15,-1> vtable as the final vptr. Returns th… |
| `00a72e30` | `Hash_Int15_ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of Hash<int,15,-1> (also the scalar deleting destructor of HashMap<int,int,15,-1>). Stores the HashMap<int,int> vtable, calls 00a72930 (clear all nodes) on this, then calls _free on this when bit 0 of i… |
