# Class: SoeUtil::HashList<HousingProcessor::CouplingDisplayData,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9b50` | `SoeUtil::HashList::Allocate` | ✅ | virtual | Virtual allocation hook shared by the SoeUtil::HashListMap and SoeUtil::HashList instantiations for HousingProcessor::CouplingDisplayData values: routes a fixed (0x2c, 4) request to the active memory manager (FUN_006fb9… |
| `00aa9b90` | `SoeUtil::HashList::Free` | ✅ | virtual | Virtual release hook shared by the HashListMap and HashList instantiations for HousingProcessor::CouplingDisplayData values: forwards (pointer, 4) to the shared free entry FUN_00401c00 (FreeMemory). |
| `00abc400` | `SoeUtil_HashList_CouplingDisplayData_1024_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the HashList<CouplingDisplayData,1024,-1> base): calls FUN_00abac30 on this, then calls _free when bit 0 of the flag argument is set. |
