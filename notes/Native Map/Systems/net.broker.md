# System: net.broker

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bee5f0` | `BaseBrokerPacket::CtorWithSubtype` | BaseBrokerPacket | Initializes a broker packet header with a subtype discriminator. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee660` | `BaseBrokerPacket::Dtor` | BaseBrokerPacket | Performs base packet teardown for broker packet objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee6b0` | `BaseBrokerPacket::ScalarDeletingDtor` | BaseBrokerPacket | Implements scalar deleting destruction for base broker packets. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee720` | `BaseAdminBrokerPacket::CtorWithSubtype` | BaseAdminBrokerPacket | Initializes an administrative broker packet header. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee790` | `BaseAdminBrokerPacket::Dtor` | BaseAdminBrokerPacket | Performs base teardown for administrative broker packet objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee7e0` | `BaseAdminBrokerPacket::ScalarDeletingDtor` | BaseAdminBrokerPacket | Implements scalar deleting destruction for administrative broker packets. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee8c0` | `BaseBrokerPacket::DtorException` | BaseBrokerPacket | Cleans up a broker packet base subobject after construction failure. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee910` | `BrokerMyItemAddedPacket::Ctor` | BrokerMyItemAddedPacket | Initializes the broker packet indicating an item was added to the player's list. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee970` | `BaseBrokerPacket::DtorException` | BaseBrokerPacket | Cleans up a broker packet base subobject after construction failure. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bee9c0` | `BaseBrokerPacket::DtorException` | BaseBrokerPacket | Cleans up a broker packet base subobject after construction failure. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beeac0` | `BrokerInformationPacket::ScalarDeletingDtor` | BrokerInformationPacket | Implements scalar deleting destruction for broker information packet objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beeb30` | `BrokerMyItemAddedPacket::ScalarDeletingDtor` | BrokerMyItemAddedPacket | Implements scalar deleting destruction for the item-added packet object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beec10` | `BrokerRemoveItemFromListsPacket::ScalarDeletingDtor` | BrokerRemoveItemFromListsPacket | Implements scalar deleting destruction for this broker packet. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beee40` | `AdminBrokerServerVersionPacket::Dtor` | AdminBrokerServerVersionPacket | Destroys server version text storage and the packet base subobjects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beef40` | `BrokerErrorPacket::Dtor` | BrokerErrorPacket | Destroys the broker error text and packet base subobjects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef070` | `AdminBrokerServerVersionPacket::ScalarDeletingDtor` | AdminBrokerServerVersionPacket | Implements scalar deleting destruction for server-version packet objects. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef090` | `BrokerErrorPacket::ScalarDeletingDtor` | BrokerErrorPacket | Implements scalar deleting destruction for broker error packets. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef790` | `BrokerErrorPacket_Parse` |  | Handles broker error payloads and looks up the referenced broker string. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00befcf0` | `BaseBrokerPacket_Dispatch` |  | Routes incoming broker-family payloads to subtype-specific client handlers. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bee850` | `BrokerInformationPacket::Ctor ⚠downgrade` | BrokerInformationPacket | Initializes a broker information response packet object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beeba0` | `BrokerRemoveItemFromListsPacket::Ctor ⚠downgrade` | BrokerRemoveItemFromListsPacket | Initializes the packet that removes an item from broker lists. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beece0` | `BrokerProxyCtor` |  | Creates a broker proxy wrapper for the supplied client-side object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef5c0` | `BrokerItemPacket_ReadFields` |  | Decodes a broker item record from a bounded packet reader. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef900` | `BrokerInformationPacket_Parse` |  | Handles a broker information response payload. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00bef9f0` | `BrokerRemoveItemFromListsPacket_Parse` |  | Handles removal of a broker item from client-side lists. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00befb40` | `ReadBrokerItemPacket` |  | Validates a broker item payload before applying it. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00befbc0` | `BrokerMyItemAddedPacket_Parse` |  | Applies a broker packet that adds an item to the player's own broker list. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bee590` | `— ⚠reject` | BaseBrokerPacket | Initializes the common broker packet header. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beedd0` | `— ⚠reject` | AdminBrokerServerVersionPacket | Initializes the administrative broker server-version packet. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00beeed0` | `— ⚠reject` | BrokerErrorPacket | Initializes a broker error packet object. | [[Native Map/Shards/S0378 00becaa0-00bf1530|S0378]] |
| `00c0dcd0` | `— ⚠reject` |  | Client-side Lua command that sends the broker 'place item for sale' request (BrokerPlaceItemPacket) built from the script arguments. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0ddd0` | `— ⚠reject` |  | Client Lua command that cancels a broker listing (BrokerCancelItemPacket). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0de80` | `— ⚠reject` |  | Client Lua command that asks the broker for the coin value of a sale (BrokerRequestItemSaleCoinPacket). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0df30` | `— ⚠reject` |  | Client Lua command that buys a broker listing (BrokerBuyItemPacket). | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0e000` | `— ⚠reject` |  | Client Lua command that re-lists an unsold broker item (BrokerReListItemPacket), adding a price/lookup word taken from the item record. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c0e110` | `— ⚠reject` |  | Client Lua command that issues a broker search (BrokerSearchPacket) with the query string plus up to six integer filters. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bee5f0` | `BaseBrokerPacket::CtorWithSubtype` | send |  | 0x82 |  |
| `00bee720` | `BaseAdminBrokerPacket::CtorWithSubtype` | send |  | 0x84 |  |
| `00bee910` | `BrokerMyItemAddedPacket::Ctor` | send |  | 9 |  |
| `00bef790` | `BrokerErrorPacket_Parse` | recv |  | 0x10 | Hypothesis: the peer sends an error payload containing a broker string key; this inference depends on this parser and BrokerErrorPacket subtype 0x10, and says nothing about original server-side error policy. |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 8 |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 9 |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 10 |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 0xb |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 0xc |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 0x10 |  |
| `00befcf0` | `BaseBrokerPacket_Dispatch` | recv |  | 0x11 |  |
| `00bee850` | `BrokerInformationPacket::Ctor ⚠downgrade` | send |  | 0xc |  |
| `00beeba0` | `BrokerRemoveItemFromListsPacket::Ctor ⚠downgrade` | send |  | 0xb |  |
| `00bef900` | `BrokerInformationPacket_Parse` | recv |  | 0xc | Hypothesis: the broker peer supplies four 32-bit values for an information response; depends on this handler and dispatcher subtype 0x0c, not original server logic. |
| `00bee590` | `— ⚠reject` | send |  | 0x82 |  |
| `00beedd0` | `— ⚠reject` | send |  | 0x11 |  |
