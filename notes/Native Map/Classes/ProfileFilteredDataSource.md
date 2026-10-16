# Class: ProfileFilteredDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8ff30` | `ProfileFilteredDataSource::RefreshRows` | ✅ | virtual | Clears its cached linked rows, resolves profile strings and record keys, adds qualifying records, then marks the data source refreshed. |
| `00c905d0` | `ProfileFilteredDataSource::Dtor` | ✅ | dtor | Restores ProfileFilteredDataSource vtables and delegates base cleanup to FUN_00c90520. |
| `00c90790` | `ProfileFilteredDataSource::Ctor` | ✅ | ctor | Constructs the BaseClient.Profiles.Filtered base profile source, then installs ProfileFilteredDataSource vtables. |
| `00c90810` | `ProfileFilteredDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls derived and base cleanup, then frees the object when the deleting flag's low bit is set. |
