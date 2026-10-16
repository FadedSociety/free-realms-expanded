# Class: SoeUtil::HashList<SoeUtil::Array<unsigned_int,0,1>,10000,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9af0` | `SoeUtil::HashList::Allocate` | ✅ | virtual | Virtual allocation hook shared by the SoeUtil::HashListMap and SoeUtil::HashList instantiations for unsigned-int-array values: routes a fixed (0x20, 4) request to the active memory manager (context from FUN_006fb9c0, ma… |
| `00aa9b30` | `SoeUtil::HashList::Free` | ✅ | virtual | Virtual release hook shared by the HashListMap and HashList instantiations: forwards (pointer, 4) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc3a0` | `SoeUtil_HashList_ArrayUint_10000_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the HashList<Array<unsigned_int,0,1>,10000,-1> base): stores the derived HashListMap vftable 01818b38 at +0, calls FUN_00ab8910 (ClearAll), then calls _free on this when bit … |
