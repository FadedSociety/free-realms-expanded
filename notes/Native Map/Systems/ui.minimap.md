# System: ui.minimap

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008dd710` | `MiniMapResetHandler` |  | Client-side handler that reloads minimap resources and fires the MiniMap:Reset UI event when the incoming name matches the current map (the string anchor carries the domain). | [[Native Map/Shards/S0229 008d92f0-008dea46|S0229]] |
| `00a8e540` | `MiniMapKillIndicator` |  | Removes a keyed indicator from the minimap UI. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8e660` | `MiniMapSelectIndicator` |  | Selects a minimap indicator and synchronizes the selected key with its view state. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8e860` | `MiniMapUpdateIndicatorLocations` |  | Refreshes minimap indicator positions when tracked entities move enough. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a8fc80` | `MiniMapPlaceIndicator` |  | Adds a keyed indicator to the minimap UI and notifies its script layer. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a90070` | `MiniMapPlaceFriendIndicator` |  | Places a friend indicator using the localized minimap friend label. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a901b0` | `MiniMapPlaceGroupMemberIndicator` |  | Places a group-member indicator using the localized minimap group label. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |
| `00a902f0` | `MiniMapPlaceGuildMemberIndicator` |  | Places a guild-member indicator using the localized minimap guild label. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a88280` | `MiniMapClearSelectedIndicator` |  | Clears the minimap selected-indicator state and invokes the matching UI call when the stored selection differs from the default pair. | [[Native Map/Shards/S0309 00a87ba0-00a8cdc0|S0309]] |
| `00a8e840` | `ForwardToMiniMapKillIndicator` |  | Delegates an indicator removal to the minimap handler. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a90430` | `— ⚠reject` |  | Creates and registers a minimap indicator from the supplied keyed record. | [[Native Map/Shards/S0310 00a8ce60-00a91b00|S0310]] |

