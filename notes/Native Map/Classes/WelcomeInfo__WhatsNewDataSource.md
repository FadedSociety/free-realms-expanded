# Class: WelcomeInfo::WhatsNewDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ace460` | `GetWhatsNewColumnName` | ✅ | virtual | Returns Subject for column 0, Body for column 1, and <error> for other indices. |
| `00ace920` | `GetField230` | ✅ | virtual | Returns the dword stored at this+0x230. |
| `00acf280` | `— ⚠reject` | ⚪ | virtual | Selects row index times 0x20; for field 0 or 1 it clears the wide string at that row offset and returns 1, otherwise returns 0. |
| `00ad0a70` | `WelcomeInfo::WhatsNewDataSource::Ctor` | ✅ | ctor | Installs the WhatsNewDataSource and entry-array vtables, stores the owner pointer, zeros array state, and parses the local welcome resource. |
| `00ad0b10` | `WelcomeInfo::WhatsNewDataSource::Dtor` | ✅ | dtor | Restores the entry-array vtable, destroys its wide-string rows, frees array storage, clears the pointer, and calls the embedded base destructor. |
| `00ad0ba0` | `WelcomeInfo::WhatsNewDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
