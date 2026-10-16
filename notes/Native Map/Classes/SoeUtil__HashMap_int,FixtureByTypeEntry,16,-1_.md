# Class: SoeUtil::HashMap<int,FixtureByTypeEntry,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa6650` | `HashMapCtor_FixtureByType16` | ✅ | ctor | Stores SoeUtil::HashMap<int,FixtureByTypeEntry,16,-1>::vftable, zeroes the count and clears the 0x40-byte bucket array with _memset. |
| `00aa96d0` | `SoeUtil::HashMap::Allocate` | ✅ | virtual | Virtual allocation hook of the SoeUtil::HashMap instantiation: routes a fixed (0xc0, 8) request to the active memory manager (context from FUN_006fb9c0, manager from FUN_006fb860, virtual slot +4), else the global manag… |
| `00aa9710` | `SoeUtil::HashMap::Free` | ✅ | virtual | Virtual release hook of the SoeUtil::HashMap instantiation: forwards (pointer, 8) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc650` | `SoeUtil_HashMap_FixtureByType_16_Dtor` | ✅ | dtor | Destructor body: stores the HashMap<int,FixtureByTypeEntry,16,-1> vftable 01818b18 at +0 and tail-jumps to FUN_00ab9ec0, which clears the 16 buckets of nodes. |
| `00abc8a0` | `SoeUtil_HashMap_FixtureByType_16_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818b18, calls FUN_00ab9ec0 to clear the buckets, then calls _free on this when bit 0 of the flag argument is set. |
