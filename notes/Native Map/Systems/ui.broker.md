# System: ui.broker

31 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0c930` | `RegisterBrokerModule` |  | Registers the client 'Broker' Lua module (Place, Cancel, Buy, Relist, Search, SelectInventoryCategory) - the consignment-store window API. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00cee3a0` | `BrokerItemDataSource::FieldName` | BrokerItemDataSource | Exposes broker listing row fields to the data-source consumer. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee410` | `BrokerDataSource::FieldName` | BrokerDataSource | Exposes broker-level configuration fields to the data-source consumer. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee480` | `BrokerPlayerInventoryDataSource::Ctor` | BrokerPlayerInventoryDataSource | Constructs the broker player-inventory data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee560` | `BrokerPlayerInventoryDataSource::CanOfferItem` | BrokerPlayerInventoryDataSource | Checks whether an inventory item can be offered in the broker UI. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cee950` | `ThisAdjustToBrokerPlayerInventoryDtor` | BrokerPlayerInventoryDataSource | Secondary-vtable deleting-destructor adjustor. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceece0` | `BrokerPlayerInventoryDataSource::ScalarDeletingDtor` | BrokerPlayerInventoryDataSource | Scalar deleting destructor for the broker player-inventory data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceefe0` | `BrokerItemDataSource::Ctor` | BrokerItemDataSource | Constructs the common broker item data-source object. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef0a0` | `ThisAdjustToBrokerItemDataSourceDtor` | BrokerItemDataSource | Secondary-vtable deleting-destructor adjustor. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef140` | `BrokerItemDataSource::RebuildVisibleIndices` | BrokerItemDataSource | Builds the filtered row-index view for the broker item table. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef210` | `BrokerItemDataSource::ScalarDeletingDtor` | BrokerItemDataSource | Scalar deleting destructor body for the broker item data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef230` | `BrokerLoggableSubobjectDtor` | Loggable | Destructor for the Loggable base subobject used by broker linked nodes. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef480` | `BrokerItemDataSource::GetField` | BrokerItemDataSource | Provides typed item-row fields to the broker UI data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cefe40` | `BrokerDataSource::Ctor` | BrokerDataSource | Builds the broker UI's data-source tree. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0040` | `BrokerDataSource::Dtor` | BrokerDataSource | Cleans the broker UI data-source hierarchy. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cf0150` | `BrokerDataSource::ScalarDeletingDtor` | BrokerDataSource | Scalar deleting destructor for BrokerDataSource. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c0ce10` | `Broker_SelectInventoryCategory` |  | Handler registered as 'SelectInventoryCategory' in the Broker module (0x0182afa8): picks the inventory page shown in the broker window. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00ceeb70` | `BrokerItemDataSource::GetCurrentRecord ⚠downgrade` | BrokerItemDataSource | Returns a broker item data-source record value after an optional notification. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceeca0` | `FindNodeByTwoKeys` |  | Called by the broker relist Lua handler to find an item record. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceee80` | `BrokerListRemoveByTwoKeys` |  | Broker UI list mutation inferred from the broker wrappers that call this routine. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceef10` | `BrokerListRemoveAt290` |  | This adjustor wrapper is used by the broker removal callback. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceef30` | `BrokerListAppendItem` |  | The broker item-added packet parser calls this insertion path. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00ceefb0` | `BrokerListRemoveAt294` |  | This adjustor wrapper is used by a broker packet removal callback. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef0b0` | `BrokerItemDataSource::ClearIndexArray` | BrokerItemDataSource | Releases the cached list-index array owned by the broker data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef2d0` | `ScalarDeletingDtorWrapper_00cef230` |  | Scalar deleting destructor wrapper; concrete class is not named here. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef450` | `FormatInt64AsLld` |  | Used for displaying a 64-bit value in the broker row accessor. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ceec10` | `BrokerItemDataSource::NotifyItemChanged ? ⚠rename` |  | The broker category-selection caller identifies this as an item-change notification path. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ceede0` | `— ⚠reject` |  | Broker row eligibility check inferred from the item-data-source caller. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef2f0` | `— ⚠reject` |  | Removes the head broker-list node; used by broker packet cleanup. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cef3a0` | `— ⚠reject` |  | Removes an owned broker row during packet-list teardown. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |
| `00cefb90` | `— ⚠reject` | BrokerDataSource | Provides broker-wide settings to the UI data source. | [[Native Map/Shards/S0423 00ceb9b0-00cf13e0|S0423]] |

