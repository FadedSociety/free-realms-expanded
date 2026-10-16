# System: script.lobby

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a241e0` | `FireLobbyGameIsStartingEvent ? ⚠rename` |  | Fires the lobby script event that tells UI scripts a lobby game is about to start; called by 00923210. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a242e0` | `FireLobbyGameIsLaunchingEvent ? ⚠rename` |  | Fires the lobby script event announcing that the lobby game is launching; called by 009386f0 and 009b9550. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a243e0` | `FireLobbyShowEvent ? ⚠rename` |  | Fires the lobby script event that opens/shows the lobby UI; called by 00923210 and 009386f0. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

