# System: net.acquaintance

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a16c30` | `RemoveAcquaintancePacket_ScalarDeletingDtor` | RemoveAcquaintancePacket | Heap-delete path of the acquaintance removal packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a15150` | `— ⚠reject` |  | Member teardown of the packet class constructed by RemoveAcquaintancePacket (00a16bb0); the trailing BasePacket vtable write marks the end of the object. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16bb0` | `— ⚠reject` | RemoveAcquaintancePacket | Builds a request that removes one acquaintance entry by name; the caller 00a1bb70 fills and sends it. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00c24a00` | `— ⚠reject` |  | Submits a remove-friend request; the payload string fields are not decoded here. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a16bb0` | `— ⚠reject` | send |  | 2 | Hypothesis: the client asks a peer to remove the named acquaintance; caller 00a1bb70 and base constructor 00a10210 establish the client send path and opcode family. The packet catalogue lacks this class, and no server-side removal behavior is established. |
| `00c24a00` | `— ⚠reject` | send | CommandPacketRemoveFriendRequest |  |  |
