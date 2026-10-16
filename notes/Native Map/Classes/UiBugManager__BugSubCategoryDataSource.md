# Class: UiBugManager::BugSubCategoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f8980` | `UiBugManager::BugSubCategoryDataSource::ColumnName` | ✅ | virtual | Returns the constant label string Bug Sub Category. |
| `009f9490` | `UiBugManager::BugSubCategoryDataSource::GetValue` | ✅ | virtual | Reads the selected key at nested offset +0xb4, searches the ten-bucket table at nested +0x48, and returns the matching record's +0x10 field or zero. |
| `009f94e0` | `UiBugManager::BugSubCategoryDataSource::SetValue` | ✅ | virtual | Gets the selected row, resolves its key through FUN_009f9330, passes the resolved value and supplied value to a manager virtual setter, and returns 1. |
| `009fc3f0` | `UiBugManager::BugSubCategoryDataSource::Dtor` | ✅ | dtor | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc560` | `UiBugManager::BugSubCategoryDataSource::Ctor` | ✅ | ctor | Registers DesignTools.BugSubCategory, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc5e0` | `UiBugManager::BugSubCategoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the subcategory data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
