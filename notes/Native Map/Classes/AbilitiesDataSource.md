# Class: AbilitiesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c917a0` | `AbilitiesDataSource::ColumnName` | ✅ | virtual | Returns Ability ID for index zero, Tooltip Description for index one, and an error label for other indices. |
| `00c917d0` | `AbilitiesDataSource::AdjustedVirtual → AbilitiesDataSource::Slot2AdjustorThunk` | ✅ | virtual | Adds four to ECX and jumps to 010c8f30, forwarding this call through the adjusted base-subobject pointer. |
| `00c91d30` | `— ⚠reject` | ⚪ | virtual | Checks that ability definitions are available, reads the indexed ability row, and returns either its numeric ID or a formatted tooltip description. |
| `00c91e60` | `AbilitiesDataSource::Dtor` | ✅ | dtor | Restores AbilitiesDataSource vtables, clears the embedded integer hash map, and destroys the GUI user-data table. |
| `00c91ef0` | `AbilitiesDataSource::Ctor` | ✅ | ctor | Initializes the BaseClient.Abilities GUI binding, installs AbilitiesDataSource vtables, and initializes its embedded 40-bucket integer hash map. |
| `00c91f70` | `AbilitiesDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls AbilitiesDataSource cleanup and frees the object only when the deleting flag's low bit is set. |
