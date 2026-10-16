# System: minigame.activity

25 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb3620` | `IClientActivityLaunchProcessor::~IClientActivityLaunchProcessor` | IClientActivityLaunchProcessor | Base destructor of the activity-launch processor interface. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb3630` | `IClientActivityLaunchProcessor::ScalarDeletingDtor` | IClientActivityLaunchProcessor | Deleting destructor of the processor interface. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb36d0` | `ClientMiniGameActivityLaunchProcessor::GetName` | ClientMiniGameActivityLaunchProcessor | Name accessor used by the activity-launch registration/event system. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb36e0` | `ClientMiniGameActivityLaunchProcessor::~ClientMiniGameActivityLaunchProcessor` | ClientMiniGameActivityLaunchProcessor | Base cleanup of the minigame activity-launch processor. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb3810` | `ClientMiniGameActivityLaunchProcessor::ScalarDeletingDtor` | ClientMiniGameActivityLaunchProcessor | Deleting destructor for the minigame processor. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb3880` | `ClientMiniGameActivityLaunchProcessor::CancelGaq` | ClientMiniGameActivityLaunchProcessor | Processor-side cancellation hook for a pending activity-launch request. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb38c0` | `BuildActivityLaunchInviteDialog` |  | The incoming-invite UI path: presents the invite and registers the accept/decline request that the vtable delegates in 00bb37b0/00bb37d0 operate on. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb3e30` | `ClientMiniGameActivityLaunchProcessor::OnInviteAutoDeclined` | ClientMiniGameActivityLaunchProcessor | Notifies the client layer that an invite was declined automatically (for example after a timeout). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4030` | `ClientMiniGameActivityLaunchProcessor::OnInviteSent` | ClientMiniGameActivityLaunchProcessor | Notifies the client event layer that an activity-launch invite was sent, with the actor id and five caller-supplied event values. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4290` | `ClientMiniGameActivityLaunchProcessor::OnMemberRemoved` | ClientMiniGameActivityLaunchProcessor | Reports that a member was removed from the pending activity launch to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4490` | `ClientMiniGameActivityLaunchProcessor::OnInviteResult` | ClientMiniGameActivityLaunchProcessor | Delivers the outcome of a sent invite (accepted/declined result) to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb46f0` | `ClientMiniGameActivityLaunchProcessor::OnKicked` | ClientMiniGameActivityLaunchProcessor | Reports that the local player was kicked from the pending activity launch to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4880` | `ClientMiniGameActivityLaunchProcessor::OnMemberKicked` | ClientMiniGameActivityLaunchProcessor | Reports that another member was kicked from the pending activity launch. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4a80` | `ClientMiniGameActivityLaunchProcessor::OnMatchmakingStart` | ClientMiniGameActivityLaunchProcessor | Signals the start of matchmaking for an activity launch to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4b90` | `ClientMiniGameActivityLaunchProcessor::OnMatchmakingLaunch` | ClientMiniGameActivityLaunchProcessor | Signals that matchmaking produced a launch to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4ca0` | `ClientMiniGameActivityLaunchProcessor::OnMatchmakingCancel` | ClientMiniGameActivityLaunchProcessor | Signals that matchmaking was cancelled to the Lua/UI layer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb3660` | `ClientMiniGameActivityLaunchProcessor::ClientMiniGameActivityLaunchProcessor ⚠d…` | ClientMiniGameActivityLaunchProcessor | Creates the minigame activity-launch processor that owns the GAQ request slot and subscribes to the activity events. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb37b0` | `ClientActivityLaunchGaqRequest::VSlot0Decline` | ClientActivityLaunchGaqRequest | One of the three one-line entries of the request object's vtable; 009c1c60 is the routine whose own strings are the DeclineInvite failure/removal logs, so this slot submits the request as declined. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb37d0` | `ClientActivityLaunchGaqRequest::VSlot1Accept` | ClientActivityLaunchGaqRequest | Accepts the pending activity-launch request; 009c19a0's own strings are the AcceptInvite logs. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb37f0` | `ClientActivityLaunchGaqRequest::VSlot2Decline2` | ClientActivityLaunchGaqRequest | Second vtable entry that declines the request; the duplication of slot 0 means the two slots differ only in how they are reached. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4db0` | `ReadFieldsAndTailRange` |  | Shared field reader of the two activity-launch handlers (00bb4e40 slot 3 and 00bb4f50 slot 7), which both parse the same GAQ invite record; the domain link comes from those callers, not from an in-function string. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4f50` | `ClientMiniGameActivityLaunchProcessor::JoinMultiplayerMiniGame ⚠downgrade` | ClientMiniGameActivityLaunchProcessor | Joins the multiplayer minigame announced by the launch record and tells the global MiniGameStateManager event about it. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb3740` | `— ⚠reject` |  | Cancels a pending activity-launch (GAQ) request; vtable slot 4 of the processor (00bb3880) and 00bb1cd0's cleanup use the identical body. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb3bf0` | `— ⚠reject` | ClientMiniGameActivityLaunchProcessor | Reports a cancelled activity launch to the Lua/UI layer; the GameEvents twin drives the minigame failure handling. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb4e40` | `— ⚠reject` | ClientMiniGameActivityLaunchProcessor | Entry point for an incoming activity-launch invite: either launches immediately or raises the invite dialog owned by 00bb38c0. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

