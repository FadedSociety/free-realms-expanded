# Class: BrokerItemDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cee3a0` | `BrokerItemDataSource::FieldName` | ✅ | virtual | Returns field-name strings for indices 0x1d through 0x21, covering item GUID, listing price, remaining time, seller name, and listing status; delegates other indices. |
| `00ceeb70` | `BrokerItemDataSource::GetCurrentRecord ⚠downgrade` | 🟢 | virtual | If the active-state byte at offset 0x2a0 is set, invokes a virtual method through the adjusted object pointer; returns the dword at offset 0x2d0. |
| `00ceefe0` | `BrokerItemDataSource::Ctor` | ✅ | ctor | Initializes the base data source, installs BrokerItemDataSource vtables at four offsets, stores the broker pointer, and zeros its list and integer-array state. |
| `00cef0a0` | `ThisAdjustToBrokerItemDataSourceDtor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to 00cef210; this is an adjustor thunk and has no independent destructor body. |
| `00cef0b0` | `BrokerItemDataSource::ClearIndexArray` | 🟢 | method | Restores the embedded integer-array vtable, zeros its size, frees its allocation with alignment 4, clears the pointer, and invokes a member cleanup helper. |
| `00cef140` | `BrokerItemDataSource::RebuildVisibleIndices` | ✅ | virtual | Walks the linked item rows, keeps indices for rows accepted by CanDisplayRow, grows the integer array as needed, stores each row index, and clears the changed flag. |
| `00cef210` | `BrokerItemDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the index-array cleanup routine and frees the object when the deleting flag bit is set; otherwise returns the object pointer. |
| `00cef480` | `BrokerItemDataSource::GetField` | ✅ | virtual | Bounds-checks a row, resolves its item record, then switches across field IDs to emit names, values, display strings, item flags, prices, quantities, and related metadata. |
