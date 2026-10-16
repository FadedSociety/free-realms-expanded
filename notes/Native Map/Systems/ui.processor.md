# System: ui.processor

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a97450` | `UIProcessor::~UIProcessor` | UIProcessor | Tears down the UIProcessor object's owned UI model maps, asset requester and sub-objects. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |
| `00a98020` | `UIProcessor::UIProcessor` | UIProcessor | Initialises the UIProcessor ui-processing state and creates a BaseClient.Quest.Reward sub-object. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a98250` | `UIProcessor::ScalarDeletingDtor` | UIProcessor | Scalar deleting destructor wrapper for UIProcessor, by its neighbour role as the only caller of 00a97450. | [[Native Map/Shards/S0312 00a97450-00a9d360|S0312]] |

