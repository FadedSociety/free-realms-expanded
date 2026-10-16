# Class: SoeUtil::HashMap<int,FixtureByGroupEntry,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa66b0` | `HashMapCtor_FixtureByGroup16` | ✅ | ctor | Stores SoeUtil::HashMap<int,FixtureByGroupEntry,16,-1>::vftable, zeroes the count and clears the 0x40-byte bucket array with _memset. |
| `00aa97c0` | `SoeUtil::HashMap::Allocate` | ✅ | virtual | Virtual allocation hook of the SoeUtil::HashMap instantiation: routes a fixed (0xb4, 4) request to the active memory manager (context from FUN_006fb9c0, manager from FUN_006fb860, virtual slot +4), else the global manag… |
| `00aa9800` | `SoeUtil::HashMap::Free` | ✅ | virtual | Virtual release hook of the SoeUtil::HashMap instantiation: forwards (pointer, 4) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc660` | `SoeUtil_HashMap_FixtureByGroup_16_Dtor` | ✅ | dtor | Destructor body: stores the HashMap<int,FixtureByGroupEntry,16,-1> vftable 01818b28 at +0 and tail-jumps to FUN_00aba030, which clears the 16 buckets of nodes. |
| `00abc8d0` | `SoeUtil_HashMap_FixtureByGroup_16_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818b28, calls FUN_00aba030 to clear the buckets, then calls _free on this when bit 0 of the flag argument is set. |
