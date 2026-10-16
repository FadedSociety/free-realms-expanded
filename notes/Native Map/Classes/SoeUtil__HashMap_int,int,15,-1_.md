# Class: SoeUtil::HashMap<int,int,15,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a727b0` | `— ⚠reject` | ⚪ | ctor | Stores the HashMap<int,int> vtable at this, sets the count at +8 to 0, calls _memset(this+0xc, 0, 0x3c) to clear the 15-entry bucket table, sets +4 to 0 and returns this (ret 0 cdecl). |
| `00a72930` | `HashMapIntInt::ClearAllNodes` | 🟢 | method | Walks the 15 bucket heads starting at this+0xc while the count at +8 is positive; for each bucket it takes the head, clears the bucket, then for each node it reads next at node+8, calls vtable slot +8 of this on the nod… |
| `00a729e0` | `HashMapIntInt::DtorEntryJump` | 🟢 | dtor | Stores the HashMap<int,int> vtable at this, then tail-jumps to the destructor body 00a72930 (jmp rel32, no return here). |
| `00a72ad0` | `HashMapIntInt::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Stores the HashMap<int,int> vtable, calls the dtor body 00a72930 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a72d60` | `HashMapIntInt::DtorBody` | ✅ | dtor | Stores the HashMap<int,int> vtable at this, then calls 00a72930 (clear all nodes) on this; does not free this. |
