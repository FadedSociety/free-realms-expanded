# System: ui.merchant

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0a890` | `RegisterMerchantSellModule` |  | Registers the client 'MerchantSell' Lua module (Populate, SortItems, BuyFromMerchant, BuyFromUi, Close). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0a9a0` | `RegisterMerchantBuyModule` |  | Registers the client 'MerchantBuy' Lua module (Populate, SellToMerchant, SelectInventoryCategory). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0a950` | `MerchantBuy_SelectInventoryCategory` |  | Handler registered as 'SelectInventoryCategory' in the MerchantBuy module (entry table 0x0182a8dc, the module 00c0a9a0 registers): picks the inventory page shown in the merchant window. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ac10` | `MerchantSell_BuyFromUi` |  | Handler registered as 'BuyFromUi' in the module named 'MerchantSell' (entry table 0x0182a8ac): purchase path driven from the buy panel. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0aa30` | `MerchantSell_Populate ? ⚠downgrade` |  | Handler registered as 'Populate' in the MerchantSell module (0x0182a8ac): fills the sell window with the player's sellable items. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0aab0` | `— ⚠reject` |  | Handler registered as 'BuyFromMerchant' in the module named 'MerchantSell' (entry table 0x0182a8ac): the purchase entry that either uses an owned record or a merchant record. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ae60` | `— ⚠reject` |  | Handler registered as 'Close' in the module named 'MerchantSell' (entry table 0x0182a8ac): closes the merchant window and tells the server the interaction ended. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

