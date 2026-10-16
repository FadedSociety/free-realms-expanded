# Class: ItemGroupsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c96be0` | `ItemGroupsDataSource::ColumnName` | ✅ | virtual | Maps indices zero through nine to item group fields including ID, Name, Description, SortOrder, Items, IconId, CategoryId, PageId, NumberOfItemsPlayerHas, and IsNew. |
| `00c96c80` | `ItemGroupsDataSource::Rebuild ?` | 🟡 | method | Refreshes the source, walks its group records, filters by the optional group id at +0x254, emits matching identifiers, and marks the source built. |
| `00c96d80` | `ItemGroupsDataSource::Refresh` | ✅ | virtual | Rebuilds rows when the dirty byte is clear, then calls a second cleanup or refresh helper and returns. |
| `00c96da0` | `ItemGroupsDataSource::SetGroupId ?` | 🟡 | method | Stores the supplied value at this+0x254, calls the rebuild routine, and returns. |
| `00c96fc0` | `— ⚠reject` | ⚪ | virtual | Looks up a group row and returns selected group fields, including names, description, and formatted item ids, or emits a missing-row message. |
| `00c97380` | `ItemGroupsDataSource::Ctor` | ✅ | ctor | Initializes the inherited data-source base, installs ItemGroupsDataSource vtables, clears source state, and stores two constructor arguments. |
| `00c97460` | `ItemGroupsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls a destructor helper, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |
