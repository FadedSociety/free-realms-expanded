# Class: SoeUtil::HashListMap<int,HousingProcessor::CouplingDisplayData,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa6b20` | `HashListMapCtor_CouplingDisplay1024` | ✅ | ctor | Stores SoeUtil::HashListMap<int,HousingProcessor::CouplingDisplayData,1024,-1>::vftable, zeroes head/tail/count and clears a 0x1000-byte bucket array with _memset. |
| `00aba120` | `SoeUtil_HashListMap_CouplingDisplayData_1024_Dtor` | ✅ | dtor | Destructor body: reads the count at +8, stores the HashListMap<int,HousingProcessor::CouplingDisplayData,1024,-1> vftable 01818b48 at +0, then while the count is nonzero calls FUN_00ab2b10 on the node list and reloads t… |
| `00aba3f0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (vtable slot 0): stores vftable 01818b48, drains the node list through FUN_00ab2b10 while the count at +8 is nonzero, then calls _free on this when bit 0 of the flag argument is set. |
| `00abac30` | `SoeUtil_HashListMap_CouplingDisplayData_1024_Dtor2` | ✅ | dtor | Destructor body: reads the count at +8, stamps the HashListMap<int,HousingProcessor::CouplingDisplayData,1024,-1> vftable 01818b48 at +0, and while the count is nonzero calls FUN_00ab2b10 on the node list and reloads th… |
| `00abc300` | `SoeUtil_HashListMap_CouplingDisplayData_1024_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,HousingProcessor::CouplingDisplayData,1024,-1> vftable 01818b48, zeroes the count and the 1024 bucket heads (0x1000 bytes) through FUN_00d836a0, then stamps the HashList<CouplingD… |
