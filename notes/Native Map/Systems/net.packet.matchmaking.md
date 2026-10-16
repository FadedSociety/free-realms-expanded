# System: net.packet.matchmaking

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a87ad0` | `EncounterMatchmaking::MatchInvitationResponsePacketCtor` | EncounterMatchmaking::MatchInvitationRe… | Builds the client's answer to a group/encounter matchmaking invitation; 00a979a0 fills the ids and sends it through 00a96030 when connected. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a862f0` | `EncounterMatchmakingBaseMatchmakingPacketDtorBody ⚠downgrade` | EncounterMatchmaking::BaseMatchmakingPa… | Destroys one matchmaking packet object after it has been built or parsed. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a87b30` | `— ⚠reject` | EncounterMatchmaking::MatchInvitationRe… | Vtable slot 0 of the matchmaking invitation response packet. | [[Native Map/Shards/S0308 00a81470-00a87b30|S0308]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a87ad0` | `EncounterMatchmaking::MatchInvitationResponsePacketCtor` | send |  | 10 | Hypothesis: the client answered a matchmaking invitation with BaseMatchmakingPacket (141) sub-opcode 10 carrying the inviter and listener ids plus the accept flag; the original server then moved the players into a group. Depends on 00a979a0 and on the group/encounter formation logic outside this shard. |
