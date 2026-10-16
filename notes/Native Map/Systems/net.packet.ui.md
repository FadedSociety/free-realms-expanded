# System: net.packet.ui

26 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a86640` | `BaseUiPacketCtor` | BaseUiPacket | Common constructor of every op-47 packet - both the task/UI notifications the client parses through the dispatcher 00a91bf0 and the client-sent requests such as SelectTaskRequest (sub 6) and the SelectQuest packets. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86710` | `BaseTaskPacketCtor` | BaseTaskPacket | Common constructor of TaskAdd (sub 1), TaskUpdate (2), TaskComplete (3) and TaskFail (4) packets. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86900` | `TaskUpdatePacketCtor` | TaskUpdatePacket | Prepares a TaskUpdate (class 47 sub 2) packet object for de-serialisation by the task/UI dispatcher. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86a80` | `TaskFailPacketCtor` | TaskFailPacket | Prepares a TaskFail (class 47 sub 4) packet object for de-serialisation by the task/UI dispatcher. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86b30` | `SelectTaskRequestCtor` | SelectTaskRequest | Builds the client request that selects/activates a task row; 00a976a0 constructs it and sends it through 00a95f20 when the world connection exists. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86d70` | `UiMessagePacketCtor` | UiMessagePacket | Prepares a UiMessage (class 47 sub 15) packet object for de-serialisation by the task/UI dispatcher (case 15), whose message text/args are then shown or executed by the UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a87d20` | `BaseUiPacket::ScalarDeletingDtor` | BaseUiPacket | Releases a BaseUiPacket-family object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87d80` | `BaseTaskPacket::ScalarDeletingDtor` | BaseTaskPacket | Releases a task packet object (BaseUiPacket subclass family) when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87de0` | `TaskAddPacket::ScalarDeletingDtor` | TaskAddPacket | Releases a task-add packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87e40` | `TaskUpdatePacket::ScalarDeletingDtor` | TaskUpdatePacket | Releases a task-update packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87ea0` | `TaskCompletePacket::ScalarDeletingDtor` | TaskCompletePacket | Releases a task-complete packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87f00` | `TaskFailPacket::ScalarDeletingDtor` | TaskFailPacket | Releases a task-fail packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a87f60` | `SelectTaskRequest::ScalarDeletingDtor` | SelectTaskRequest | Releases a select-task request object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a88020` | `StartTimerPacket::ScalarDeletingDtor` | StartTimerPacket | Releases a start-timer packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a88080` | `ObjectiveTargetUpdatePacket::Construct` | ObjectiveTargetUpdatePacket | Creates the object for objective-target updates (BaseUiPacket family, sub-opcode 14) before its fields are filled from a parse path. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a88120` | `ObjectiveTargetUpdatePacket::ScalarDeletingDtor` | ObjectiveTargetUpdatePacket | Releases an objective-target update packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a881a0` | `UiMessagePacket::ScalarDeletingDtor` | UiMessagePacket | Releases a UI message packet object when its owner drops it; memory management only. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8b710` | `ExecuteScriptPacket::Construct` | ExecuteScriptPacket | Creates an ExecuteScriptPacket (BaseUiPacket family, sub-opcode 7), the UI script-execution packet that the community catalogue lists under class 47 sub-opcode 7. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8b820` | `LoadingScreenPacket::Construct` | LoadingScreenPacket | Creates a LoadingScreenPacket (BaseUiPacket family, sub-opcode 9) carrying two display strings. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8b8c0` | `LoadingScreenPacket::ScalarDeletingDtor` | LoadingScreenPacket | Releases a LoadingScreenPacket object and its two string members when its owner drops it. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a91bf0` | `BaseUiPacketDispatch` |  | Caller 009471a0 routes outer packet family 0x2f here; branches cover task, UI, timer, objective-target and cinematic handling. | [[Native Map/Shards/S0311 00a91bf0-00a97400|S0311]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a867e0` | `TaskAddPacketCtor ⚠downgrade` | TaskAddPacket | Prepares a TaskAdd (class 47 sub 1) packet object for de-serialisation when the task/UI dispatcher handles an incoming row. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86ec0` | `CinematicStartLookAtPacketDtorBody` | CinematicStartLookAtPacket | Cleans up the two Target objects of a parsed CinematicStartLookAtPacket. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a869d0` | `— ⚠reject` | TaskCompletePacket | Prepares a TaskComplete (class 47 sub 3) packet object for de-serialisation by the task/UI dispatcher. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86bf0` | `— ⚠reject` | StartTimerPacket | Prepares a StartTimer (class 47 sub 10) packet object for de-serialisation by the task/UI dispatcher (case 10). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a86e40` | `— ⚠reject` | CinematicStartLookAtPacket | Prepares a cinematic look-at instruction (class 47 sub 16) for de-serialisation; the dispatcher then runs 008c6170 to start the camera move. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a86640` | `BaseUiPacketCtor` | recv |  | 47 | Hypothesis: the original server and client exchanged one class-47 family whose second u16 selects the packet (6 SelectTaskRequest, 10 StartTimer, 15 UiMessage, 16 CinematicStartLookAt, 12 SelectQuest, 13 SelectedQuestLocked per the OSFR packet table). Depends on the per-subclass constructors and on the send helpers outside this shard. |
| `00a86640` | `BaseUiPacketCtor` | send |  | 47 | Hypothesis: the original server and client exchanged one class-47 family whose second u16 selects the packet (6 SelectTaskRequest, 10 StartTimer, 15 UiMessage, 16 CinematicStartLookAt, 12 SelectQuest, 13 SelectedQuestLocked per the OSFR packet table). Depends on the per-subclass constructors and on the send helpers outside this shard. |
| `00a86710` | `BaseTaskPacketCtor` | recv | BaseUiPacket |  |  |
| `00a86900` | `TaskUpdatePacketCtor` | recv |  | 2 | Hypothesis: the server updated an existing task row with class 47 sub-opcode 2 (progress values); the client refreshed the Goals row. Depends on the parser 00a8f5f0 and the task handler. |
| `00a86a80` | `TaskFailPacketCtor` | recv |  | 4 | Hypothesis: the server failed a task with class 47 sub-opcode 4 plus the task id; the client showed the failure state. Depends on the parser 00a8d970 and the task handler. |
| `00a86b30` | `SelectTaskRequestCtor` | send |  | 6 | Hypothesis: the client selected a task with class 47 sub-opcode 6 carrying three ids (task, step and a third handle); the original server started the task timer and pushed TaskUpdate/UiMessage packets back. Depends on 00a976a0 and on the absent task state machine. |
| `00a86d70` | `UiMessagePacketCtor` | recv |  | 0xf | Hypothesis: the server sent UI messages (e.g. quest-limit or reward toasts) as class 47 sub-opcode 15; the client dispatcher resolved the string id and ran the notification helper. Depends on the parser 00a8dba0 and the QuestLimit handling inside 00a91bf0. |
| `00a91bf0` | `BaseUiPacketDispatch` | recv | BaseUiPacket |  | Hypothesis: the server can send BaseUiPacket subtypes to trigger these client branches. This depends on the 0x2f route in 009471a0 and this function's readers/callbacks; server triggers and authoritative effects are unknown. |
| `00a867e0` | `TaskAddPacketCtor ⚠downgrade` | recv |  | 1 | Hypothesis: the original server pushed task rows to the client as class 47 sub-opcode 1 with (task id, name id, icon, state) fields, which the client stored as the Goals/task list. Depends on the parser 00a8f580 invoked by 00a91bf0 and on the Goals window UI outside this shard. |
| `00a869d0` | `— ⚠reject` | recv |  | 3 | Hypothesis: the server marked a task complete with class 47 sub-opcode 3 plus the task id; the client removed the Goals row and paid the reward. Depends on the parser 00a8d900 and the task handler. |
| `00a86bf0` | `— ⚠reject` | recv |  | 10 | Hypothesis: the server started a timed task step for the client with class 47 sub-opcode 10 carrying a duration id; the client ran the countdown and reported back. Depends on the parser 00a8dac0 and on the timer UI. |
| `00a86e40` | `— ⚠reject` | recv |  | 0x10 | Hypothesis: the server triggered a scripted camera look-at with class 47 sub-opcode 16 carrying two targets (source and destination); the client started the cinematic. Depends on 00a91bf0 case 0x10 and on 008c6170 outside this shard. |
