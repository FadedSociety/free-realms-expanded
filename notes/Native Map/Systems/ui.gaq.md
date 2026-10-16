# System: ui.gaq

33 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a09070` | `GaqItemGuildRenameRequest_OnNameDenied` | GaqItemGuildRenameRequest | Denied branch of the guild-name request game action: tells the UI that the chosen name was rejected. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a770` | `CallLuaHandlerWithPlayerAndGuild` |  | Raises a guild-aware script handler such as GAQGuildInvite when a guild invite game action is added. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a0a890` | `CallLuaHandlerWithPlayer` |  | Raises player-only script handlers such as GAQHouseInvite, GAQFriendInvite, GAQGroupInvite and GAQInviteAndStartMiniGame. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## 🟢 likely (25)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a0c3a0` | `ClientRewardManager_AddItemReceived` | ClientRewardManager | Feeds the client reward manager's 'you received an item' notification into the GAQ UI queue; the class name comes from the caller 00b899f0, whose own log string starts with 'ClientRewardManager::PlayItemReceivedNotifica… | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0ca80` | `Gaq_AddFriendBeatHighScore` |  | Queues the 'a friend beat your high score' GAQ notification item with its parameters. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0cd90` | `Gaq_AddGameTopScore` |  | Queues the 'new game top score' GAQ notification. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0d090` | `Gaq_AddPersonalHighScore` |  | Queues the 'new personal high score' GAQ notification. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0d540` | `Gaq_AddTradeInviteThunk` |  | Call-site wrapper for Gaq_AddTradeInvite that dereferences the GAQ manager holder before the call. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0d550` | `Gaq_AddGroupAnnounceEncounter` |  | Queues the group-encounter announcement toast (used when a group starts a minigame encounter). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0d810` | `Gaq_AddGroupAnnounceEncounterThunk` |  | Call-site wrapper for Gaq_AddGroupAnnounceEncounter. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0daa0` | `Gaq_AddDuelRequestThunk` |  | Call-site wrapper for Gaq_AddDuelRequest. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0dc00` | `Gaq_RemoveItemByIdThunk` |  | Convenience wrapper used by encounter/trade cancellation paths to drop a queue item and tell Lua. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0dc20` | `Gaq_AddFriendInviteToGame` |  | Queues the 'friend invites you to a game' toast (mini-game invitation). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0df40` | `Gaq_RemoveExpiredItems` |  | Periodic cleanup of GAQ items that report themselves expired/finished. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0df80` | `Gaq_RemoveItemsExcept` |  | Cleanup used when a group/guild invitation is answered: every matching invite except the answered one is dropped. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0dfe0` | `Gaq_RemoveExpiredItemsThunk` |  | Call-site wrapper around the expired-item sweep, used by minigame/encounter cancellation handlers. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e020` | `Gaq_Thunk_00a0bf80` |  | GAQ manager call-site wrapper (target body is outside this shard). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e030` | `Gaq_RemoveByTwoFields` |  | Targeted removal of a queued invitation identified by a pair of ids. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e090` | `Gaq_AddClaimNotification` |  | Queues the item-claim notification shown after a coin-store purchase or gift claim. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e670` | `Gaq_AddFriendInviteToGameThunk` |  | Call-site wrapper for the friend mini-game invitation GAQ action. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e6c0` | `Gaq_AddItemReceivedThunk` |  | Call-site wrapper used by the reward/gift handlers to reach the item-received notification. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e770` | `Gaq_RemoveByItemFields` |  | Targeted removal of a queued item identified by two item fields. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e7f0` | `Gaq_DispatchItemById` |  | Per-item action dispatcher for the GAQ (used when an item's state changes so the queue can drop or update it). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e9e0` | `Gaq_FlushItemsToLua` |  | Propagates the C++ queue's item states to the Lua GAQ UI (drop items whose status changed). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0eb40` | `Gaq_FlushItemsToLuaThunk` |  | Call-site wrapper for the GAQ flush-to-Lua sweep. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0eb50` | `Gaq_AddClaimNotificationThunk` |  | Call-site wrapper for the item-claim notification builder. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0eb90` | `Gaq_RemoveFirstSlot14Thunk` |  | Call-site wrapper for the first-slot-14 removal used by trade/guild cancellation paths. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0ebc0` | `Gaq_RemoveByItemFieldsThunk` |  | Call-site wrapper for the two-field queue removal. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a0e740` | `Gaq_RemoveFirstSlot14 ?` |  | Removes the first queue item of a given kind (kind tested by the item's own virtual predicate). | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0e7c0` | `Gaq_RemoveFirstSlot20 ?` |  | Removes the first queued item of the kind selected by slot +0x20. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a0d320` | `— ⚠reject` |  | Queues the incoming trade-invite toast; the queued GaqItemTradeInvite carries the id used to accept/decline. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0d840` | `— ⚠reject` |  | Queues an incoming duel request toast. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a0dae0` | `— ⚠reject` |  | Removes one GAQ notification/queue item by id and mirrors the removal into the Lua GAQ UI. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |

