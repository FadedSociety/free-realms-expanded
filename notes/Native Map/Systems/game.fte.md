# System: game.fte

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c89f90` | `ValidateNamedFteTrigger` | BasePacket | Checks whether a named FTE has triggered. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00928bd0` | `ClientFirstTimeEventEntryHashListCtor` |  | Creates the FirstTimeEvent entry hash-list temporary during PlayClient initialization; the specialization is identified by helper 009250a0. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00c88b10` | `FormatFirstTimeEventSummary ⚠downgrade` |  | Formats and displays the current first-time-event list status. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb3320` | `— ⚠reject` |  | Populates the first-time-event table used to gate one-time client events; called from the bootstrap loader 0092b170. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c89270` | `— ⚠reject` |  | Updates the client's first-time-event cleared set and deferred clear queue. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c896f0` | `— ⚠reject` |  | Drains deferred first-time-event callbacks when deferred mode ends. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c89960` | `— ⚠reject` | ClientFirstTimeEventManager | Initializes the client's first-time-event manager state. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c89db0` | `— ⚠reject` |  | Enables or disables the client's first-time-event manager and notifies the server. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00c89db0` | `— ⚠reject` | send | PacketTunneledClientPacket |  | Hypothesis: the original server receives a first-time-event enable-state message and may update associated session state. This inference depends only on the client constructing subtype 4 and sending its bytes through the gateway; server-side use is unknown. |
