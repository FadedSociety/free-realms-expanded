# System: game.achievement

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bad7a0` | `HashList_ClientAchievementData_256_8_Ctor` | SoeUtil::HashList<ClientAchievementData… | Constructs the HashList wrapper used for ClientAchievementData records. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00ce37d0` | `AchievementDataSource::ColumnNameByIndex` | AchievementDataSource | Returns the column header text for an achievement data row; also slot [1] of AchievementMostRecentDataSource. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce3980` | `AchievementObjectiveDataSource::ColumnNameByIndex` | AchievementObjectiveDataSource | Returns the column header text for an achievement objective row. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ce38c0` | `AchievementDataSource::FlagGatedTailCall ?` | AchievementDataSource | Virtual at AchievementDataSource::vftable +0x30 slot [2] (no stack arguments): when the byte at this+0x22c is zero it calls vtbl+0x38 of the primary object at this-0x30; then tail-jumps to FUN_010c8f30 with ecx = this+4. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce38f0` | `AchievementDataSource::FlagGatedCall010c90c0 ?` | AchievementDataSource | Virtual at AchievementDataSource::vftable +0x34 slot [4] (one stack argument, ret 4): when the byte at this+0x228 is zero it calls vtbl+0x38 of the primary object at this-0x34; then calls FUN_010c90c0 with ecx = this an… | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |
| `00ce3c40` | `AchievementObjectiveDataSource::LinkedIdOrZero ?` | AchievementObjectiveDataSource | Fastcall with no stack arguments: returns the dword at +0xf0 of the object pointed to by this+0x22c, or 0 when that pointer is null. | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bad8f0` | `— ⚠reject` |  | Builds the achievements state of the owner, a player client-data object. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00ce4770` | `— ⚠reject` | AchievementObjectiveDataSource | Formats one cell of the achievement objective table (column order as in ColumnNameByIndex: Name, Description, Status, Completed, NotUsed, Id). | [[Native Map/Shards/S0421 00ce0340-00ce4c30|S0421]] |

