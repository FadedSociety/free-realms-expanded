# Class: GameCommerce::MarketingDataSourceFlatFileLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d66980` | `MarketingDataSourceFlatFileLoader::SetLoadResult ? ⚠rename` | 🟡 | virtual | Passes the stored source pointer and state byte to helper 00d776e0, then clears the completion byte at offset 0x233. |
| `00d66a50` | `— ⚠reject` | ⚪ | virtual | Releases a shared path string when referenced, resets its string and loader fields, and clears the active source pointer and completion flag. |
| `00d66ac0` | `MarketingDataSourceFlatFileLoader::Ctor ?` | 🟡 | ctor | Initializes loader and string members, stores input flags, assigns the path, and appends a slash when the path lacks a trailing slash or backslash. |
| `00d66ba0` | `MarketingDataSourceFlatFileLoader::GetName ?` | 🟡 | virtual | Returns the static string FlatFileLoader. |
| `00d66bb0` | `MarketingDataSourceFlatFileLoader::GetCompleteFlag ?` | 🟡 | virtual | Returns the byte stored at loader offset 0x233. |
| `00d66bc0` | `MarketingDataSourceFlatFileLoader::Dtor ?` | 🟡 | dtor | Restores loader vtable, releases shared path and directory strings, resets loader state, releases the string base, and calls helper 00d77700. |
| `00d66cd0` | `MarketingDataSourceFlatFileLoader::CanLoad ? ⚠rename` | 🟡 | virtual | When completion byte is zero, reallocates the input buffer to 0x100001 if current length is smaller, then returns one; otherwise returns zero. |
| `00d66d70` | `MarketingDataSourceFlatFileLoader::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the flat-file loader destructor body and frees the object when the deleting flag is set. |
| `00d671d0` | `MarketingDataSourceFlatFileLoader::LoadAllRecordSets ?` | 🟡 | function | Checks loader input and helper status, iterates the marketing source record list, calls the per-record file loader for each node, and returns aggregate success. |
| `00d67220` | `MarketingDataSourceFlatFileLoader::Load ?` | 🟡 | virtual | Sets completion flag, stores the source pointer, invokes LoadAllRecordSets, and stores its boolean result at offset 0x232. |
