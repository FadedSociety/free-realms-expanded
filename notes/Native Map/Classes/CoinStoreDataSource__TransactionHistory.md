# Class: CoinStoreDataSource::TransactionHistory

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf0a80` | `CoinStoreDataSource::TransactionHistory::FieldName` | ✅ | virtual | Maps field indices 0 through 8 to id, type, merchant GUID, item ID, quantity, tint ID, price, bought-back flag, and quantity remaining. |
| `00cf1f70` | `— ⚠reject` | ⚪ | virtual | Checks the backing store and selected row, resolves a transaction object, then switches on field id 0 through 8 to copy or release field values; returns zero when prerequisites fail and one for handled/default branches. |
| `00cf2250` | `CoinStoreDataSource::TransactionHistory::Dtor` | ✅ | dtor | Writes the TransactionHistory vtable at the base and embedded offsets, then destroys the embedded GUI datasource base through 0091bf60. |
| `00cf22c0` | `CoinStoreDataSource::TransactionHistory::GetRowCount` | ✅ | virtual | Returns the 32-bit value at this+0x22c. |
| `00cf2390` | `— ⚠reject` | ⚪ | ctor | Constructs the GUI datasource base, installs TransactionHistory vtables, stores its selected datasource pointer, sets the row mode to 2, clears the count, and sets an enabled byte. |
| `00cf2420` | `CoinStoreDataSource::TransactionHistory::ScalarDeletingDtor` | ✅ | dtor | Calls the TransactionHistory destructor body, frees the object when the low bit of the flag is set, and otherwise returns the object pointer. |
