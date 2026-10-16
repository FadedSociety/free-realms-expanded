# Class: HousingDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbb310` | `HousingDataSource::ColumnHeaderText` | ✅ | virtual | Virtual slot 1 of HousingDataSource (thiscall, index argument): switches on indices 0 to 0x11 and returns a header string: 0 Instance Guid, 1 Owner Guid, 2 Owner Name, 3 Permission Level, 4 In Edit Mode, 5 Max Fixture C… |
| `00cbdc90` | `HousingDataSource::GetValue` | ✅ | virtual | Reads the active housing instance and maps column indices to its two 64-bit identifiers, strings, booleans, counts and numeric fields; some columns resolve house names and player-scoped strings. |
| `00cbf9b0` | `HousingDataSource::HousingDataSource` | ✅ | ctor | Registers BaseClient.Housing.Instance, installs class and housing-guid-array vtables, initializes storage, then creates and attaches the fixture list and fixture-item data sources. |
| `00cbfb00` | `— ⚠reject` | ⚪ | dtor | Deletes the two owned data-source objects when the global deletion guard is enabled, clears their pointers, frees the housing-guid array with alignment 8, clears its storage, and destroys the embedded holder. |
| `00cbfbf0` | `HousingDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00cbfb00 to release the housing data source's subviews, array and holder, then frees the object when the low flag bit is set. |
