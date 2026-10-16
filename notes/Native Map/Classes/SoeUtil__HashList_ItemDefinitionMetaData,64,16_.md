# Class: SoeUtil::HashList<ItemDefinitionMetaData,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c73c10` | `AdjustorThunk_plus114_to_00c73b60` | ✅ | virtual | Thunk: adds 0x114 to ecx (this) and jumps to 00c73b60 (bytes 81 c1 14 01 00 00 e9 rel32). Dispatches vtable slot [1] of the secondary base of HashListMap<int,ItemDefinitionMetaData,64,16>. |
| `00c73c20` | `AdjustorThunk_plus114_to_00c73bb0` | ✅ | virtual | Thunk: adds 0x114 to ecx (this) and jumps to 00c73bb0 (bytes 81 c1 14 01 00 00 e9 rel32). Dispatches vtable slot [2] of the secondary base of HashListMap<int,ItemDefinitionMetaData,64,16>. |
| `00c74a30` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the HashListMap 64 constructor FUN_00c73dd0 with the security cookie, then stores the SoeUtil::HashList<ItemDefinitionMetaData,64,16> vtable. |
| `00c74af0` | `SoeUtil::HashList_ItemDefinitionMetaData_64_16::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c731c0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
