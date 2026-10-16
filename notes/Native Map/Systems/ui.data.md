# System: ui.data

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099f480` | `StoreBundleGroups::AdjustorThunk` | InGamePurchaseDataSource::StoreBundleGr… | Adjustor thunk for the StoreBundleGroups virtual method at slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f490` | `StorePortalCategories::AdjustorThunk` | InGamePurchaseDataSource::StorePortalCa… | Adjustor thunk for the StorePortalCategories virtual method at slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f4a0` | `StoreBillboards::AdjustorThunk` | InGamePurchaseDataSource::StoreBillboar… | Adjustor thunk for the StoreBillboards virtual method at slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f4b0` | `StoreBillboardPanels::AdjustorThunk` | InGamePurchaseDataSource::StoreBillboar… | Adjustor thunk for StoreBillboardPanels slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f4c0` | `GiftOrderHistory::AdjustorThunk` | InGamePurchaseDataSource::GiftOrderHist… | Adjustor thunk for GiftOrderHistory slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f4d0` | `CreditCardPayment::AdjustorThunk` | InGamePurchaseDataSource::CreditCardPay… | Adjustor thunk for CreditCardPayment slot 1. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a0690` | `ValidateMerchantSellDataSource` |  | Runtime validator for the MerchantSellBundles GUI data source. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099d440` | `PopulateRowsFromNestedChain → StoreBundleGroups::PopulateRows` |  | The adjustor thunk caller at 0099F480 identifies the StoreBundleGroups callback context; the rows come from an instance-reached chain, not a global list. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009b4440` | `FindActionBarDataSourceByHash` |  | Finds an ActionBarDataSource candidate by hash; equal-hash collisions are not disambiguated by this function. | [[Native Map/Shards/S0271 009b2210-009b7170|S0271]] |
| `00a7e490` | `SetField270AndRefresh` |  | Mechanical field setter and refresh call; caller 00a76520 invokes it in an ItemAbilities data-source path, but this function does not identify its owning class. | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## 🟡 speculative (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099d4c0` | `PopulatePortalCategoryRows ? → StorePortalCategories::PopulateRows` |  | Portal-category data-source callback inferred from its matching vtable wrapper at 0099F490. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099d510` | `PopulateBillboardRows ? → StoreBillboards::PopulateRows` |  | Billboard data-source callback inferred from its matching vtable wrapper at 0099F4A0. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099d600` | `PopulateGiftOrderHistoryRows ? → GiftOrderHistory::PopulateRows` |  | Gift-order-history row callback inferred from its vtable wrapper at 0099F4C0. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099d680` | `PopulateCreditCardPaymentRows ? → CreditCardPayment::PopulateRows` |  | Credit-card payment data-source callback inferred from the wrapper at 0099F4D0. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a09f0` | `AssignAndNotify ?` |  | Field assignment and callback dispatch; the callback meanings are not identified. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00a7c900` | `RefreshDataSourceAndClearDirty258 ?` |  | Refresh-and-clear helper of the GFC data-source family in this file, whose objects mark a changed byte at +0x258 (set by 00a7c9f0, cleared by the refresh helpers 00a7ca90/00a7cbd0). | [[Native Map/Shards/S0307 00a7b930-00a81420|S0307]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099d560` | `— ⚠reject` |  | Billboard-panel row callback inferred from the wrapper at 0099F4B0. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099efd0` | `— ⚠reject` |  | Writes param_2 to nested object field offset 0x4B7, invokes virtual slot 0x38, marks byte 0x4A6 dirty, and invokes virtual slot 0x10. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |

