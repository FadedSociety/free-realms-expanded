# Class: GuildDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cb53f0` | `GuildDataSource::ColumnName` | ✅ | virtual | Returns column labels for guild GUID, name, message of the day, and guild creation/renaming permissions, with an error label for other indices. |
| `00cb5710` | `GuildDataSource::GetRowCount ⚠downgrade` | 🟢 | virtual | Returns the field at offset +0x9c from the object referenced by this+0x228. |
| `00cb5e30` | `GuildDataSource::GetCellValue ⚠downgrade` | 🟢 | virtual | Formats guild GUID, name, message of the day, and one permission field into the requested output; returns false if the referenced row is absent or the column is unsupported. |
| `00cb6120` | `GuildDataSource::Ctor` | ✅ | ctor | Initializes the BaseClient.Guilds holder, installs GuildDataSource vtables, and stores the two supplied source arguments. |
| `00cb6200` | `GuildDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the embedded GUI data holder and frees the object when the deleting flag's low bit is set. |
