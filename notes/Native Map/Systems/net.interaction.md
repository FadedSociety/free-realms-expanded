# System: net.interaction

1 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0092efe0` | `PlayClient::RequestInteraction` | PlayClient | Client interaction request path for an eligible nearby target; it is gated by game state and the target scan. | [[Native Map/Shards/S0245 0092b170-0092f380|S0245]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `0092efe0` | `PlayClient::RequestInteraction` | send | CommandPacketInteractRequest |  |  |
