# Class: UiBugManager::BugCategoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f8950` | `UiBugManager::BugCategoryDataSource::ColumnName` | ✅ | virtual | Returns the constant label string Bug Category. |
| `009f93f0` | `UiBugManager::BugCategoryDataSource::GetValue` | ✅ | virtual | Returns the dword at offset +0x44 of the object referenced by this+0x224. |
| `009f9400` | `UiBugManager::BugCategoryDataSource::SetValue` | ✅ | virtual | Gets the selected row through a virtual call, resolves its category key with FUN_009f92c0, passes the result and supplied value to a manager virtual setter, and returns 1. |
| `009fc390` | `UiBugManager::BugCategoryDataSource::Dtor` | ✅ | dtor | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc4c0` | `UiBugManager::BugCategoryDataSource::Ctor` | ✅ | ctor | Registers DesignTools.BugCategory, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc540` | `UiBugManager::BugCategoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the category data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
