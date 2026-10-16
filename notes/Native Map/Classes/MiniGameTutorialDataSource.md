# Class: MiniGameTutorialDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8d510` | `MiniGameTutorialDataSource::ColumnName` | ✅ | virtual | Maps indices 0 through 4 to UI labels including Icon ID and returns a default label for other indices. |
| `00c8d730` | `MiniGameTutorialDataSource::GetCount` | ✅ | virtual | Returns the dword at object offset 0x23c. |
| `00c8df70` | `SetKeyedByteOrInsert` | 🟢 | method | Constructs a tutorial entry string, links the new node into the list and hash buckets, and increments stored row count. |
| `00c8e080` | `MiniGameTutorialDataSource::GetColumnValue` | ✅ | virtual | Resolves a tutorial row and returns its ID, localized title, icon, placeholder TODO column, or boolean field through UI value helpers. |
| `00c8e1b0` | `MiniGameTutorialDataSource::Dtor` | ✅ | dtor | Restores tutorial data source vtables, destroys its entry table, calls an additional cleanup routine, and destroys embedded GUI state. |
| `00c8e240` | `MiniGameTutorialDataSource::ScalarDeletingAdjustor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to 0x00c8e330; this is an adjustor thunk, not the destructor body. |
| `00c8e250` | `MiniGameTutorialDataSource::MiniGameTutorialDataSource` | ✅ | ctor | Registers BaseClient.MiniGameTutorials, installs source vtables, initializes data-source dependencies and table fields, and invokes the tutorial file loader. |
| `00c8e330` | `MiniGameTutorialDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the class destructor and frees the object when deleting flag bit 0 is set. |
