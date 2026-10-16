# Class: SoeUtil::HashMap<int,SoeUtil::IString<unsigned_short>,15,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a72820` | `HashMapIntWideString::Ctor` | ✅ | ctor | Stores the HashMap<int,IString<unsigned_short>> vtable at this, sets the count at +8 to 0, calls _memset(this+0xc, 0, 0x3c) for the 15-entry bucket table, sets +4 to 0 and returns this. |
| `00a73160` | `HashMapIntWideString::ClearAllNodes` | 🟢 | method | Walks the 15 bucket heads from this+0xc while the count at +8 is positive. For each bucket it clears the head and, for each node (next at node+0x14), stores the IString<unsigned_short> vtable at node+4, releases the wid… |
| `00a73350` | `HashMapIntWideString::DtorEntryJump` | 🟢 | dtor | Stores the HashMap<int,IString<unsigned_short>> vtable at this, then tail-jumps to 00a73160 (jmp rel32). |
| `00a73360` | `HashMapIntWideString::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Stores the HashMap<int,IString<unsigned_short>> vtable, calls the dtor body 00a73160 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a73390` | `HashMapIntWideString::DtorBody` | ✅ | dtor | Stores the HashMap<int,IString<unsigned_short>> vtable at this, then calls 00a73160 (clear all nodes) on this; does not free this. |
