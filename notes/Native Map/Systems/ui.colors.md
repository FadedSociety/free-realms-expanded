# System: ui.colors

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5410` | `LoadUiColors` |  | Populates the UI colour table at startup (bootstrap loader 0092b170). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5340` | `HashListLinkTailMod10` |  | Link step of the UI colour table (key at node+0x10, value loaded elsewhere); its only caller is the allocating wrapper 00bb5390. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5390` | `HashListAppendAllocMod10` |  | Insert step of the UI colour table behind the UiColors.txt loader 00bb5410. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00c207c0` | `TintGroups_GetValue` |  | Retrieves a value from the TintGroups data source. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

