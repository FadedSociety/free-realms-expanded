# Class: IgnoreListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf8350` | `— ⚠reject` | ⚪ | virtual | Returns one of three static strings for indices 0 through 2, or an error string for other indices. |
| `00cf8ca0` | `— ⚠reject` | ⚪ | virtual | Checks the row index against the row count, advances through the linked rows, then returns the row name or a related formatted value. |
| `00cf9b30` | `IgnoreListDataSource::Cleanup ?` | 🟡 | dtor | Restores the IgnoreListDataSource vtables, cleans up the embedded map, then destroys the embedded GUI userdata table. |
| `00cf9bc0` | `IgnoreListDataSource::GetRowCount ?` | 🟡 | virtual | Returns the row-count dword at offset 0x238. |
| `00cf9c90` | `IgnoreListDataSource::IgnoreListDataSource` | ✅ | ctor | Initializes the datasource base with BaseClient.IgnoreList, stores its constructor argument, installs the datasource vtables, and initializes the embedded IgnoreData hash list. |
| `00cf9d20` | `IgnoreListDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the ignore-list datasource cleanup and frees the object when the delete flag's low bit is set. |
