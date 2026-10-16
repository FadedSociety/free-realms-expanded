# System: net.player_update

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0092e5d0` | `SendFlourishUpdate` |  | Sends the client's flourish player-update packet in connected mode. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092e670` | `SendJumpUpdate` |  | Emits a player jump update when connected; the packet class is directly identified in this function. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092e860` | `SendPositionUpdateFlag2` |  | One connected-mode position-update variant; the packet class is identified by this function's vtable reference. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092ea40` | `SendPositionUpdateFlag2WithModeBit` |  | A position-update variant distinguished by its mode-derived flag bit; packet identity is directly referenced in this function. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092ebf0` | `SendPositionUpdateFlag0x82` |  | A position-update variant with flag constant 0x82; packet identity is directly referenced in this function. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092eda0` | `SendPositionUpdateFlag0x42` |  | A position-update variant with flag constant 0x42; packet identity is directly referenced in this function. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |
| `0092f460` | `BaseClient::DispatchBasePlayerUpdate` | BaseClient | Shared BaseClient/PlayClient virtual dispatch for inbound BasePlayerUpdate subtypes; documented cases include NPC creation and item-definition receive handling. | [[Native Map/Shards/S0246 0092f460-0092f460|S0246]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0092f460` | `BaseClient::DispatchBasePlayerUpdate` | recv | BasePlayerUpdatePacket |  | Hypothesis: the original server sent type-35 BasePlayerUpdatePacket messages with subtype payloads and framing accepted by these client readers. Native evidence establishes the receive contract, not original server state logic. |
