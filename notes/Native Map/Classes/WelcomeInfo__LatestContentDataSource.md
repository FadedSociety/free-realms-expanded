# Class: WelcomeInfo::LatestContentDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ace560` | `GetLatestContentColumnName` | ✅ | virtual | Returns Title for column 0, Description for column 1, and <error> otherwise. |
| `00ace940` | `GetField230` | ✅ | virtual | Returns the dword stored at this+0x230. |
| `00acf890` | `GetLatestContentCellValue` | ✅ | virtual | Reads the selected 8-byte row field, asks the string interface to resolve it, and on lookup failure may format <STRING %d> into the output. |
| `00ad0e60` | `WelcomeInfo::LatestContentDataSource::Ctor` | ✅ | ctor | Installs LatestContentDataSource and entry-array vtables, stores the owner pointer, and zeros the array count, capacity, and pointer. |
| `00ad0ef0` | `WelcomeInfo::LatestContentDataSource::Dtor` | ✅ | dtor | Clears the element count, restores the entry-array vtable, frees array storage, zeros the pointer, and calls the embedded base destructor. |
| `00ad0f80` | `WelcomeInfo::LatestContentDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
