# System: game.stats

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a12aa0` | `CharacterStatMap_Find` |  | Character-stat lookup: 71 buckets matches the SoeUtil::HashListMap<int,CharacterStat,71,-1> whose constructor is 00a10e40. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12ae0` | `CharacterStatMap_GetOr` |  | Stat value getter with a default, used by combat/effect code. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |
| `00a12b20` | `CharacterStatMap_GetFloatOr` |  | Float-valued character-stat getter with a default. | [[Native Map/Shards/S0287 00a0c3a0-00a13140|S0287]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b404b0` | `— ⚠reject` |  | Per-transaction logging used by the session-stats dump 00b40630 for the order, credit-card and wallet lists (called through 00b40550). | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |
| `00b40550` | `— ⚠reject` |  | Order-aware variant of the transaction log line used for the orders section of the session-stats dump 00b40630. | [[Native Map/Shards/S0343 00b3f370-00b44f00|S0343]] |

