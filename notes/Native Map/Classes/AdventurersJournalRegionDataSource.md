# Class: AdventurersJournalRegionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c97710` | `AdventurersJournalRegionDataSource::ColumnName` | ✅ | virtual | Returns one of eight region fields by index: Region Id, Region Name, Region Description, Tab Image Id, Chapter Map Image Id, Is Selected, CompletedStringId, or Should Be Hidden. |
| `00c97b30` | `— ⚠reject` | ⚪ | virtual | Reads the first record from the source's row list and returns the 32-bit value at record offset +0x10, or returns zero when empty. |
| `00c97bb0` | `AdventurersJournalRegionDataSource::Rebuild ?` | 🟡 | method | Refreshes the source, walks the linked records under its region data table, emits each record id, and marks the source built. |
| `00c97e70` | `AdventurersJournalRegionDataSource::Refresh` | ✅ | virtual | Rebuilds the region rows when the dirty byte is clear, then calls the shared data-source refresh helper. |
| `00c97f30` | `— ⚠reject` | ⚪ | virtual | Looks up a journal region row and returns selected fields, formats region strings, changes selected chapter state, and checks whether a region is selectable. |
| `00c98910` | `AdventurersJournalRegionDataSource::Dtor` | ✅ | dtor | Restores the region data-source vtable pointers and invokes the shared embedded GUI data-table destructor. |
| `00c98ad0` | `AdventurersJournalRegionDataSource::Ctor` | ✅ | ctor | Initializes the named journal region table, installs the region data-source vtables, stores two source pointers, and clears the dirty flag. |
| `00c98b60` | `AdventurersJournalRegionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the region data-source destructor, frees the object when deleting flag bit one is set, and otherwise returns the pointer. |
