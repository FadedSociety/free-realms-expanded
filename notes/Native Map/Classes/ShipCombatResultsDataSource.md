# Class: ShipCombatResultsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00caf230` | `ShipCombatResultsDataSource::GetColumnName` | ✅ | virtual | Returns the name for each column index, including Position, Guid, Name, Color, Gold, Kills, Score, and trophy fields. |
| `00caf380` | `ShipCombatResultsDataSource::GetCount` | ✅ | virtual | Returns the 32-bit value stored at object offset 0x230. |
| `00cafa50` | `ShipCombatResultsDataSource::Dtor` | ✅ | dtor | Installs results datasource vtables, destroys embedded row array, frees its storage, and calls inherited datasource cleanup. |
| `00cafb30` | `ShipCombatResultsDataSource::Ctor` | ✅ | ctor | Registers named results datasource, installs its vtables, stores passed owner, and initializes specialized results array. |
| `00cafbc0` | `ShipCombatResultsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls results datasource destructor and frees object when scalar deleting flag's low bit is set. |
| `00cafbe0` | `ShipCombatResultsDataSource::GetCellValue` | ✅ | virtual | Checks row index, selects a field by column ID, and writes the requested result value into the supplied output structure. |
