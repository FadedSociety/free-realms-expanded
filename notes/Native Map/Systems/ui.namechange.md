# System: ui.namechange

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0098dbd0` | `NameChangeDataSource::GetColumnName` | NameChangeDataSource | Provides the field names exposed by the name-change data source. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098dc40` | `NameChangeDataSource::Method2` | NameChangeDataSource | Runs this data source's second virtual method and its follow-up helper; its higher-level effect is unresolved. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098dc70` | `NameChangeDataSource::RebuildFilteredRows` | NameChangeDataSource | Rebuilds the name-change data source's visible row list under its optional key filter. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e240` | `NameChangeDataSource::GetFieldValue` | NameChangeDataSource | Exposes name-change records to the UI through the NameChangeDataSource field getter. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e790` | `NameChangeDataSource::Destructor` | NameChangeDataSource | Tears down NameChangeDataSource's embedded user-data table. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e810` | `NameChangeDataSource::Constructor` | NameChangeDataSource | Initializes the name-change data source and its client property group. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `0098e8a0` | `NameChangeDataSource::ScalarDeletingDestructor` | NameChangeDataSource | Provides scalar deletion for the name-change data source. | [[Native Map/Shards/S0264 00989810-0098eae0|S0264]] |
| `00a15730` | `RefreshNameChangePermissions` |  | Refreshes name-change UI values and per-element state; eligible list entries also trigger the client?s NudgeUpdate send helper. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c1f5a0` | `StartTwoPartNameCheck` |  | GFC binding for the two-part name check implemented by 00a21c00. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |
| `00c1f720` | `ConfirmNameChangeForLocalHandle` |  | GFC binding for the local-handle name-change confirmation handled by 00a20b50. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a16e30` | `— ⚠reject` |  | User-visible handling of the rename outcome, called by the response dispatcher 00a21840 right after it parses a sub-opcode 4 NameChangeResponsePacket. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a16e30` | `— ⚠reject` | recv | NameChangeResponsePacket |  | Hypothesis: the numeric values delivered by the 192/4 receive path are peer-provided rename result codes. This depends on dispatcher 00a21840, parser 00a1ac10, and comparison with the emulator enum; the client text mapping is proven, but original-server semantics are not. |
