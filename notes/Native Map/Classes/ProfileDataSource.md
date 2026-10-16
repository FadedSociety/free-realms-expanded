# Class: ProfileDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8f2b0` | `ProfileDataSource::ColumnName` | ✅ | virtual | Maps fields 0 through 15 to profile, rank, stars, icon set, membership and trial labels. |
| `00c8f390` | `ProfileDataSource::OnRowsChanged` | ✅ | virtual | If the source is not dirty, invokes a virtual callback through its adjusted owner, then calls FUN_010c8f30. |
| `00c8f3b0` | `ProfileDataSource::VirtualDispatchAdjustor` | ✅ | virtual | Loads the function pointer at vtable offset 0x10, clears byte this+0x258, and jumps through it. |
| `00c8f3c0` | `ProfileDataSource::ClearDirtyAndJump` | ✅ | virtual | Clears byte this+0x258 then jumps to 0x010caf20. |
| `00c8f830` | `ProfileDataSource::ReadColumn` | ✅ | virtual | Selects a profile row by key and formats one of its indexed fields into the caller's string output; missing string IDs are formatted as an error. |
| `00c90290` | `ProfileDataSource::RefreshRows` | ✅ | virtual | Clears cached rows, resolves profile strings and keys, rebuilds rows from the profile tables, and marks the source refreshed. |
| `00c90520` | `ProfileDataSource::Dtor` | ✅ | dtor | Restores ProfileDataSource vtables, clears the embedded ProfileSortingData list, and destroys the embedded GUI user-data table. |
| `00c90630` | `ProfileDataSource::Ctor` | ✅ | ctor | Initializes the GUI table for BaseClient.Profiles, installs ProfileDataSource vtables, stores the constructor argument, and zeros row/list state. |
| `00c906d0` | `ProfileDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls ProfileDataSource cleanup and frees the object when the deleting flag's low bit is set; otherwise returns the object pointer. |
| `00c906f0` | `— ⚠reject` | ⚪ | ctor | Initializes the BaseClient.Profiles binding and ProfileDataSource vtables, stores param_2 as the source argument, and zeros the embedded sort-list and refresh fields. |
