# System: net.packet.client_path

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a4800` | `ClientPathBasePacket::ClientPathBasePacket` | ClientPathBasePacket | Base constructor of opcode-98 client-path packets. Receive router 009471a0 handles incoming opcode 98; sender 008b7df0 builds the outgoing request path. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4860` | `ClientPathBasePacket::ClientPathBasePacketSub` | ClientPathBasePacket | Variable-sub-opcode constructor for opcode-98 client-path packets. Sender 008b7df0 builds a request; receive router 009471a0 uses the reply construction path through 008eba20. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008b7d90` | `ClientPathBasePacket::ScalarDeletingDtor` | ClientPathBasePacket | Releases a ClientPathBasePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7df0` | `ClientPathRequestPacket::ClientPathRequestPacket` | ClientPathRequestPacket | Initializes a client path request packet object. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7ea0` | `ClientPathRequestPacket::ScalarDeletingDtor` | ClientPathRequestPacket | Releases a ClientPathRequestPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a48d0` | `ClientPathBasePacket::~ClientPathBasePacket` | ClientPathBasePacket | Base teardown step for opcode-98 client-path packets; class binding comes from the router case that constructs and destroys them. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a4800` | `ClientPathBasePacket::ClientPathBasePacket` | send |  | 98 | Hypothesis: the peer accepts client opcode-98 requests and sends opcode-98 replies. Client parser 009471a0 and paths 008b7df0/008eba20 establish both directions; payload semantics remain open. |
| `008a4800` | `ClientPathBasePacket::ClientPathBasePacket` | recv |  | 98 | Hypothesis: the peer accepts client opcode-98 requests and sends opcode-98 replies. Client parser 009471a0 and paths 008b7df0/008eba20 establish both directions; payload semantics remain open. |
| `008a4860` | `ClientPathBasePacket::ClientPathBasePacketSub` | send |  | 98 | Hypothesis: the peer accepts client path requests and returns compatible opcode-98 replies. This depends on sender 008b7df0, router 009471a0 and reply path 008eba20; subtype meanings must be read at those sites. |
| `008a4860` | `ClientPathBasePacket::ClientPathBasePacketSub` | recv |  | 98 | Hypothesis: the peer accepts client path requests and returns compatible opcode-98 replies. This depends on sender 008b7df0, router 009471a0 and reply path 008eba20; subtype meanings must be read at those sites. |
