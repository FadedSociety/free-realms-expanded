# System: game.interaction

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a380d0` | `InteractionProcessor::InteractionProcessor` | InteractionProcessor | Client interaction processor for the BaseClient.Interactions system: it owns the interaction list and the target pair the interaction code reads. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a384c0` | `InteractionProcessor::ClearTargetAndSendFreeNpc` | InteractionProcessor | Clears the interaction target: when a target was set it sends the free-NPC interaction record and then resets the stored pair. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00bafe30` | `ClosestInteractionTargetDataSource_ScalarDeletingDtor` | ClosestInteractionTargetDataSource | Destroys a ClosestInteractionTargetDataSource object. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a381a0` | `InteractionProcessor::~InteractionProcessor ⚠downgrade` | InteractionProcessor | Destructor body of the client interaction processor: destroys its interaction list and releases the BaseClient.Interactions object it holds. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a383e0` | `InteractionProcessor::SendInteractRequest` | InteractionProcessor | Builds the client interaction request (CommandPacketInteractRequest) for the current target and hands it to the send path when the owner is ready. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a39000` | `ResetPairAndSendFreeNpc` |  | Clears the interaction target to the default pair and sends the free-NPC interaction record when the owner is ready. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00a39090` | `SendInteractionSelectThenReset` |  | Sends the selected interaction (CommandPacketInteractionSelect, tunnelled) when the owner is ready, then clears the target through 00a39000. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00bafb90` | `ClosestInteractionTargetDataSource::GetField` | ClosestInteractionTargetDataSource | Per-mode field getter of the closest-interaction data source. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bafd00` | `ClosestInteractionTargetDataSource_DtorBody` | ClosestInteractionTargetDataSource | Destroys the closest-interaction data source. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bafd80` | `ClosestInteractionTargetDataSource_Ctor` | ClosestInteractionTargetDataSource | Builds the closest-interaction data source registered under the BaseClient name. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a370b0` | `PairAt10NotDefaultPair ?` |  | Returns 0 when the dword at this+0x10 equals DAT_01be68f8 and the dword at this+0x14 equals DAT_01be68fc; returns 1 otherwise. No calls and no writes. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00963d10` | `— ⚠reject` |  | Updates a state bit, optionally notifies a related object, and stores a secondary state byte. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00a38470` | `— ⚠reject` | InteractionProcessor | Re-sends the interaction request when the player's target pair changes or a request is pending, then clears the stored pair. | [[Native Map/Shards/S0294 00a35150-00a39a80|S0294]] |
| `00baf6e0` | `— ⚠reject` | ClosestInteractionTargetDataSource | Refreshes the cached closest-interaction target used for the interaction prompt. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bafa70` | `— ⚠reject` |  | Thiscall: calls the closest-target update 00baf6e0 when the tracked-count gate passes, and when the cached pair changes it clears the flag on the previous target object (lookup 00984960, virtual slot [9] with 0, then 00… | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a383e0` | `InteractionProcessor::SendInteractRequest` | send | CommandPacketInteractRequest |  |  |
| `00a39090` | `SendInteractionSelectThenReset` | send | CommandPacketInteractionSelect |  |  |
| `00a38470` | `— ⚠reject` | send | CommandPacketInteractRequest |  |  |
