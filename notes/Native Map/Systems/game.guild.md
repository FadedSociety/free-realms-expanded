# System: game.guild

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a12cd0` | `PendingGuildNameRequestMap_Dtor` | ClientGuildManager::PendingGuildNameReq… | Destroys the guild manager's pending-name-request map; the vtable names the owning class explicitly. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a13ed0` | `HashMap_PendingGuildNameRequest_DtorWithEH` | SoeUtil::HashMap<int,ClientGuildManager… | Destructor body used when the map is destroyed in place (for example as a member or a stack object) rather than deleted through the deleting destructor at 00a13660. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00b1e2b0` | `GuildNameHashListMapCtor` | SoeUtil::HashListMap<int,GuildName,100,… | Initializes the manager?s guild-name lookup table. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1e330` | `GuildNameHashListScalarDeletingDtor` | SoeUtil::HashList<GuildName,100,-1> | Scalar deleting destructor for HashList<GuildName,100,-1>; calls the hash-list cleanup and conditionally frees the object. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a13660` | `HashMap_PendingGuildNameRequest_ScalarDeletingDtor` | SoeUtil::HashMap<int,ClientGuildManager… | Destroys the container whose RTTI names its value type as ClientGuildManager::PendingGuildNameRequest; this body does not tie its entries to any specific guild or appearance rename packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00b1bba0` | `TouchPairTimeEntry` |  | Touch helper for the request cache; caller 00b1e960 reads this stamp and suppresses a repeated request while its age is below 0x3d. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1bc10` | `InitHashListNodeKey` |  | Node initialiser of the 100-bucket hash list used by the guild manager: the insert path 00b1c110 reads back the +0x48 key stored here to pick the bucket (*key % 100), so the label follows that caller. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1ca10` | `HandleGuildInviteCancel` |  | Handler for guild sub-packet 26 (GuildInviteCancelPacket) in the dispatcher 00b1ee10 (case 0x1a): it only clears client-side invite state, so the label follows the caller's case and the recovered table. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1ce80` | `HandleGuildPlayerStatusUpdate` |  | Guild sub-opcode 31 handler. The interpretation of +0x3a0/+0x3a4 as the local guid is inferred from the client object layout. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d030` | `UpdateGuildNameCacheEntry` |  | Updates the client guild-name cache and publishes the changed name through a helper whose exact datasource contract is not recovered. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1e3b0` | `GuildManagerCtor` |  | Constructor of the guild client manager that owns the guild list, the name table and the two Lua-facing datasources: FUN_00cb6120 and FUN_00cb6260 are the builders of the objects whose strings are 'BaseClient.Guilds' an… | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1e960` | `RequestGuildName` |  | Guild-name cache lookup and request path. The 0x3d comparison is a 61-unit window in the low dword returned by FUN_00702670; it is not established as wall-clock duration. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1e7e0` | `HandleGuildInviteNotification ? ⚠downgrade` |  | Guild sub-opcode 24 handler. One branch registers an invite action; the other auto-sends a decline after the inviter query. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b1c110` | `— ⚠reject` |  | Insert path of the guild manager's 100-bucket hash list: the constructor 00b1e2b0 stores SoeUtil::HashList<GuildName,100,-1>::vftable into the same layout (count at +0x10, buckets from +0x14), which is what the bucket a… | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d6c0` | `— ⚠reject` |  | Handles guild sub-packet 28 (GuildInviteDeclineNotificationPacket) in 00b1ee10 (case 0x1c), distinguishing a declined invitation from a timed-out one by the boolean. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1d7d0` | `— ⚠reject` |  | Handles guild sub-packet 17 (GuildMemberStatusUpdatePacket) in 00b1ee10 (case 0x11): it keeps the member table in step and announces the membership change. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00b1ec10` | `— ⚠reject` |  | Replaces cached guild state from a full-data response and notifies the client data source. | [[Native Map/Shards/S0337 00b1b5a0-00b21f20|S0337]] |
| `00cb6b80` | `— ⚠reject` | GuildMemberDataSource | Applies an update to one cached guild member row. | [[Native Map/Shards/S0412 00cb0bf0-00cb6b80|S0412]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b1ca10` | `HandleGuildInviteCancel` | recv | BaseGuildPacket |  | Hypothesis with a wire link: the original server cancelled a pending guild invitation with sub-opcode 26; the client had no reply. Depends on 00b1ee10 case 0x1a. |
| `00b1ce80` | `HandleGuildPlayerStatusUpdate` | recv | GuildPlayerStatusUpdatePacket |  | Hypothesis with a wire link: the original server sent a player guild-status update; the client refreshed local guild state only when the returned player guid matched its client guid. Depends on dispatcher 00b1ee10 and guild data source from 00b1e3b0. |
| `00b1e960` | `RequestGuildName` | send | GuildNameRequestPacket |  | Hypothesis with a wire link: on cache miss the client sends a guild-name request and records a timestamp to throttle repeats; a server response would populate the name table through 00b1d030. Depends on 00b1e4e0 and cache helpers 00b1bba0/00b18ed0. |
| `00b1e7e0` | `HandleGuildInviteNotification ? ⚠downgrade` | recv | GuildInviteNotificationPacket |  | Hypothesis with wire links: the original server delivered invite notifications; for one inviter-query outcome the client registered an invite action, and for the other it sent a decline. This describes client-observed protocol, not original server policy. Depends on 00b1ee10 case 0x18 and this handler's branches. |
| `00b1e7e0` | `HandleGuildInviteNotification ? ⚠downgrade` | send | GuildInviteDeclinePacket |  | Hypothesis with wire links: the original server delivered invite notifications; for one inviter-query outcome the client registered an invite action, and for the other it sent a decline. This describes client-observed protocol, not original server policy. Depends on 00b1ee10 case 0x18 and this handler's branches. |
| `00b1d6c0` | `— ⚠reject` | recv | GuildInviteDeclineNotificationPacket |  | Hypothesis with a wire link: the same sub-opcode 28 covered both decline and timeout, with the flag distinguishing them; the original server therefore decided which one to report. Depends on 00b1ee10 case 0x1c and 00b1d1b0. |
| `00b1d7d0` | `— ⚠reject` | recv | GuildMemberStatusUpdatePacket |  | Hypothesis with a wire link: sub-opcode 17 carried the guild guid, member guid and status fields; the client applied them and produced the player-facing message, so the original server owned the membership decision. Depends on 00b1ee10 case 0x11, on 00b1c840 for the field widths and on the member datasource from 00b1e3b0. |
| `00b1ec10` | `— ⚠reject` | recv | GuildDataFullPacket |  |  |
