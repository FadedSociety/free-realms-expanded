# Class: CoinStoreDataSource::PlayerSell

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf0860` | `CoinStoreDataSource::PlayerSell::Ctor` | ✅ | ctor | Installs PlayerSell vtables at four object offsets and calls a member initialization helper. |
| `00cf08d0` | `CoinStoreDataSource::PlayerSell::CanSellItem ⚠downgrade` | 🟢 | virtual | Rejects invalid items and records in state 1; when a sale limit is active, requires positive available count before returning true. |
| `00cf0930` | `CoinStoreDataSource::PlayerSell::FieldName` | ✅ | virtual | Returns CanPreviewItem for index 0x1d and delegates every other index to the shared field-name helper. |
| `00cf0fd0` | `ThisAdjustToPlayerSellDtor` | ✅ | dtor | Subtracts 0x254 from ECX and jumps to 00cf1280; this is a this-adjusting thunk with no independent destructor body. |
| `00cf11f0` | `CoinStoreDataSource::PlayerSell::Ctor` | ✅ | ctor | Initializes the shared data-source base and installs PlayerSell vtables at four object offsets. |
| `00cf1280` | `CoinStoreDataSource::PlayerSell::ScalarDeletingDtor` | ✅ | dtor | Calls PlayerSell cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
| `00cf1d90` | `CoinStoreDataSource::PlayerSell::GetFieldValue` | ✅ | virtual | Returns one when row index is outside the active count; for field 4 computes available quantity and writes it, for field 0x1d writes an eligibility-dependent string, and delegates other fields to 00a6e5f0. |
