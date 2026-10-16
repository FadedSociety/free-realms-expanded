# System: ui.trade

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c05ce0` | `RegisterTradeModule` |  | Registers the client Trade window API (RejectInvite, Accept, Cancel, Lock, CancelLock, OfferItems, OfferCoins, GetPartnerGuid, GetPartnerName, GetPartnerCoinOffer, SelectInventoryCategory) that ScriptsBase.lua calls. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cf5ca0` | `TradeItemDataSource::LogUpdatePlayerOrOther` | TradeItemDataSource | Logs which side of the trade item datasource was updated. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5cd0` | `TradeItemDataSource::ClearFlag_2d0` | TradeItemDataSource | Resets a trade item datasource flag. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5ce0` | `TradeInventoryDataSource::Dtor` | TradeInventoryDataSource | Destroys the trade inventory datasource and its embedded member. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5d60` | `TradeInventoryDataSource::GetFieldName` | TradeInventoryDataSource | Supplies field names for the trade inventory datasource. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf6170` | `TradeInventoryDataSource::AdjustThisAndJump_00cf61c0` | TradeInventoryDataSource | Provides an adjusted-base destructor entry for TradeInventoryDataSource. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf6180` | `TradeItemDataSource::GetField_2cc` | TradeItemDataSource | Exposes one trade item datasource field through its virtual interface. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf61c0` | `TradeInventoryDataSource::ScalarDeletingDtor` | TradeInventoryDataSource | Implements the MSVC scalar deleting destructor for this datasource. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf6ed0` | `TradeInventoryDataSource::TradeInventoryDataSource` | TradeInventoryDataSource | Constructs the inventory datasource used by the trade UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7200` | `TradeItemDataSource::GetCell` | TradeItemDataSource | Supplies row cell values to the trade item UI datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf76e0` | `TradeInventoryDataSource::GetCell` | TradeInventoryDataSource | Supplies inventory row values and restriction messages to the trade UI. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7860` | `TradeInventoryDataSource::Initialize` | TradeInventoryDataSource | Builds the three trade UI datasources when the trade view opens. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c06020` | `Trade_GetPartnerGuid` |  | Handler registered as 'GetPartnerGuid' in the Trade module (0x01829c48): exposes the partner's 64-bit guid to Lua as text. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cf61e0` | `TradeInventoryDataSource::UpdateOtherCoin` | TradeInventoryDataSource | Updates the displayed other-player coin amount in the trade state. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cf7050` | `AllocateTradeDatasourceRow ?` |  | Allocates and registers a trade datasource row helper, based on its caller. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf70d0` | `TradeItemDataSource::TradeItemDataSource ?` | TradeItemDataSource | Constructs one side's trade item datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7170` | `TradeItemDataSource::DestructorAdjustor ?` | TradeItemDataSource | Adjusts the TradeItemDataSource subobject pointer before forwarding to its deleting destructor. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf7180` | `TradeItemDataSource::Cleanup ?` | TradeItemDataSource | Releases the trade item datasource contents. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf71e0` | `TradeItemDataSource::ScalarDeletingDtor ?` | TradeItemDataSource | Scalar deleting destructor for TradeItemDataSource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c05e20` | `— ⚠reject` |  | Handler registered as 'OfferCoins' in the Trade module (entry table 0x01829c48): hands the player's coin offer to the native trade window. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05e90` | `— ⚠reject` |  | Handler registered as 'SelectInventoryCategory' in the Trade module (0x01829c48): selects which inventory page the trade window shows. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05f00` | `— ⚠reject` |  | Handler registered as 'GetPartnerName' in the Trade module (0x01829c48). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cf6b10` | `— ⚠reject` | TradeInventoryDataSource | Filters an item row for display in the trade inventory datasource. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf6f80` | `— ⚠reject` |  | Copies selected trade UI text; caller is a trade selection callback. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

