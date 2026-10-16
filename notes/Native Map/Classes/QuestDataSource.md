# Class: QuestDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf7a10` | `QuestDataSource::GetColumnName` | ✅ | virtual | Returns the column name for each index from Quest Id through System Quest, or an error string for unsupported indices. |
| `00cf7bb0` | `QuestDataSource::GetRowCount ?` | 🟡 | virtual | Returns the value at offset 0x14 of the object referenced by this+0x228. |
| `00cf7c70` | `— ⚠reject` | ⚪ | virtual | Populates objective rows when the source is marked dirty, then passes the supplied argument to FUN_010c90c0. |
| `00cf7ca0` | `QuestDataSource::SelectQuest ?` | 🟡 | method | Looks up the supplied quest identifier with FUN_00c7a610, stores the returned record at offset 0x25c, and rebuilds quest rows. |
| `00cf7cd0` | `QuestDataSource::GetCell ?` | 🟡 | virtual | Ensures rows are populated, resolves the row's runtime quest record, then writes the requested quest identifier, title, description, time, progress, flags, or related value. |
| `00cf8110` | `QuestDataSource::Cleanup ?` | 🟡 | dtor | Restores the QuestDataSource vtables at the object and embedded offsets, then destroys its embedded GUI userdata table. |
| `00cf81f0` | `QuestDataSource::QuestDataSource` | ✅ | ctor | Initializes the datasource base with BaseClient.QuestData, installs QuestDataSource vtables, stores two arguments, and clears the population state. |
| `00cf8280` | `QuestDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the datasource cleanup routine and frees the object when the delete flag's low bit is set. |
