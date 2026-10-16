# System: net.packet.broker

41 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0c6e0` | `BaseBrokerPacket_DtorBody` | BaseBrokerPacket | Common teardown of the broker packet family objects. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c7a0` | `BaseBrokerPacket_DtorBody_00c0c7a0` | BaseBrokerPacket | Common broker teardown used by the Place send path. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c7f0` | `BaseBrokerPacket_DtorBody_00c0c7f0 → BrokerReListItemPacket_DtorBody_00c0c7f0` | BaseBrokerPacket | Common broker teardown used by the Relist send path. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c840` | `BaseBrokerPacket_DtorBody_00c0c840` | BaseBrokerPacket | Common broker teardown used by the Buy send path. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c9a0` | `BrokerSearchPacket_ScalarDeletingDtor` | BrokerSearchPacket | MSVC scalar deleting destructor for the broker search packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ca10` | `BrokerPlaceItemPacket_ScalarDeletingDtor` | BrokerPlaceItemPacket | MSVC scalar deleting destructor for the broker place-item packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cb00` | `BrokerReListItemPacket_ScalarDeletingDtor` | BrokerReListItemPacket | MSVC scalar deleting destructor for the broker relist packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cbe0` | `BrokerBuyItemPacket_ScalarDeletingDtor` | BrokerBuyItemPacket | MSVC scalar deleting destructor for the broker buy packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ccc0` | `BrokerCancelItemPacket_ScalarDeletingDtor` | BrokerCancelItemPacket | MSVC scalar deleting destructor for the broker cancel packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cda0` | `BrokerRequestItemSaleCoinPacket_ScalarDeletingDtor` | BrokerRequestItemSaleCoinPacket | MSVC scalar deleting destructor for the sale-coin request packet. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cee280` | `BaseBrokerPacketDtor` | BaseBrokerPacket | Base-class destructor used during broker packet cleanup. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee2d0` | `BrokerMyItemsPacket::Ctor` | BrokerMyItemsPacket | Constructs the broker my-items packet object; this body does not show sending or parsing. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee340` | `BaseBrokerPacketDtor` | BaseBrokerPacket | Base-class destructor used in broker packet exception cleanup. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceea10` | `BrokerSearchItemsPacket::Ctor` | BrokerSearchItemsPacket | Constructs a broker search-items packet object; direction is not visible here. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceea90` | `BrokerSearchItemsPacket::ScalarDeletingDtor` | BrokerSearchItemsPacket | Scalar deleting destructor for BrokerSearchItemsPacket. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceeb00` | `BrokerMyItemsPacket::ScalarDeletingDtor` | BrokerMyItemsPacket | Scalar deleting destructor for BrokerMyItemsPacket. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟢 likely (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0c890` | `BaseBrokerPacket_DtorBody_00c0c890 ⚠downgrade` | BaseBrokerPacket | Common broker teardown emitted for another broker send path (the owning code is outside this shard). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c8e0` | `BaseBrokerPacket_DtorBody_00c0c8e0 ⚠downgrade` | BaseBrokerPacket | Common broker teardown emitted for another broker send path (the owning code is outside this shard). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0ca80` | `BrokerReListItemPacket_Ctor ⚠downgrade` | BrokerReListItemPacket | Builds the broker relist request sent by the registered Broker.Relist handler 00c0e000 (new price/duration for an existing listing). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cef0` | `BrokerPacket_WriteCommonHeader` |  | Shared broker packet preamble used by the cancel, sale-coin and place/relist/search serialisers (all six broker body writers call it first). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cfa0` | `BrokerPlaceItemPacket_SerializeBody` |  | Serialises the BrokerPlaceItemPacket body for the ClientGateway template 00c0d590 (the Broker.Place send path). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d140` | `BrokerBuyItemPacket_SerializeBody` |  | Serialises the BrokerBuyItemPacket body for the ClientGateway template 00c0d8e0 (the Broker.Buy send path). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d1a0` | `BrokerReListItemPacket_SerializeBody` |  | Serialises the BrokerReListItemPacket body for the ClientGateway template 00c0d9f0 (the Broker.Relist send path). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d590` | `SendBrokerPlaceItemPacket_Gateway` |  | Sends BrokerPlaceItemPacket built by the registered Broker.Place handler 00c0dcd0. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d6a0` | `SendBrokerCancelItemPacket_Gateway` |  | Sends BrokerCancelItemPacket built by the registered Broker.Cancel handler 00c0ddd0. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d7c0` | `SendBrokerRequestItemSaleCoinPacket_Gateway` |  | Sends BrokerRequestItemSaleCoinPacket built by the helper 00c0de80 (called from the Broker.Buy handler 00c0df30) - the two packet classes share the same body shape. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d8e0` | `SendBrokerBuyItemPacket_Gateway` |  | Sends BrokerBuyItemPacket built by the registered Broker.Buy handler 00c0df30. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d9f0` | `SendBrokerReListItemPacket_Gateway` |  | Sends BrokerReListItemPacket built by the registered Broker.Relist handler 00c0e000. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0db00` | `SendBrokerSearchPacket_Gateway` |  | Sends BrokerSearchPacket built by the registered Broker.Search handler 00c0e110. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cefdf0` | `ReadBrokerItemPairAndNextRow` |  | Row deserializer helper used by broker packet parsing. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0180` | `ReadBrokerItemPairAndNextRow` |  | Helper for broker payload row parsing. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf02a0` | `ReadBrokerItemPacketRange` |  | Bounded packet-reader helper called by the broker my-items parser. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0340` | `ParseBrokerSearchResults` |  | Receive-side broker search-results handler identified by the central broker subtype dispatcher. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0590` | `ParseBrokerMyItems` |  | Receive-side my-items handler identified by the central broker subtype dispatcher. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0c670` | `— ⚠reject` | BrokerSearchPacket | Builds the broker (consignment) search query sent by the registered Broker.Search handler 00c0e110. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0c730` | `— ⚠reject` | BrokerPlaceItemPacket | Builds the broker item-listing request sent by the registered Broker.Place handler 00c0dcd0. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cb70` | `— ⚠reject` | BrokerBuyItemPacket | Builds the broker purchase request sent by the registered Broker.Buy handler 00c0df30. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cc50` | `— ⚠reject` | BrokerCancelItemPacket | Builds the broker cancel-listing request sent by the registered Broker.Cancel handler 00c0ddd0. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0cd30` | `— ⚠reject` | BrokerRequestItemSaleCoinPacket | Builds the broker request that asks what an item would sell for (built by the helper 00c0de80, which the Broker.Buy handler 00c0df30 calls before the purchase). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c0d2c0` | `— ⚠reject` |  | Serialises the BrokerSearchPacket body for the ClientGateway template 00c0db00 (the Broker.Search send path). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cefcf0` | `— ⚠reject` |  | Parses broker row payload fields; called by broker packet subtype dispatch. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c0ca80` | `BrokerReListItemPacket_Ctor ⚠downgrade` | send |  | 4 | Hypothesis: the original server updated the price/count of an existing consignment listing; the inference depends on the sender path 00c0e000 -> 00c0d9f0 and the body writer 00c0d1a0. No broker handler exists in any recovered server. |
| `00c0c670` | `— ⚠reject` | send |  | 2 | Hypothesis: the original server ran a consignment-store search with the five filter fields (item/class/level/price bounds, -1 = unset) and returned matching listings; the inference depends on the sender path 00c0e110 -> 00c0db00 and the body writer 00c0d2c0. No broker handler exists in any recovered server and no broker frame was captured. |
| `00c0c730` | `— ⚠reject` | send |  | 3 | Hypothesis: the original server placed the selected item on the consignment broker for the given price (the four zeroed fields plus the embedded guid carry item id, count, price and duration); the inference depends on the sender path 00c0dcd0 -> 00c0d590 and the body writer 00c0cfa0. No broker handler exists in any recovered server. |
| `00c0cb70` | `— ⚠reject` | send |  | 5 | Hypothesis: the original server transferred the listed item from the seller to the buyer, moved coins in both accounts (minus fees) and replied with the updated listing/wallet; the inference depends on the sender path 00c0df30 -> 00c0d8e0 and the body writer 00c0d140. No broker handler exists in any recovered server. |
| `00c0cc50` | `— ⚠reject` | send |  | 6 | Hypothesis: the original server removed the consignment listing and returned the item; the inference depends on the sender path 00c0ddd0 -> 00c0d6a0 and the shared body writer 00c0cef0. No broker handler exists in any recovered server. |
| `00c0cd30` | `— ⚠reject` | send |  | 7 | Hypothesis: the original server answered the consignment price query for an item so the buy dialog could show the cost before confirmation; the inference depends on the caller chain 00c0df30 -> 00c0de80 -> 00c0d7c0 and the shared body writer 00c0cef0. No broker handler exists in any recovered server. |
