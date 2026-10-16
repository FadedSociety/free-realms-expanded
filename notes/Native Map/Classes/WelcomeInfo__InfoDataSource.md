# Class: WelcomeInfo::InfoDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ace4a0` | `GetInfoColumnName` | ✅ | virtual | Returns Treasure Ticket Count for column 0, Did You Know for column 1, and <error> otherwise. |
| `00acfc20` | `GetInfoCellValue` | ✅ | virtual | For one requested column, returns the localized count for key 0x17a or resolves a row string and formats its identifier as <STRING %d> when unresolved. |
| `00ad0bc0` | `WelcomeInfo::InfoDataSource::Ctor` | ✅ | ctor | Installs InfoDataSource and integer-array vtables, stores the owner, initializes array fields and selection to -1, then loads WelcomeTips.txt. |
| `00ad0c70` | `WelcomeInfo::InfoDataSource::Dtor` | ✅ | dtor | Restores the integer-array vtable, clears its count, frees array storage, zeros the pointer, and calls the embedded base destructor. |
| `00ad0d00` | `WelcomeInfo::InfoDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the InfoDataSource destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
