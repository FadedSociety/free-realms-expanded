# System: game.trade

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c6df10` | `TradeDispatchPacket` |  | Client-side dispatcher for incoming player-to-player trade messages; the strings name the trade script events and UI messages it raises. | [[Native Map/Shards/S0400 00c6df10-00c73b60|S0400]] |
| `00cf68d0` | `TradeSessionTracker::Ctor` | TradeSessionTracker | Initializes client-side trade session state and partner name/coin datasource values. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a10940` | `MethodThunk_00c6df10` |  | Member-function thunk that reaches the trade handler from the client object (+0x1d008 sub-object). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00cf6240` | `GetPartnerCoinCountIfPartnerState` |  | Reads the partner-side coin amount while the trade object is in state 2. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf6260` | `SetTradeField_2e4` |  | Updates one trade session field and triggers its change callback. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf65b0` | `TradeSessionTracker::Dtor` | TradeSessionTracker | Cleans up the client trade session tracker and its owned interface members. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf6850` | `ComputeTradeEligibleCount` |  | Computes count available to show or add in the trade inventory. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cf69c0` | `TradeInventoryAdjustmentCallback ? → TradeOfferItemCountUpdate` |  | Trade inventory adjustment callback; its exact role is unresolved. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf6be0` | `TradeStateCleanup ?` |  | Trade state cleanup, called from trade packet dispatch and trade inventory setup. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |
| `00cf6c90` | `AdjustTradeItemQuantity ?` |  | Adjusts an item's quantity during a trade offer; caller is the trade-offer request handler. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cf6800` | `— ⚠reject` |  | Computes an available count for the selected trade item. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf79d0` | `— ⚠reject` |  | Applies an item-side update from the trade dispatch path. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c6df10` | `TradeDispatchPacket` | recv |  | 2/4/5/0xb/0xc/0xd/0… | Hypothesis: the server sends each trade state change as one family message whose sub-opcode is one of the cases above. This depends on the Trade*Packet readers reached from this switch (for example 00c6c730 and 00c6cdb0), not on this function alone. |
