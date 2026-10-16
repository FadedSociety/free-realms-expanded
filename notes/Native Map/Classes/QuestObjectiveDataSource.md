# Class: QuestObjectiveDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf7ae0` | `QuestObjectiveDataSource::GetColumnName` | ✅ | virtual | Returns Name, Description, Status, NotUsed, or Id for indices 0 through 5, and an error string for other indices. |
| `00cf7bc0` | `QuestObjectiveDataSource::GetRowCount ?` | 🟡 | virtual | Returns zero when the pointer at this+0x22c is null; otherwise returns the dword at the referenced object plus 0xf8. |
| `00cf7fd0` | `— ⚠reject` | ⚪ | virtual | Resolves the row's runtime quest, finds its objective by ID, then returns objective name, description, status, or identifier fields. |
| `00cf8180` | `QuestObjectiveDataSource::Cleanup ?` | 🟡 | dtor | Restores the QuestObjectiveDataSource vtables at the object and embedded offsets, then destroys its embedded GUI userdata table. |
| `00cf82a0` | `QuestObjectiveDataSource::QuestObjectiveDataSource ?` | 🟡 | ctor | Initializes its base, installs the QuestObjectiveDataSource vtables, stores two constructor arguments, and clears a state byte. |
| `00cf8330` | `QuestObjectiveDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the datasource cleanup routine and frees the object when the delete flag's low bit is set. |
