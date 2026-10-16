# Class: ProxiedCharactersDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb2740` | `FilteredRecordRowsRebuild` | 🟢 | method | Refreshes the backing source, clears the temporary entry array, walks source entries, filters them through a virtual predicate, appends selected records, and marks the row set ready. |
| `00cb2810` | `ProxiedCharactersDataSource::Dtor` | ✅ | dtor | Restores vtables, destroys the embedded entry array, frees its aligned storage, and releases its GUI user-data holder. |
| `00cb28c0` | `— ⚠reject` | ⚪ | virtual | Rebuilds rows when the dirty flag at this+0x228 is clear, then invokes a virtual method through the adjusted base pointer. |
| `00cb2920` | `ProxiedCharactersDataSource::Ctor` | ✅ | ctor | Initializes the GUI data-source base, stores the source argument, installs proxied-character vtables, clears the dirty state, and initializes its entry array and counts. |
| `00cb29c0` | `ProxiedCharactersDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the proxied-character data-source destructor and frees the object when the deleting flag's low bit is set. |
| `00cb29e0` | `ProxiedCharactersDataSource::GetCellValue` | ✅ | virtual | Ensures rows are built, resolves the requested row and source record, then maps requested columns to identifiers, strings, status flags, and other row values. |
