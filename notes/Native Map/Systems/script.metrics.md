# System: script.metrics

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01044bb0` | `HashListMap_ClientLuaMetricsEntry::Ctor` | SoeUtil::HashListMap<int,ClientLuaMetri… | Constructor of the 20-bucket hash-list map used for ClientLuaMetrics entries. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044db0` | `HashList_ClientLuaMetricsEntry::Ctor` | SoeUtil::HashList<ClientLuaMetrics::Ent… | Constructor of the 20-bucket HashList for ClientLuaMetrics entries, used as a base-level initialiser. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01044e30` | `HashList_ClientLuaMetricsEntry::ScalarDeletingDtor` | SoeUtil::HashList<ClientLuaMetrics::Ent… | Scalar deleting destructor of HashList<ClientLuaMetrics::Entry,20,-1> (vtable slot 0). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01044e50` | `HashList_ClientLuaMetricsEntry::CtorClearFlag` |  | Constructs the HashList<ClientLuaMetrics::Entry> base and clears the byte at +0x64, the same offset that the StringKeyMap20 functions set to 1. Wrapper for an object whose derived class is not identified here. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

