# System: net.packet.wall_of_data

23 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adcd60` | `WallOfDataBasePacket::WallOfDataBasePacket` | WallOfDataBasePacket | Common header for every opcode-194 telemetry message the client sends (click-move, keyboard, UI events, membership purchase, wallet balance); matches the community catalogue's WallOfDataBasePacket opcode 194. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adce20` | `WallOfDataPlayerClickMovePacket::WallOfDataPlayerClickMovePacket` | WallOfDataPlayerClickMovePacket | Builds the click-move telemetry message (event 3) that records where the player clicked to move; emitter 00ade5e0 fills and sends it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adced0` | `WallOfDataMembershipPurchasePacket::WallOfDataMembershipPurchasePacket` | WallOfDataMembershipPurchasePacket | Builds the membership-purchase telemetry message (event 5); emitter 00ade690 fills the flag/dword and sends it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adcf80` | `WallOfDataWalletBalancePacket::WallOfDataWalletBalancePacket` | WallOfDataWalletBalancePacket | Builds the wallet-balance telemetry message (event 6); emitter 00ade730 fills the two dwords and sends it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add0a0` | `WallOfDataBasePacket::ScalarDeletingDtor` | WallOfDataBasePacket | Heap-delete path for wall-of-data packet objects. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add100` | `WallOfDataPlayerClickMovePacket::ScalarDeletingDtor` | WallOfDataPlayerClickMovePacket | Heap-delete path for click-move telemetry packets. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add160` | `WallOfDataMembershipPurchasePacket::ScalarDeletingDtor` | WallOfDataMembershipPurchasePacket | Heap-delete path for membership-purchase telemetry packets. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add1c0` | `WallOfDataWalletBalancePacket::ScalarDeletingDtor` | WallOfDataWalletBalancePacket | Heap-delete path for wallet-balance telemetry packets. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add3b0` | `WallOfDataPlayerKeyboardPacket::~WallOfDataPlayerKeyboardPacket` | WallOfDataPlayerKeyboardPacket | Non-deleting destructor of the keyboard telemetry packet. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add4c0` | `WallOfDataPlayerKeyboardPacket::WallOfDataPlayerKeyboardPacket` | WallOfDataPlayerKeyboardPacket | Builds the batched keyboard/mouse telemetry message (event 2) flushed by 00ade7f0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add520` | `WallOfDataPlayerKeyboardPacket::ScalarDeletingDtor` | WallOfDataPlayerKeyboardPacket | Heap-delete path for keyboard telemetry packets. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add540` | `WallOfDataUIEventPacket::~WallOfDataUIEventPacket` | WallOfDataUIEventPacket | Non-deleting destructor of the UI-event telemetry packet; the scalar deleting dtor 00add780 and the emitter 00ade4f0 call it. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add700` | `WallOfDataUIEventPacket::WallOfDataUIEventPacket` | WallOfDataUIEventPacket | Builds the UI-event telemetry message (event 4) whose three strings are filled by the emitter 00ade4f0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00add780` | `WallOfDataUIEventPacket::ScalarDeletingDtor` | WallOfDataUIEventPacket | Heap-delete path for UI-event telemetry packets. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00addeb0` | `WallOfData_SendUIEvent` |  | Frames and sends the event-4 UI telemetry message; called by the emitter 00ade4f0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade100` | `WallOfData_SendMembershipPurchase` |  | Frames and sends the event-5 membership-purchase telemetry message; called by the emitter 00ade690. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade210` | `WallOfData_SendWalletBalance` |  | Frames and sends the event-6 wallet-balance telemetry message; called by the emitter 00ade730. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade3d0` | `WallOfData_SendKeyboard` |  | Frames and sends the event-2 batched keyboard/mouse telemetry message; called by the flush timer 00ade7f0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00addaa0` | `WallOfData_WriteHeader` |  | Serializes the [2-byte opcode 194][1-byte event type] header that every wall-of-data telemetry message begins with. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00addb40` | `WallOfDataKeyboardPacket_WriteEntries` |  | Serializes the keyboard/mouse event batch of the event-2 telemetry packet; called by sender 00ade3d0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00addbf0` | `WallOfDataMembershipPurchasePacket_Serialize` | WallOfDataMembershipPurchasePacket | Body serializer for the event-5 membership-purchase telemetry packet; called by sender 00ade100. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00addc90` | `WallOfDataWalletBalancePacket_Serialize` | WallOfDataWalletBalancePacket | Body serializer for the event-6 wallet-balance telemetry packet; called by sender 00ade210. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00addfe0` | `— ⚠reject` |  | Frames and sends the event-3 click-move telemetry message; called by the emitter 00ade5e0. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00adcd60` | `WallOfDataBasePacket::WallOfDataBasePacket` | send | WallOfDataBasePacket | 194 | Hypothesis: the original server received opcode-194 telemetry and used the sub-id to record player activity (movement, keyboard, UI events, purchases, wallet state); depends on the emitters 00ade4f0-00ade7f0 and the send wrappers 00addeb0-00ade3d0. |
| `00adce20` | `WallOfDataPlayerClickMovePacket::WallOfDataPlayerClickMovePacket` | send | WallOfDataBasePacket |  | Hypothesis: the server logged client click-to-move targets for the wall-of-data view; depends on emitter 00ade5e0 and sender 00addfe0. |
| `00adced0` | `WallOfDataMembershipPurchasePacket::WallOfDataMembershipPurchasePacket` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded membership purchases triggered in the client; depends on emitter 00ade690 and sender 00ade100. |
| `00adcf80` | `WallOfDataWalletBalancePacket::WallOfDataWalletBalancePacket` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded the player's coin/station-cash balance changes; depends on emitter 00ade730 and sender 00ade210. |
| `00add4c0` | `WallOfDataPlayerKeyboardPacket::WallOfDataPlayerKeyboardPacket` | send | WallOfDataBasePacket |  | Hypothesis: the server stored batched movement/input events for the wall-of-data feature; depends on 00ade7f0 and the serializer 00ade3d0. |
| `00add700` | `WallOfDataUIEventPacket::WallOfDataUIEventPacket` | send | WallOfDataUIEventPacket |  | Hypothesis: the server recorded UI events (screen/control/action names) for the wall-of-data view; depends on emitter 00ade4f0 and sender 00addeb0. |
| `00addeb0` | `WallOfData_SendUIEvent` | send | WallOfDataUIEventPacket |  | Hypothesis: the original server stored UI-event telemetry with three string dimensions (screen, control, action) for the wall-of-data dashboard; depends on emitter 00ade4f0 and the file-type table 00add7a0. |
| `00ade100` | `WallOfData_SendMembershipPurchase` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded membership-purchase telemetry; depends on emitter 00ade690 (called from 00b342f0). |
| `00ade210` | `WallOfData_SendWalletBalance` | send | WallOfDataBasePacket |  | Hypothesis: the server recorded wallet balance telemetry; depends on emitter 00ade730 (called from 00a12530). |
| `00ade3d0` | `WallOfData_SendKeyboard` | send | WallOfDataBasePacket |  | Hypothesis: the server stored batched movement/input telemetry; depends on flush timer 00ade7f0 and recorder array 00add410. |
| `00addaa0` | `WallOfData_WriteHeader` | send | WallOfDataBasePacket |  | Hypothesis: the server decodes wall-of-data telemetry by reading the 2-byte opcode 194 and the 1-byte event type before the per-event body; depends on the senders 00addeb0-00ade3d0. |
| `00addb40` | `WallOfDataKeyboardPacket_WriteEntries` | send | WallOfDataBasePacket |  |  |
| `00addbf0` | `WallOfDataMembershipPurchasePacket_Serialize` | send | WallOfDataBasePacket |  |  |
| `00addc90` | `WallOfDataWalletBalancePacket_Serialize` | send | WallOfDataBasePacket |  |  |
| `00addfe0` | `— ⚠reject` | send | WallOfDataBasePacket |  | Hypothesis: the server logged click-to-move destinations for the wall-of-data view; depends on emitter 00ade5e0 (called from 0093bba0). |
