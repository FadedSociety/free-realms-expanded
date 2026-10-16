# System: ui.photographer

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c06440` | `RegisterPhotographerModule` |  | Registers the client 'Photographer' Lua module (PhotoStart/Stop, resolution, auto focus, flash, capture, EnableScreenshot - 11 entries). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c06200` | `Photographer_GetPhotoResolution` |  | Handler registered as 'PhotoGetPhotoResolution' in the Photographer module (entry table 0x01829de0): returns the width/height pair the photo mode is using. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

