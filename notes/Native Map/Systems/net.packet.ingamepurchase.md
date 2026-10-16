# System: net.packet.ingamepurchase

21 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1fe70` | `InGamePurchaseUpdateMerchantSellVftableRestore` | InGamePurchaseUpdateMerchantSellBundleQ… | Exception-unwind cleanup / base destructor for the merchant-sell quantity packet class; it converts the partially built object back to BasePacket. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b2a320` | `BuyBundleFromMerchantRequest::Dtor` | InGamePurchaseBuyBundleFromMerchantRequ… | Cleans up merchant bundle purchase request fields. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2a440` | `BuyBundleFromMerchantResponse::Dtor` | InGamePurchaseBuyBundleFromMerchantResp… | Cleans up merchant bundle purchase response fields. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2a860` | `PlaceOrderClientTicket::Dtor` | PacketInGamePurchasePlaceOrderClientTic… | Cleans up the client-ticket packet's dynamic payload. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2a920` | `PlaceOrderClientTicket::ScalarDeletingDtor` | PacketInGamePurchasePlaceOrderClientTic… | MSVC scalar deleting destructor for the client-ticket packet. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c120` | `StoreBundleContentResponse::Constructor` | PacketInGamePurchaseStoreBundleContentR… | Constructs a store-bundle content response packet object. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c180` | `StoreBundleContentResponse::ScalarDeletingDtor` | PacketInGamePurchaseStoreBundleContentR… | MSVC scalar deleting destructor for store-bundle content response. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c1a0` | `UpdateSaleDisplay::Constructor` | InGamePurchaseUpdateSaleDisplay | Constructs a sale-display update packet. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c210` | `UpdateSaleDisplay::Dtor` | InGamePurchaseUpdateSaleDisplay | Cleans up sale-display update contents. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c280` | `UpdateSaleDisplay::ScalarDeletingDtor` | InGamePurchaseUpdateSaleDisplay | MSVC scalar deleting destructor for sale-display update. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c2a0` | `BuyBundleFromMerchantRequest::ScalarDeletingDtor` | InGamePurchaseBuyBundleFromMerchantRequ… | MSVC scalar deleting destructor for merchant bundle purchase request. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c2c0` | `BuyBundleFromMerchantResponse::ScalarDeletingDtor` | InGamePurchaseBuyBundleFromMerchantResp… | MSVC scalar deleting destructor for merchant bundle purchase response. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c610` | `PreviewOrder::Constructor` | PacketInGamePurchasePreviewOrder<GameCo… | Constructs a preview-order packet object. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c680` | `PreviewOrder::ScalarDeletingDtor` | PacketInGamePurchasePreviewOrder<GameCo… | MSVC scalar deleting destructor for preview-order packet. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2c810` | `UpdateItemRequirementsReplyDataList::Dtor` | SoeUtil::List<InGamePurchaseUpdateItemR… | Cleans up item-requirement reply records. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2cba0` | `UpdateItemRequirementsReplyDataList::ScalarDeletingDtor` | SoeUtil::List<InGamePurchaseUpdateItemR… | MSVC scalar deleting destructor for item-requirement reply data list. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b2a7f0` | `PlaceOrderClientTicket::Constructor ⚠downgrade` | PacketInGamePurchasePlaceOrderClientTic… | Constructs a place-order client-ticket packet object. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b2a230` | `ResponseData::Dtor ? ⚠rename` | PacketInGamePurchaseStoreBundleContentR… | Cleans up response payload storage before base-packet teardown. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1fdf0` | `— ⚠reject` | PacketInGamePurchaseGiftNotification | Derived destructor body for PacketInGamePurchaseGiftNotification; releases its array and string members before restoring the BasePacket base vtable. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b2a290` | `— ⚠reject` | InGamePurchaseBuyBundleFromMerchantRequ… | Constructs a merchant bundle purchase request object; subtype 43 is absent from the packet catalogue. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |
| `00b2a3b0` | `— ⚠reject` | InGamePurchaseBuyBundleFromMerchantResp… | Constructs a merchant bundle purchase response packet object. | [[Native Map/Shards/S0339 00b29890-00b2eaf0|S0339]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b2c120` | `StoreBundleContentResponse::Constructor` | recv | PacketInGamePurchaseStoreBundleContentResponse |  |  |
| `00b2c1a0` | `UpdateSaleDisplay::Constructor` | recv | InGamePurchaseUpdateSaleDisplay |  |  |
| `00b2c610` | `PreviewOrder::Constructor` | send | PacketInGamePurchasePreviewOrder |  |  |
| `00b2a7f0` | `PlaceOrderClientTicket::Constructor ⚠downgrade` | send |  | 0x24 |  |
| `00b2a290` | `— ⚠reject` | send |  | 0x2b |  |
| `00b2a3b0` | `— ⚠reject` | recv |  | 0x2c |  |
