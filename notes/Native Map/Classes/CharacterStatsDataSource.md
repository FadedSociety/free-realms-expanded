# Class: CharacterStatsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8e970` | `CharacterStatsDataSource::ColumnName` | ✅ | virtual | Returns Stat ID for column 0, Stat Value for column 1, and an error label for other indices. |
| `00c8eb30` | `CharacterStatsDataSource::GetStatValue` | ✅ | virtual | Resolves a character stat and returns its value field at +0x10, or zero if no matching entry exists. |
| `00c8eb50` | `— ⚠reject` | ⚪ | virtual | Resolves a stat row and returns stat ID, localized name, value formatted as GenericInteger, or other row data through the UI value API. |
| `00c8ecf0` | `CharacterStatsDataSource::Dtor` | ✅ | dtor | Restores CharacterStatsDataSource vtables and destroys embedded GUI data source state. |
| `00c8ed60` | `CharacterStatsDataSource::CharacterStatsDataSource` | ✅ | ctor | Registers BaseClient.PlayerStats, installs source vtables, and stores the supplied source dependency. |
| `00c8ede0` | `CharacterStatsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the source destructor and frees the object when deleting flag bit 0 is set. |
