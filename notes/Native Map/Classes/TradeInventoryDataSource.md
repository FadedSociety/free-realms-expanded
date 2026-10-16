# Class: TradeInventoryDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf5ce0` | `TradeInventoryDataSource::Dtor` | ✅ | dtor | Restores the TradeInventoryDataSource vtables at the object and base offsets, clears shared state DAT_01bf2070, then destroys its embedded member through 00cb1b10. |
| `00cf5d60` | `TradeInventoryDataSource::GetFieldName` | ✅ | virtual | Returns Available Count for field id 0x1b and No Trade Message for 0x1c; delegates other field ids to 00cb09d0. |
| `00cf6170` | `TradeInventoryDataSource::AdjustThisAndJump_00cf61c0` | ✅ | function | The 11-byte body subtracts 0x254 from ECX and jumps to 00cf61c0; this is a this-adjusting jump thunk to the scalar deleting destructor. |
| `00cf61c0` | `TradeInventoryDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the TradeInventoryDataSource destructor, frees this when the low bit of the flag is set, and returns the object pointer otherwise. |
| `00cf61e0` | `TradeInventoryDataSource::UpdateOtherCoin` | 🟢 | virtual | When the update flag is false, sets trade state bits to value 2 and writes the supplied coin count at the trade object+0x50 when changed, calls update helper 00821dc0, then logs TradeHandler:OnUpdateOtherCoin. |
| `00cf6b10` | `— ⚠reject` | ⚪ | virtual | Gets the active player identifier, checks it against the datasource filter, then returns whether the supplied item range has a positive count. |
| `00cf6ed0` | `TradeInventoryDataSource::TradeInventoryDataSource` | ✅ | ctor | Initializes the datasource base with BaseClient.Trade.Inventory, installs four TradeInventoryDataSource vtable pointers, and stores its constructor arguments. |
| `00cf76e0` | `TradeInventoryDataSource::GetCell` | ✅ | virtual | Handles trade inventory columns, returning counts and item values or formatting ItemNoTrade, ItemEquipped, and ItemEquippedOnPet warnings. |
| `00cf7860` | `TradeInventoryDataSource::Initialize` | ✅ | method | Clears trade state, creates partner and local item datasources plus an inventory datasource, then registers them under BaseClient.Trade.PartnerItems, MyItems, and Inventory. |
