# Class: AdventurersJournalStickerDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c97850` | `AdventurersJournalStickerDataSource::ColumnName` | ✅ | virtual | Returns Sticker Id, Sticker Quest Id, Sticker Name, Sticker Description, Image Set Id, or Is Complete for indices zero through five. |
| `00c97cc0` | `AdventurersJournalStickerDataSource::GetCount` | ✅ | virtual | Walks sticker records and counts entries whose hub key matches the current hub, then returns the count. |
| `00c97d00` | `AdventurersJournalStickerDataSource::Rebuild ?` | 🟡 | method | Refreshes the source, walks sticker records, emits ids for entries whose hub key matches the selected hub, and marks the source built. |
| `00c97ed0` | `AdventurersJournalStickerDataSource::Refresh` | ✅ | virtual | Rebuilds sticker rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c98400` | `— ⚠reject` | ⚪ | virtual | Looks up a sticker row and returns its fields, string data, completion state, and hub membership; missing or unsupported rows return failure. |
| `00c989f0` | `AdventurersJournalStickerDataSource::Dtor` | ✅ | dtor | Restores sticker data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98c30` | `AdventurersJournalStickerDataSource::Ctor` | ✅ | ctor | Initializes the named journal sticker table, installs sticker source vtables, stores two source pointers, and clears the dirty flag. |
| `00c98cc0` | `AdventurersJournalStickerDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the sticker source destructor, frees the instance when deleting flag bit one is set, and otherwise returns its pointer. |
