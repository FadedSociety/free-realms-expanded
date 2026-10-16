# Class: PlayerPositionsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a85900` | `PlayerPositionsDataSource::GetColumnName` | ✅ | virtual | Column-title getter (vtable slot [1]): switches on the column index and returns the literals "Guid" (0), "Name" (1), "X Location" (2), "Z Location" (3), "Is Friend" (4), "Is Group Member" (5), "Is In Encounter" (6) or "… |
| `00a85a70` | `PlayerPositionsDataSource::GetRowCount` | ✅ | virtual | Row-count getter (vtable slot [2]): a 7-byte field getter that loads the dword at this+0x234 and returns it (the current number of position rows). |
| `00a85df0` | `PlayerPositionsDataSourceAppendRow` | 🟢 | method | Copies one PlayerPosition from param_2 into the table: allocates a node via 00a85cd0, copies the 64-bit guid from param_2[0]/[1], assigns the name string from param_2[3] with the SoeUtil IString assign routine 00402ce0,… |
| `00a85e70` | `— ⚠reject` | ⚪ | virtual | Lua/UI field getter (vtable slot [4]): walks the row list from this+0x22c (next link at +0x40, count at +0x234) to row param_2 and, by column param_3, writes the guid (0, via 0098e1f0), the name string (1, from +0xc via… |
| `00a85fb0` | `PlayerPositionsDataSourceClear` | 🟢 | method | Empties the positions table: destroys every node of the list at this+0x25c with 00a85d60 (list object at this+0x258) and then tail-calls the data source's vtable slot [4] to notify listeners. |
| `00a86050` | `PlayerPositionsDataSourceCtor` | ✅ | ctor | Constructor of the positions data source: registers the name through 00922890(PTR_s_BaseClient.PlayerPositions), stores the three class vtables, saves the owner at [0x95], installs the List<...PlayerPosition,-1> vtable … |
| `00a860f0` | `PlayerPositionsDataSourceDtor` | ✅ | dtor | Destructor body of the positions data source: installs the SoeUtil::List<...PlayerPosition,-1> vtable into the list sub-object at this+0x258 and drains the row list at this+0x25c through 00a85d60, then calls the base da… |
| `00a86170` | `PlayerPositionsDataSourceScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of PlayerPositionsDataSource: runs the destructor body 00a860f0 and frees the object with _free when the delete flag bit 0 is set. |
