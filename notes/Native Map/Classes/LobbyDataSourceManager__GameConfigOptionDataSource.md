# Class: LobbyDataSourceManager::GameConfigOptionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c91120` | `GameConfigOptionDataSource::ColumnName` | ✅ | virtual | Returns the literal column names OptionValueId, IsDefaultValue, and OptionLabelStringId for indices one through three; index zero reads a localized option label. |
| `00c911c0` | `GameConfigOptionDataSource::CountValues` | ✅ | virtual | Builds a temporary integer list from the configured option mapping and returns the count stored in its resulting list object. |
| `00c91310` | `GameConfigOptionDataSource::GetColumnValue` | ✅ | virtual | Returns the localized option label, numeric option value, default-value flag, or formatted option string according to the requested column index. |
| `00c914b0` | `GameConfigOptionDataSource::Dtor` | ✅ | dtor | Restores class and string vtables, releases two owned strings, and destroys the embedded GUI user-data table. |
| `00c91580` | `GameConfigOptionDataSource::Ctor` | ✅ | ctor | Initializes the base GUI table, class vtables, two empty owned strings, option data-source fields, and the supplied option configuration values. |
| `00c91650` | `GameConfigOptionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the class cleanup routine and frees the object when the deleting flag's low bit is set; otherwise returns the original pointer. |
