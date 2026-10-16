# Class: SoeUtil::Hash<SoeUtil::IString<unsigned_short>,15,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a72a50` | `Hash_AllocNode0x18` | ✅ | virtual | Virtual slot [1] of HashMap<int,IString<unsigned_short>,15,-1> and Hash<IString<unsigned_short>,15,-1>. Same routing as 00a729f0 (memory manager from 006fb860, otherwise DAT_01bdf280, otherwise 006fb950), but with size … |
| `00a73530` | `Hash_IntWide15_Ctor` | ✅ | ctor | Installs the HashMap<int,IString<unsigned_short>,15,-1> vtable, sets the count at +8 to 0, calls _memset(this+0xc, 0, 0x3c) for the 15-entry bucket table, sets +4 to 0, then installs the Hash<IString<unsigned_short>,15,… |
| `00a735b0` | `Hash_IntWide15_ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of Hash<IString<unsigned_short>,15,-1>. Stores the HashMap<int,IString<unsigned_short>> vtable, calls 00a73160 (clear all nodes) on this, then calls _free on this when bit 0 of its stack argument is set… |
