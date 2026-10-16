# System: ui.friends

33 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0128ea60` | `Engine::AccountShowFriends::Ctor` | Engine::AccountShowFriends | Constructor of the show-friends Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ead0` | `Engine::AccountShowFriends::GetActionName` | Engine::AccountShowFriends | Virtual slot [1] giving the bare action name of the show-friends action; the game use of the name is not established here. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eaf0` | `Engine::AccountShowFriends::GetSourceTag` | Engine::AccountShowFriends | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ec20` | `Engine::AccountAddFriend::GetActionName` | Engine::AccountAddFriend | Virtual slot [1] giving the bare action name of the add-friend action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ec40` | `Engine::AccountAddFriend::GetSourceTag` | Engine::AccountAddFriend | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ed00` | `Engine::AccountRemoveFriend::Ctor` | Engine::AccountRemoveFriend | Constructor of the remove-friend Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ed70` | `Engine::AccountRemoveFriend::GetActionName` | Engine::AccountRemoveFriend | Virtual slot [1] giving the bare action name of the remove-friend action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ed90` | `Engine::AccountRemoveFriend::GetSourceTag` | Engine::AccountRemoveFriend | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eec0` | `Engine::AccountAddIgnore::GetActionName` | Engine::AccountAddIgnore | Virtual slot [1] giving the bare action name of the add-ignore action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eee0` | `Engine::AccountAddIgnore::GetSourceTag` | Engine::AccountAddIgnore | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f010` | `Engine::AccountRemoveIgnore::GetActionName` | Engine::AccountRemoveIgnore | Virtual slot [1] giving the bare action name of the remove-ignore action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f030` | `Engine::AccountRemoveIgnore::GetSourceTag` | Engine::AccountRemoveIgnore | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f160` | `Engine::AccountAcceptFriendRequest::GetActionName` | Engine::AccountAcceptFriendRequest | Virtual slot [1] giving the bare action name of the accept-friend-request action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f180` | `Engine::AccountAcceptFriendRequest::GetSourceTag` | Engine::AccountAcceptFriendRequest | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f2b0` | `Engine::AccountRejectFriendRequest::GetActionName` | Engine::AccountRejectFriendRequest | Virtual slot [1] giving the bare action name of the reject-friend-request action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f2d0` | `Engine::AccountRejectFriendRequest::GetSourceTag` | Engine::AccountRejectFriendRequest | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `01295090` | `Engine::AccountShowFriends::OpenFriendsListDialog` | Engine::AccountShowFriends | Opens the FriendsList dialog with its XML value and page counters. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01290f70` | `Engine::AccountOnReceiveFriendLogoutUC::Handle` | Engine::AccountOnReceiveFriendLogoutUC | Vtable slot [2] handler for a friend going offline: refreshes the Friends list. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `01291250` | `Engine::AccountOnRequestAddFriendUC::ApplyAddFriendResult` | Engine::AccountOnRequestAddFriendUC | Client-side add-friend result handler: dispatches the success path to a friend-request routine and displays a TCG dialog for the error paths. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01291c70` | `Engine::AccountOnRequestAddIgnoreUC::ApplyAddIgnoreResult` | Engine::AccountOnRequestAddIgnoreUC | Client-side add-ignore result handler: refreshes the ignore list on success and displays the matching TCG dialog. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `012928f0` | `Engine::AccountOnRequestFriendConfirmUC::ApplyFriendConfirmResult` | Engine::AccountOnRequestFriendConfirmUC | Client-side friend-confirmation result handler: the zero-code success path also requires arg 5 true; nonzero codes show an error, while zero with arg 5 false takes no action. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01293230` | `Engine::AccountOnRequestRemoveFriendUC::ApplyRemoveFriendResult` | Engine::AccountOnRequestRemoveFriendUC | Client-side remove-friend result handler: refreshes Friends and shows a removed dialog on code 0; other codes select their corresponding dialogs. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01293e90` | `Engine::AccountOnRequestRemoveIgnoreUC::ApplyRemoveIgnoreResult` | Engine::AccountOnRequestRemoveIgnoreUC | Client-side remove-ignore result handler: refreshes Ignores and shows a removed dialog on code 0; other codes select their corresponding dialogs. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01295220` | `Engine::AccountAddFriend::AddFriendOrShowError` | Engine::AccountAddFriend | Add-friend UI action handler that invokes the add routine and displays its two observed error dialogs. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |
| `01295650` | `Engine::AccountAddIgnore::AddIgnoreOrShowError` | Engine::AccountAddIgnore | Add-ignore UI action handler that invokes the ignore routine and displays the observed cannot-ignore-yourself dialog. | [[Native Map/Shards/S0671 01291250-01295650|S0671]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01290c90` | `Engine::AccountOnReceiveFriendLoginUC::Handle ? ⚠downgrade` | Engine::AccountOnReceiveFriendLoginUC | Vtable slot [2] handler for a friend coming online: refreshes the Friends list. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0128ebb0` | `— ⚠reject` | Engine::AccountAddFriend | Constructor of the add-friend Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ee50` | `— ⚠reject` | Engine::AccountAddIgnore | Constructor of the add-ignore Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128efa0` | `— ⚠reject` | Engine::AccountRemoveIgnore | Constructor of the remove-ignore Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f0f0` | `— ⚠reject` | Engine::AccountAcceptFriendRequest | Constructor of the accept-friend-request Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f240` | `— ⚠reject` | Engine::AccountRejectFriendRequest | Constructor of the reject-friend-request Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128fda0` | `— ⚠reject` | Engine::AccountOnReceiveFriendConfirmRe… | Vtable slot [2] handler that shows the incoming friend-invite prompt with Accept and Decline buttons. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `01290210` | `— ⚠reject` | Engine::AccountOnReceiveFriendConfirmRe… | Vtable slot [2] handler for the reply to a friend request the player sent: shows declined or accepted feedback and refreshes the Friends list. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `01290f70` | `Engine::AccountOnReceiveFriendLogoutUC::Handle` | recv | FriendOfflinePacket |  | HYPOTHESIS (class/catalogue/note association only; the native handler does not identify or parse the packet): the server sends the friend-offline notice (community FriendOfflinePacket, family 74 sub-opcode 3 per vault note 087) and the client refreshes its roster. |
| `01290c90` | `Engine::AccountOnReceiveFriendLoginUC::Handle ? ⚠downgrade` | recv | FriendOnlinePacket |  | HYPOTHESIS (class/catalogue/note association only; the native handler does not identify or parse the packet): the server sends the friend-online notice (community FriendOnlinePacket, family 74 sub-opcode 2 per vault note 087) and the client refreshes its roster. |
| `0128fda0` | `— ⚠reject` | recv | CommandPacketConfirmFriendRequest |  | HYPOTHESIS (class/catalogue/note association only; the native handler does not identify or parse the packet): the server sends a friend-confirm request command (community CommandPacketConfirmFriendRequest, sub-opcode 16 per vault note 087) naming the requester; the client answers through the Accept or Reject actions (0128fad0 / 0128fc00). |
| `01290210` | `— ⚠reject` | recv | CommandPacketConfirmFriendResponse |  | HYPOTHESIS (class/catalogue/note association only; the native handler does not identify or parse the packet): the server relays the requested player's answer (community CommandPacketConfirmFriendResponse, sub-opcode 17 per vault note 087); the client shows declined or accepted feedback and refreshes the roster. |
