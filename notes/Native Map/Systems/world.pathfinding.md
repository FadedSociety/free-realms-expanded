# System: world.pathfinding

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009cb990` | `ClearBreadcrumbTargetAndPathState` |  | Clears the client's breadcrumb target and associated path state. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ccd50` | `InitPathStateFromMinimapOptions` |  | Initializes the client path state and its minimap visibility setting. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cd010` | `RequestKynapsePath` |  | Requests a Kynapse path between the selected endpoints. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cd2f0` | `HandleKynapsePathReply` |  | Consumes the path reply associated with RequestKynapsePath. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cd6c0` | `SetBreadcrumbTarget` |  | Sets or refreshes the client's breadcrumb destination. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009cbcf0` | `UpdateNodeChainsFromPosition` |  | Called by the breadcrumb updater 009cd410 to maintain path geometry as position changes. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cd410` | `UpdateBreadcrumbPathState` |  | Per-frame path tracking called by PlayClient::Update. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009cd010` | `RequestKynapsePath` | send | PacketTunneledClientPacket |  | Hypothesis: a remote path service accepts this endpoint request and supplies route nodes later consumed by 009cd2f0. The client strings and receive path support this link; they do not establish whether the original server or Kynapse middleware produced the result. |
| `009cd2f0` | `HandleKynapsePathReply` | recv | PacketTunneledClientPacket |  | Hypothesis: the reply source returns an ordered node list or a failed result for the request at 009cd010. The handler's state branches and returned-node loop establish the client contract only; server-side generation remains unknown. |
| `009cd6c0` | `SetBreadcrumbTarget` | send | PacketTunneledClientPacket |  |  |
| `009cd410` | `UpdateBreadcrumbPathState` | send | PacketTunneledClientPacket |  |  |
