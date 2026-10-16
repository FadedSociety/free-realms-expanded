# Class: ClaimDataSource::AvailableItems

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c69150` | `ClaimDataSource::AvailableItems::FieldName` | ✅ | virtual | Virtual method at slot 1 of the class vtable: switches on the field index and returns the literal name of each column - 0 ItemId, 1 StackCount, 2 ImageSetId, 3 ImageTintValue, 4 NameId, 5 DescriptionId, anything else th… |
| `00c69410` | `— ⚠reject` | ⚪ | virtual | Virtual method at slot 2: follows this+0x228 to the data table, then table+0x1c4, and returns the row count at that pointer's +0xa4, or 0 when the table is absent. |
| `00c69570` | `ClaimDataSource::AvailableItems::RefreshCachedCount` | ✅ | virtual | Virtual method of the secondary AvailableItems sub-object vtable: adjusts this back by 0x220 to reach the complete object, calls virtual slot +0xc on it, then resolves the table at +0x1c4 through this+4 and stores the c… |
| `00c695a0` | `ClaimDataSource::AvailableItems::GetFieldValue` | ✅ | virtual | Virtual method at slot 4, the field reader of the data source: for a valid row index (0 <= param_2 < the row count at table+0xa4) it computes the row as param_2*0x168 + the row array at table+0xa0 and answers field para… |
| `00c697f0` | `ClaimDataSource::AvailableItems::~AvailableItems` | ✅ | dtor | Destructor body: rewrites the four AvailableItems vtables at +0, +0x30, +0x34 and +0x254 (the last one temporarily as InternalDsBase), then runs the GuiUserDataTable base destructor FUN_0091bf60 under an SEH frame. |
| `00c69890` | `ClaimDataSource::AvailableItems::ScalarDeletingDtorThunk` | ✅ | virtual | Eleven-byte adjustor thunk at slot 0 of the secondary vtable: the bytes are sub ecx,0x254 followed by jmp 00c69a10, so it re-bases this onto the complete object and jumps into the scalar deleting destructor instead of r… |
| `00c69980` | `ClaimDataSource::AvailableItems::AvailableItems` | ✅ | ctor | Constructor taking a name pointer and an owner: runs the GUI data-table constructor FUN_00922890(param_2), stores the InternalDsBase vtable at +0x254, stores param_3 at +0x258 (the owner back-pointer), then overwrites +… |
| `00c69a10` | `ClaimDataSource::AvailableItems::ScalarDeletingDtor` | ✅ | virtual | Scalar deleting destructor at slot 0 of the primary vtable: calls the destructor body 00c697f0 and, when bit 0 of the flag is set, calls _free on this without returning. |
