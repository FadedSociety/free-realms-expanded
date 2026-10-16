# System: net.quest

64 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdc8a0` | `BaseProgressiveQuestPacket::BaseProgressiveQuestPacket` | BaseProgressiveQuestPacket | Base constructor of the opcode 207 family, used both by the inbound handlers in this shard and by the request packets built elsewhere. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdc920` | `BaseProgressiveQuestPacket::~BaseProgressiveQuestPacket` | BaseProgressiveQuestPacket | Tear down the temporary progressive-quest packet objects used by the inbound handlers. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdce90` | `BaseProgressiveQuestPacket::ScalarDeletingDtor` | BaseProgressiveQuestPacket | Deleting destructor of the base progressive-quest packet class. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdcef0` | `ProgressiveQuestClientDataPacket::ScalarDeletingDtor` | ProgressiveQuestClientDataPacket | Deleting destructor of the ClientData packet class. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdcf50` | `ProgressiveQuestShowWindowPacket::ScalarDeletingDtor` | ProgressiveQuestShowWindowPacket | Deleting destructor of the ShowWindow packet class. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdcfb0` | `ProgressiveQuestNotifyRewardItemPacket::ScalarDeletingDtor` | ProgressiveQuestNotifyRewardItemPacket | Deleting destructor of the NotifyRewardItem packet class. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde610` | `LuaCall_UnifiedMessageWindow_ShowItemPanel` |  | Inbound opcode-207 sub-7 handler: it shows the reward item panel with the four parsed values. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00c79730` | `QuestAbandonedPacket::Construct` | QuestAbandonedPacket | Quest-abandoned packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c799c0` | `QuestObjectiveUpdatePacket::Construct` | QuestObjectiveUpdatePacket | Objective-update packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79b70` | `QuestObjectiveFailedPacket::Construct` | QuestObjectiveFailedPacket | Objective-failed packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79c50` | `CompletedQuestCountUpdatePacket::Construct` | CompletedQuestCountUpdatePacket | Completed-quest count update packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79e40` | `SelectQuestPacket::Construct` | SelectQuestPacket | Quest-selection request packet, built by the native setter SelectQuest (00c7c540). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79f00` | `SelectedQuestLockedPacket::Construct` | SelectedQuestLockedPacket | Selected-quest-locked packet, built by the auto-switch setter SetAutoSwitchSelectedQuest (00c7c5d0). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a1a0` | `QuestCompletePacket::ScalarDeletingDtor` | QuestCompletePacket | Scalar deleting destructor of the quest-complete packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a200` | `QuestFailedPacket::ScalarDeletingDtor` | QuestFailedPacket | Scalar deleting destructor of the quest-failed packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a260` | `QuestAbandonedPacket::ScalarDeletingDtor` | QuestAbandonedPacket | Scalar deleting destructor of the quest-abandoned packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a2c0` | `QuestObjectiveActivatedPacket::ScalarDeletingDtor` | QuestObjectiveActivatedPacket | Scalar deleting destructor of the objective-activated packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a320` | `QuestObjectiveUpdatePacket::ScalarDeletingDtor` | QuestObjectiveUpdatePacket | Scalar deleting destructor of the objective-update packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a390` | `QuestObjectiveFailedPacket::ScalarDeletingDtor` | QuestObjectiveFailedPacket | Scalar deleting destructor of the objective-failed packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a3f0` | `CompletedQuestCountUpdatePacket::ScalarDeletingDtor → CompletedQuestCountUpdate…` | CompletedQuestCountUpdatePacket | Scalar deleting destructor of the completed-count update packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a470` | `QuestEndBlockedPacket::Construct` | QuestEndBlockedPacket | Quest-end-blocked packet; the client logs it as received in the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a4f0` | `QuestEndBlockedPacket::ScalarDeletingDtor → QuestEndBlockedPacket::ScalarDeleti…` | QuestEndBlockedPacket | Scalar deleting destructor of the quest-end-blocked packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a550` | `SelectQuestPacket::ScalarDeletingDtor` | SelectQuestPacket | Scalar deleting destructor of the select-quest packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7a5b0` | `SelectedQuestLockedPacket::ScalarDeletingDtor` | SelectedQuestLockedPacket | Scalar deleting destructor of the selected-quest-locked packet (vtable slot [0]). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7cd40` | `DispatchQuestPacket` |  | Handles inbound quest-family updates and routes them into local journal state and UI actions. This describes client behavior only. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |

## 🟢 likely (32)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdc970` | `ProgressiveQuestClientDataPacket::Ctor` | ProgressiveQuestClientDataPacket | Builds the temporary packet object whose fields the opcode-207 sub-1 handler 00bde8a0 parses and installs on the player. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdc9e0` | `ProgressiveQuestClientDataPacket::~ProgressiveQuestClientDataPacket` | ProgressiveQuestClientDataPacket | Tear down the temporary ClientData packet object after 00bde8a0 parsed it. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdca90` | `ProgressiveQuestShowWindowPacket::~ProgressiveQuestShowWindowPacket` | ProgressiveQuestShowWindowPacket | Tear down the temporary ShowWindow packet object used by the inbound handler. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdcae0` | `ProgressiveQuestNotifyRewardItemPacket::Ctor` | ProgressiveQuestNotifyRewardItemPacket | Builds the temporary packet object whose fields the inbound sub-7 handler 00bde610 parses before calling the Lua UnifiedMessageWindow:ShowItemPanel. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdcb40` | `ProgressiveQuestNotifyRewardItemPacket::~ProgressiveQuestNotifyRewardItemPacket` | ProgressiveQuestNotifyRewardItemPacket | Tear down the temporary NotifyRewardItem packet object used by the inbound handler. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdd9c0` | `ClientProgressiveQuestSlot::Read` | ClientProgressiveQuestSlot | Field parser for one ClientProgressiveQuestSlot element of the opcode-207 sub-1 client-data body; called once per element by 00bddfa0. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bddba0` | `ClientProgressiveQuestPrizeSlot::Read` | ClientProgressiveQuestPrizeSlot | Field parser for one ClientProgressiveQuestPrizeSlot element of the opcode-207 sub-1 client-data body; called once per element by 00bde020. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bddcb0` | `ClientProgressiveQuestData_Init → ClientProgressiveQuestData_ArrayMembersInit` | ClientProgressiveQuestData | Constructor body for the 0x50-byte record that carries the opcode-207 sub-1 client data; 00bde8a0 allocates it before parsing. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bddd20` | `ClientProgressiveQuestData::~ClientProgressiveQuestData` | ClientProgressiveQuestData | Frees the two arrays when the data record is replaced or released by 00bde8a0. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdddb0` | `ClientProgressiveQuestData::ScalarDeletingDtor` | ClientProgressiveQuestData | Deletes the 0x50-byte data record; no vtable slot is recorded for it in this shard. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bdde70` | `NotifyRewardItemBody_Read` |  | Body parser for the sub-7 NotifyRewardItem message consumed by 00bde610, which forwards four of the fields to the Lua UnifiedMessageWindow:ShowItemPanel. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde020` | `PacketRead_PrizeArray` |  | Parses the prize array of the opcode-207 sub-1 client-data body into the record's prize array at +0x40. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde1f0` | `ClientProgressiveQuestData_Read` | ClientProgressiveQuestData | Body parser for the opcode-207 sub-1 client-data message; 00bde8a0 and 00bde820 call it on the allocated record. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde820` | `ParseProgressiveQuestClientDataBuffer` |  | Pure validation parser for an opcode-207 sub-1 client-data body; the parsed record is discarded, so callers use it to test a buffer without installing it. No caller is recorded in the map. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde8a0` | `ProgressiveQuestClientData_ApplyFromPacket` |  | Replaces the local progressive-quest data record, parses the supplied buffer, and notifies three data sources after a clean full-buffer parse. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00c79590` | `QuestCompletePacket::Construct ⚠downgrade` | QuestCompletePacket | Quest-complete packet, built by the quest-state handler 00c7cd40 that logs received quest packets. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79600` | `TrivialDtorBasePacket_00c79600` |  | Body destructor of a quest packet temporary (quest-complete family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79660` | `QuestFailedPacket::Construct ⚠downgrade` | QuestFailedPacket | Quest-failed packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c796d0` | `TrivialDtorBasePacket_00c796d0` |  | Body destructor of a quest packet temporary (quest-failed family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c797a0` | `TrivialDtorBasePacket_00c797a0` |  | Body destructor of a quest packet temporary (quest-abandoned family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79810` | `QuestObjectiveAddedPacket::Destruct` | QuestObjectiveAddedPacket | Body destructor of the added-objective packet temporary. The class binding rests on the neighbour DeletingDestruct 00c7c010 (accepted label). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c798a0` | `QuestObjectiveActivatedPacket::Construct ⚠downgrade` | QuestObjectiveActivatedPacket | Objective-activated packet, built by the quest-state handler 00c7cd40. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79920` | `TrivialDtorBasePacket_00c79920` |  | Body destructor of a quest packet temporary (objective-activated family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79a50` | `TrivialDtorBasePacket_00c79a50 → QuestObjectiveUpdatePacket::Destruct` |  | Body destructor of a quest packet temporary (objective-update family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79ae0` | `TrivialDtorBasePacket_00c79ae0` |  | Body destructor of a quest packet temporary (objective-complete family, no constructor in this shard); the exact class is not proven here. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79be0` | `TrivialDtorBasePacket_00c79be0 → QuestObjectiveFailedPacket::Destruct` |  | Body destructor of a quest packet temporary (objective-failed family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79cc0` | `TrivialDtorBasePacket_00c79cc0 → CompletedQuestCountUpdatePacket::Destruct` |  | Body destructor of a quest packet temporary (completed-count family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79dd0` | `TrivialDtorBasePacket_00c79dd0` |  | Body destructor of a quest packet temporary (end-blocked family); the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79eb0` | `TrivialDtorBasePacket_00c79eb0` |  | Body destructor of the select-quest request temporary; the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79f70` | `TrivialDtorBasePacket_00c79f70` |  | Body destructor of the selected-quest-locked request temporary; the exact class is not proven in this function. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c7c250` | `SendSelectQuestRequest` |  | Sends the client's selected-quest request. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `00c7c360` | `SendSelectedQuestLockedRequest` |  | Sends the client's quest helper auto-switch setting. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bdca30` | `ProgressiveQuestShowWindowPacket::Ctor ?` | ProgressiveQuestShowWindowPacket | Initializes a ProgressiveQuestShowWindowPacket scratch object with family 0xcf and constructor sub-opcode 0; the sole caller uses this object while parsing the inbound 0xcf/5 message before showing the mystery reward br… | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde510` | `LuaCall_OrderResponse ?` |  | Inbound opcode-207 sub-6 handler: it forwards an order/claim response to the script layer, first firing a pending callback for the status codes in the switch. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bddfa0` | `— ⚠reject` |  | Parses the slot array of the opcode-207 sub-1 client-data body into the record's slot array at +0x30. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde3d0` | `— ⚠reject` |  | Inbound opcode-207 sub-5 handler: it opens the mystery reward browser in the requested size. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bde9c0` | `— ⚠reject` |  | Dispatch handler for the progressive-quest packet family; sub 1 carries the client data, sub 5 the mystery reward browser, sub 6 the order response and sub 7 the reward item notification. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00c79470` | `— ⚠reject` | QuestInfoPacket | Body destructor of the quest-info packet temporary. The class binding rests on the neighbour DeletingDestruct 00c7bef0 (accepted label), not on this function's own vtable. | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |
| `00c79d20` | `— ⚠reject` | QuestEndPacket | Body destructor of the quest-end packet temporary. The class binding rests on the neighbour DeletingDestruct 00c7c0f0 (accepted label). | [[Native Map/Shards/S0401 00c73bb0-00c7a5b0|S0401]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c79730` | `QuestAbandonedPacket::Construct` | recv |  | 6 |  |
| `00c799c0` | `QuestObjectiveUpdatePacket::Construct` | recv |  | 9 |  |
| `00c79b70` | `QuestObjectiveFailedPacket::Construct` | recv |  | 11 |  |
| `00c79c50` | `CompletedQuestCountUpdatePacket::Construct` | recv |  | 12 |  |
| `00c79e40` | `SelectQuestPacket::Construct` | send |  | 12 |  |
| `00c79f00` | `SelectedQuestLockedPacket::Construct` | send |  | 13 |  |
| `00c7a470` | `QuestEndBlockedPacket::Construct` | recv |  | 16 |  |
| `00bdc970` | `ProgressiveQuestClientDataPacket::Ctor` | recv |  | 1 |  |
| `00bdcae0` | `ProgressiveQuestNotifyRewardItemPacket::Ctor` | recv |  | 7 |  |
| `00c79590` | `QuestCompletePacket::Construct ⚠downgrade` | recv |  | 4 |  |
| `00c79660` | `QuestFailedPacket::Construct ⚠downgrade` | recv |  | 5 |  |
| `00c798a0` | `QuestObjectiveActivatedPacket::Construct ⚠downgrade` | recv |  | 8 |  |
