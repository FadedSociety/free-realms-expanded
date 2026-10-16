# System: game.wall_of_data

9 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00add410` | `WallOfData_RecordEvent6Pair` |  | Records one keyboard/mouse event pair for the batch that 00ade7f0 later flushes as a keyboard telemetry packet. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add5e0` | `WallOfData_Initialize` |  | Initializes the wall-of-data recorder. 00add7a0 later fills its event table, and 00ade7f0 uses +0x184 as the flush interval/value. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add7a0` | `WallOfData_LoadEventTypes ⚠downgrade` |  | Loads the per-event enable/rate configuration used by WallOfData_IsEventEnabled (00add220) and the emitters. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade5e0` | `WallOfData_EmitClickMove` |  | Emits the click-move telemetry row; called from 0093bba0 in the click-to-move code path. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade730` | `WallOfData_EmitWalletBalance` |  | Emits the wallet-balance telemetry row; called from 00a12530 in the wallet/economy code. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ade4f0` | `WallOfData_EmitUIEvent ? ⚠downgrade` |  | Emits the UI-event telemetry row (three strings) raised by UI callbacks. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade690` | `WallOfData_EmitMembershipPurchase ? ⚠downgrade` |  | Emits the membership-purchase telemetry row; called from 00b342f0 in the membership/store code. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade7f0` | `WallOfData_FlushKeyboard ? ⚠rename` |  | Timer-driven flush of the batched keyboard/mouse telemetry; called from the client update path 0094a970. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00add220` | `— ⚠reject` |  | Per-event gate used by Wall-of-Data emitters and the keyboard flush; checks enable, session threshold and optional elapsed-time interval. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ade5e0` | `WallOfData_EmitClickMove` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded click-to-move destinations; depends on sender 00addfe0 and on the click-to-move caller 0093bba0. |
| `00ade730` | `WallOfData_EmitWalletBalance` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded wallet balances; depends on sender 00ade210 and caller 00a12530. |
| `00ade4f0` | `WallOfData_EmitUIEvent ? ⚠downgrade` | send | WallOfDataUIEventPacket |  | Hypothesis: the server received UI-event rows keyed by the three strings; depends on the event-type table and sender 00addeb0. |
| `00ade690` | `WallOfData_EmitMembershipPurchase ? ⚠downgrade` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded membership purchases; depends on sender 00ade100 and caller 00b342f0. |
| `00ade7f0` | `WallOfData_FlushKeyboard ? ⚠rename` | send | WallOfDataBasePacket |  | Hypothesis: the server received the batched input rows; depends on recorder append 00add410 and sender 00ade3d0. |
