# System: game.social_share

23 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adfc00` | `HandleHttpProviderStatus` |  | Completes social-share HTTP requests; the status strings describe client behavior, not server policy. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0c00` | `SocialShareManager::ScalarDeletingDtor` | SocialShareManager | Deleting destructor for SocialShareManager. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adebb0` | `SocialShareMessageTrackingMap_Construct ⚠downgrade` |  | Initializes the map used for pending social-share message tracking; lookup function 00adee10 searches its bucket chains and handler 00ae0520 consumes a matching response. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adf030` | `MessageTrackingNodeBaseCtor` | SocialShareManager::MessageTrackingNode | Base-constructor half of the tracking-map node used by the keyed constructor 00adf410 and the map insert paths 00adf770/00adf810; the node type is the value of HashListMap<int,SocialShareManager::MessageTrackingNode,64,… | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adf0b0` | `MessageTrackingNodeBaseDtor` | SocialShareManager::MessageTrackingNode | Cleanup half of the tracking-map node, called by the map remove path 00adf8b0 and by the deleting-destructor wrappers 00adf3c0 and 00adf540. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adfef0` | `RequestSocialShareAuthorization` |  | Social-share manager authorization flow; server-side permission rules are not shown. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adffe0` | `SubmitSocialSharePayload` |  | Submits authorized social-share data through the manager's callback path. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae03d0` | `HandleProviderStatusResponse` |  | Receive-side completion for provider-status response sub-opcode 2. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0850` | `SocialShareProviderStateCtor` |  | Builds one of three provider records owned by SocialShareManager. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae08e0` | `SocialShareProviderStateDtor` |  | Element destructor used by the SocialShareManager provider-state array. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0f60` | `HandleSocialSharePublishHttp` |  | HTTP callback continuation for authorized social-share publish. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae1060` | `ProcessSocialShareRequests` |  | Runs queued manager callbacks while preserving the next-node pointer before dispatch. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae10b0` | `PollSocialShareManager` |  | Periodic manager update for both provider slots. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adf620` | `SetFields3d0AndEnable ?` |  | Caller 008ec090 supplies the social-share object; the exact meaning of the three fields is unresolved. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adfe10` | `ClearTrackedRequests ?` |  | Bulk cleanup of tracked social-share requests. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## ⚪ unknown (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adee10` | `— ⚠reject` |  | Resolves a tracked social-share message by id; 00ae0520 uses the result to match an incoming publish response against the pending entry it then clears. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adf650` | `— ⚠reject` |  | Caches one matching social-share string pair; exact pair semantics are not established here. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adfe50` | `— ⚠reject` |  | Queues provider status requests through the manager callback interface. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae00b0` | `— ⚠reject` |  | Refreshes local social-share authorization tracking when identifiers change. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0520` | `— ⚠reject` |  | Receive-side completion for social-share publish response sub-opcode 3. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae09b0` | `— ⚠reject` | SocialShareManager | Constructs the manager coordinating social-share provider state and requests. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0b10` | `— ⚠reject` | SocialShareManager | Tears down SocialShareManager and its inherited HTTP handler state. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0ea0` | `— ⚠reject` |  | HTTP callback continuation for provider status lookup. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00ae03d0` | `HandleProviderStatusResponse` | recv |  | 2 |  |
| `00ae0f60` | `HandleSocialSharePublishHttp` | send |  | 3 |  |
| `00ae0520` | `— ⚠reject` | recv |  | 3 |  |
