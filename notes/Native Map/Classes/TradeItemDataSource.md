# Class: TradeItemDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf5ca0` | `TradeItemDataSource::LogUpdatePlayerOrOther` | ✅ | virtual | Checks the byte at this+0x2e8 and logs either TradeHandler:OnUpdatePlayer or TradeHandler:OnUpdateOther. |
| `00cf5cd0` | `TradeItemDataSource::ClearFlag_2d0` | ✅ | virtual | Clears the byte at this+0x2d0. |
| `00cf6180` | `TradeItemDataSource::GetField_2cc` | ✅ | virtual | Returns the 32-bit field at this+0x2cc. |
| `00cf70d0` | `TradeItemDataSource::TradeItemDataSource ?` | 🟡 | ctor | Initializes its datasource base, installs four TradeItemDataSource vtable pointers, stores a flag, and initializes the embedded item list. |
| `00cf7170` | `TradeItemDataSource::DestructorAdjustor ?` | 🟡 | virtual | Subtracts 0x254 from the this pointer, then jumps to 00cf71e0; it is an adjustor thunk, not the destructor body. |
| `00cf7180` | `TradeItemDataSource::Cleanup ?` | 🟡 | dtor | Cleans the embedded item list through FUN_00cf6e60, then destroys the datasource base state. |
| `00cf71e0` | `TradeItemDataSource::ScalarDeletingDtor ?` | 🟡 | dtor | Calls the class cleanup and frees the object when the delete flag's low bit is set. |
| `00cf7200` | `TradeItemDataSource::GetCell` | ✅ | virtual | Resolves a displayed trade item and dispatches by column index to write its ID, count, item metadata, flags, or related values into the UI cell. |
