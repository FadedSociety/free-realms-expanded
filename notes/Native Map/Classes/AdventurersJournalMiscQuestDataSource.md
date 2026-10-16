# Class: AdventurersJournalMiscQuestDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c978c0` | `AdventurersJournalMiscQuestDataSource::ColumnName` | ✅ | virtual | Maps indices zero through twelve to miscellaneous quest fields, including quest identity and text, timestamps, progress, job data, objective count, membership, helper text, icon, and system flag. |
| `00c97d70` | `AdventurersJournalMiscQuestDataSource::GetCount` | ✅ | virtual | Walks active quest records, counts records whose ids are absent from the hub quest map, and returns the count. |
| `00c97de0` | `AdventurersJournalMiscQuestDataSource::Rebuild` | ✅ | method | Refreshes the source, walks active native quest records, emits quest ids absent from the hub-quest map, marks the source built, and returns a pointer-like value. |
| `00c97f00` | `AdventurersJournalMiscQuestDataSource::Refresh` | ✅ | virtual | Rebuilds miscellaneous quest rows when the dirty byte is clear, then calls the shared refresh helper. |
| `00c985f0` | `— ⚠reject` | ⚪ | virtual | Looks up a journal quest row, resolves its active quest record, then returns requested quest metadata, text, progress, job and membership values by column index. |
| `00c98a60` | `AdventurersJournalMiscQuestDataSource::Dtor` | ✅ | dtor | Restores miscellaneous quest data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98ce0` | `AdventurersJournalMiscQuestDataSource::Ctor` | ✅ | ctor | Initializes the named journal miscellaneous quest table, installs its vtables, stores one source pointer, and clears the dirty flag. |
| `00c98d60` | `AdventurersJournalMiscQuestDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the miscellaneous quest source destructor, frees the object when deleting flag bit one is set, and otherwise returns the original pointer. |
