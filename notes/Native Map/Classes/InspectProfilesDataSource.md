# Class: InspectProfilesDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbfeb0` | `InspectProfilesDataSource::GetColumnName` | ✅ | virtual | Maps profile selectors 0 through 4 to ID, rank, name, icon ID and badge-image ID, returning an error string for other values. |
| `00cc0160` | `InspectProfilesDataSource::GetRowCount` | ✅ | virtual | Returns the profile count stored at offset 0x184 in the object referenced at this+0x224. |
| `00cc01d0` | `InspectProfilesDataSource::GetValue ⚠downgrade` | 🟢 | virtual | Bounds-checks a profile row, returns its ID or rank for columns zero and one, then resolves the profile record and returns its name/icon fields for columns two through four. |
| `00cc0ef0` | `InspectProfilesDataSource::InspectProfilesDataSource` | ✅ | ctor | Registers BaseClient.Inspect.Jobs, installs the profile data-source vtables, and stores three supplied data pointers. |
| `00cc0fe0` | `InspectProfilesDataSource::ScalarDeletingDtor` | ✅ | dtor | Destroys the embedded GUI user-data holder and frees the object when the low bit of the flag argument is set; otherwise returns the object pointer. |
