# Class: ItemGroupItemsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c96b70` | `ItemGroupItemsDataSource::ColumnName` | ✅ | virtual | Maps column indices zero through four to GroupId, ItemId, ItemCount, IsEquipped, and IsMarketPlaceItem; other indices return <error>. |
| `00c96ce0` | `ItemGroupItemsDataSource::Rebuild ?` | 🟡 | method | Refreshes the source, finds the selected group in a ten-bucket map, requests each item definition, emits item ids, and marks the source built. |
| `00c96dc0` | `ItemGroupItemsDataSource::Refresh` | ✅ | virtual | Rebuilds the selected group's rows when the dirty byte is clear, then calls a second cleanup or refresh helper. |
| `00c96de0` | `ItemGroupItemsDataSource::SetGroupId` | 🟢 | method | Stores the supplied value at this+0x254 and rebuilds the data source. |
| `00c96e00` | `— ⚠reject` | ⚪ | virtual | Looks up a row key, returns one of its item-group fields by column index, performs membership checks for selected columns, and reports a missing row string when absent. |
| `00c97230` | `ItemGroupItemsDataSource::Ctor` | ✅ | ctor | Initializes the named source table, clears state, installs ItemGroupItemsDataSource vtables, stores the source argument, and copies its field at +0x20c. |
| `00c97320` | `ItemGroupItemsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the embedded data-source destructor, frees the instance when deleting flag bit one is set, and otherwise returns the original pointer. |
