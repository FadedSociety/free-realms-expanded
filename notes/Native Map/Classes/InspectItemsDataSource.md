# Class: InspectItemsDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cbff10` | `InspectItemsDataSource::InspectItemsDataSource` | ✅ | ctor | Registers BaseClient.Inspect.Items, installs the class's primary and secondary vtables, stores two arguments, and calls FUN_008356b0 with zero. |
| `00cc0020` | `InspectItemsDataSource::GetColumnName` | ✅ | virtual | Returns IsInMarketplace for selector 0x1b; for other selectors tail-jumps to FUN_00cb09d0 with the original selector. |
| `00cc0080` | `InspectItemsDataSource::DestructorAdjustor` | ✅ | function | Subtracts 0x254 from ECX and tail-jumps to FUN_00cc0170, adjusting the secondary-base this pointer for the shared scalar deleting destructor. |
| `00cc0170` | `InspectItemsDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00cb1b10 to clean up inspect-item data, frees the object when the low flag bit is set, and otherwise returns the object pointer. |
| `00cc02d0` | `InspectItemsDataSource::GetValue` | ✅ | virtual | Resolves an item from the inspect source and item-definition tables, then returns requested identifier, strings, flags, prices and definition-derived values for supported column selectors. |
