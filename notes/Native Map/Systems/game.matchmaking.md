# System: game.matchmaking

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a82b30` | `EncounterMatchmaking::MatchmakingRequestDtor` | EncounterMatchmaking::MatchmakingRequest | Tears down one request entry of the matchmaking request list (the 0xf0-byte nodes walked by the data sources). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a82ba0` | `EncounterMatchmaking::MatchmakingRequestScalarDeletingDtor` | EncounterMatchmaking::MatchmakingRequest | Vtable slot 0 of EncounterMatchmaking::MatchmakingRequest. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83360` | `EncounterMatchmaking::MatchmakingRequestCopyCtor` | EncounterMatchmaking::MatchmakingRequest | Duplicates a request entry, e.g. when a queued request is re-published or when the sender builds a cancel/response copy. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83490` | `OnPlayerMatchmakingRemoved` | MatchmakingRequestsDataSource | Bookkeeping when a player leaves/aborts matchmaking: drops that player's outstanding requests and tells the UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83ed0` | `OnPlayerQueuedGroup` | MatchmakingRequestsDataSource | Applies an AddMatchRequestResponse notification: records the request in the Matchmaking.Requests table and emits the PlayerQueuedGroup UI event for another connection handle. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a84150` | `CancelMatchRequestsForPlayer` | MatchmakingRequestsDataSource | Cancels all of a player's outstanding matchmaking requests, telling the matchmaking server (CancelMatchRequestPacket) and dropping the local table rows. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟢 likely (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a82c80` | `EhCleanupCallMatchmakingRequestDtor` |  | Compiler-generated unwind cleanup wrapper for the matchmaking request destructor. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a82f10` | `MatchmakingRequestDeletingDtorCopy` | EncounterMatchmaking::MatchmakingRequest | Alternate delete form of MatchmakingRequest (duplicate code of 00a82ba0). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a832c0` | `MatchmakingRequestListRemoveNode` |  | Removal step shared by all request-list operations of the matchmaking data sources and handlers. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a835a0` | `RemoveMatchmakingRequestByIds` | MatchmakingRequestsDataSource | Drops the request identified by a 64-bit id pair (the same pair that is copied into cancel/response packets). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83600` | `MatchmakingRequestListClear` |  | Clears a whole matchmaking request list (used before it is replaced or destroyed). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83620` | `MatchmakingRequestCopyCtorWrapper` | EncounterMatchmaking::MatchmakingRequest | Out-of-line copy-construction entry point for MatchmakingRequest. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |
| `00a83a80` | `MatchmakingRequestListAppend` |  | Adds a queued matchmaking request entry (called after the PlayerQueuedGroup event data has been received). | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a83b80` | `BuildAndSendQueueStatsRequest ? ⚠downgrade` |  | Asks the matchmaking server for the statistics of the queue the client is currently interested in. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a83c00` | `— ⚠reject` | MatchmakingQueuesDataSource | Connection lifecycle for the matchmaking service: on connect it asks for the queue list, on loss it clears the queue table and tells the UI. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a84150` | `CancelMatchRequestsForPlayer` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the wire payload of a cancel is [u16 141][u16 6][u64 request handle] inside a class-6 tunnel; the original server had to drop that participant from the queue named by the handle and notify the remaining group members. Depends on 00a838b0's envelope layout and on the OSFR table entry CancelMatchRequestPacket = 6. |
| `00a84150` | `CancelMatchRequestsForPlayer` | send |  | 141/6 | Hypothesis: the wire payload of a cancel is [u16 141][u16 6][u64 request handle] inside a class-6 tunnel; the original server had to drop that participant from the queue named by the handle and notify the remaining group members. Depends on 00a838b0's envelope layout and on the OSFR table entry CancelMatchRequestPacket = 6. |
| `00a83b80` | `BuildAndSendQueueStatsRequest ? ⚠downgrade` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the client requests queue statistics with BaseMatchmakingPacket sub-opcode 0xd and expects a QueueStatsResponse back; note the OSFR server packet table lists QueueStatsRequestPacket as 10, which conflicts with the client constant and should be re-checked. Depends on the caller chain 00a83b80 -> 00a7fd30 and on the reply handler in the matchmaking packet family. |
| `00a83c00` | `— ⚠reject` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the original server received ListQueuesRequestPacket (class 141, sub 1 per OSFR table) wrapped in class 6 immediately after the client noticed the matchmaking server, and answered with ListQueuesResponsePacket rows that populate Matchmaking.Queues. Depends on 00a7fb90 and on the queue response handler. |
