# System: ui.reward

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88ec0` | `ClientRewardManager::ClientRewardManager` | ClientRewardManager | Creates the client reward manager that dispatches the opcode-50 reward family (dispatcher 00b8a640) and holds the deferred bank totals at +0x3c/+0x40. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88f30` | `ClientRewardManager::~ClientRewardManager` | ClientRewardManager | Tear down the reward manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b890f0` | `ClientRewardManager::ScalarDeletingDtor` | ClientRewardManager | Deleting destructor for the reward manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b899f0` | `ClientRewardManager::PlayItemReceivedNotification` | ClientRewardManager | Route an item reward to either the mystery-chest panel or the generic item-received script event, as recorded in the reward findings note. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a390` | `ClientRewardBundle::ClientRewardBundle` | ClientRewardBundle | Create the bundle carried by RewardBundlePacket (stored at packet+0x1c) and queued inside PlayerRewardEvent. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a400` | `ClientRewardBundle::ScalarDeletingDtor` | ClientRewardBundle | Release bundle objects; the same FUN_0103d010 destruction is used by the event destructor 00b8a560 and the dispatcher cleanup 00b886a0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a460` | `ClientRewardBundle_CopyCtor` | ClientRewardBundle | Duplicate a bundle when a PlayerRewardEvent owns a copy (called from 00b8a4d0). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a4d0` | `PlayerRewardEvent::PlayerRewardEvent` | PlayerRewardEvent | Create the queued event that holds a received reward bundle until it is presented to the player. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a560` | `PlayerRewardEvent::~PlayerRewardEvent` | PlayerRewardEvent | Destroy a queued reward event; called from the deleting destructor 00b8a5d0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a5d0` | `PlayerRewardEvent_ScalarDeletingDtor` | PlayerRewardEvent | Release a queued reward event object. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88850` | `ClientRewardManager::FindReward ? ⚠rename` | ClientRewardManager | Resolve a reward definition/id for the reward presentation paths; called by the reward dispatcher 00b8a640 and by 00cef480/00cff140. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b886a0` | `— ⚠reject` |  | Unwind/cleanup step for temporary reward bundle objects inside the reward dispatcher 00b8a640 (the same bundle destructor is used by 00b8a560 and 00b8a400). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b896b0` | `— ⚠reject` |  | Present the item-received notification for reward bundle entries; called from the bundle presentation 00b89ca0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b89ca0` | `— ⚠reject` |  | Turn one received ClientRewardBundle into the on-screen reward toast / item notification, as recorded in the reward findings note. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b8a5f0` | `— ⚠reject` | PlayerRewardEvent | Deliver the queued reward bundle to the player when the event is processed. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

