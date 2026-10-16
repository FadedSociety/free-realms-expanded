# Class: AdventurersJournalHubDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c977a0` | `AdventurersJournalHubDataSource::ColumnName` | ✅ | virtual | Maps indices zero through ten to hub fields including quest id, title, description, active and complete state, and map coordinates. |
| `00c97c10` | `AdventurersJournalHubDataSource::GetCount ⚠downgrade` | 🟢 | virtual | Walks the linked row entries and counts those whose key matches the active hub id, then returns the count. |
| `00c97c50` | `AdventurersJournalHubDataSource::Rebuild ?` | 🟡 | method | Refreshes the source, walks hub records, emits ids whose hub key matches the selected hub, then marks the source built. |
| `00c97ea0` | `AdventurersJournalHubDataSource::Refresh` | ✅ | virtual | Rebuilds hub rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c98170` | `AdventurersJournalHubDataSource::GetFieldValue` | ✅ | virtual | Looks up a hub row and returns its fields by column, including hub text, active quest title and description, completion description, and map coordinates. |
| `00c98980` | `AdventurersJournalHubDataSource::Dtor` | ✅ | dtor | Restores the hub data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98b80` | `AdventurersJournalHubDataSource::Ctor` | ✅ | ctor | Initializes the named journal hub table, installs the hub source vtables, stores two source pointers, and clears its dirty flag. |
| `00c98c10` | `AdventurersJournalHubDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the hub data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |
