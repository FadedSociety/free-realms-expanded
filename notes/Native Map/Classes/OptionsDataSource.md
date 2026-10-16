# Class: OptionsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c98db0` | `OptionsDataSource::ColumnName` | ✅ | virtual | Maps indices zero through twelve to option table fields such as Section, DisplayName, Tooltip, Value, OptionName, OptionType, Callback, Disabled, and DefaultValue. |
| `00c98e90` | `OptionsDataSource::GetRowId` | ✅ | virtual | Loads the row object at this+0x228, reads its field at +8, and returns that value. |
| `00c99250` | `OptionsDataSource::GetFieldValue` | ✅ | virtual | Selects an option row and column, reads typed string, numeric, boolean, and callback values, formats output, and reports missing options. |
| `00c998a0` | `OptionsDataSource::Dtor` | ✅ | dtor | Restores OptionsDataSource vtable pointers and calls the shared embedded GUI data table destructor. |
| `00c99920` | `OptionsDataSource::Ctor` | ✅ | ctor | Initializes the options table and its vtables, then stores the row data and related source pointers. |
| `00c999a0` | `OptionsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the options data source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the pointer. |
