# System: game.friends

14 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00917b20` | `SetAutoRefuseSetting` |  | Stores the auto-refuse choice for one kind of invitation (the table entries are invitation kinds, the first one FriendInvitation). | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `00917b60` | `IsAutoRefuseEnabled` |  | Tells invitation code whether auto-refuse is on for an invitation category. The GAQ invite creators call it. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `0093b9b0` | `CheckAndSubmitFriendRequest` |  | Validate and submit an add-friend request. The first condition is a case-insensitive comparison with the current profile name; the later predicates independently block submission. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00cb3fd0` | `ShowFriendStatusMessage` |  | Presents friend request and teleport status notifications. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `00cb47a0` | `HandleFriendOnline` |  | Applies an online event to the local friend cache and notifies the social UI script. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `00cb4da0` | `HandleFriendAdded` |  | Processes a friend-added event in the local client. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `00cb4ee0` | `HandleFriendRemoved` |  | Processes a friend-removed event in the local client. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cb4170` | `CopyRecordAndSetIndex` |  | Initializes a temporary record used by the friend data-source code. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `00cb4450` | `CopyRecordAndStoreField` |  | Helper used by the friend-data update code to prepare a client record. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `00cb4950` | `HandleFriendOffline ⚠downgrade` |  | Applies an offline event to the local friend cache and notifies the social UI script. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |
| `0128f830` | `Engine::AccountRemoveFriend::Handle` | Engine::AccountRemoveFriend | Vtable slot [2] RemoveFriend action handler; gates a call to FUN_013bda40() on AccountID and FUN_0127ebb0(), passing two global strings. The callee effect is unknown. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f980` | `Engine::AccountRemoveIgnore::Handle` | Engine::AccountRemoveIgnore | Vtable slot [2] RemoveIgnore action handler; gates a call to FUN_013bdb40() on AccountID and FUN_0127ebb0(), passing two global strings. The callee effect is unknown. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128fad0` | `Engine::AccountAcceptFriendRequest::Handle` | Engine::AccountAcceptFriendRequest | Vtable slot [2] handler of the AcceptFriendRequest action: builds the reciprocate friend-confirm command with accept set to 1. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128fc00` | `Engine::AccountRejectFriendRequest::Handle` | Engine::AccountRejectFriendRequest | Vtable slot [2] handler of the RejectFriendRequest action: builds the reciprocate friend-confirm command with accept cleared to 0. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0093b9b0` | `CheckAndSubmitFriendRequest` | send | CommandPacketAddFriendRequest |  | Hypothesis: the endpoint processes an add-friend request after the client name and candidate checks; this depends on the CommandPacketAddFriendRequest constructor and client-world tunnel send, not original server code. |
| `0093b9b0` | `CheckAndSubmitFriendRequest` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the endpoint processes an add-friend request after the client name and candidate checks; this depends on the CommandPacketAddFriendRequest constructor and client-world tunnel send, not original server code. |
