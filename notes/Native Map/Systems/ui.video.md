# System: ui.video

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c05820` | `VideoLibrary_GetVideoLibraryVideoInfo` |  | Handler registered as 'GetVideoLibraryVideoInfo' in the VideoCapture client Lua module (entry table 0x01829af8): exposes one saved video-library record to the client UI scripts. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05880` | `VideoCapture_GetVideoCaptureOutputFile` |  | Handler registered as 'GetVideoCaptureOutputFile' in the VideoCapture client Lua module (0x01829af8): returns the configured capture output path to the client UI. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05940` | `VideoLibrary_GetVideoLibraryMyVideosDirectory` |  | Handler registered as 'GetVideoLibraryMyVideosDirectory' in the VideoCapture client Lua module (0x01829af8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05ac0` | `VideoLibrary_SetVideoLibraryVideoInfoParam` |  | Handler registered as 'SetVideoLibraryVideoInfoParam' in the VideoCapture client Lua module (0x01829af8): writes one field of a stored video-info record. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c05bd0` | `VideoLibrary_GetVideoLibraryVideoInfoParam` |  | Handler registered as 'GetVideoLibraryVideoInfoParam' in the VideoCapture client Lua module (0x01829af8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00d2e2b0` | `IStringQuad_Ctor` |  | Initialises the video-capture caption strings of the owning object. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c05780` | `VideoLibraryVideoInfo_ToLuaTable ? ⚠downgrade` |  | Private helper behind the registered VideoCapture function GetVideoLibraryVideoInfo (00c05820): it converts one stored video-info record into the Lua table the client script consumes. The Video domain comes from the onl… | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c05a00` | `— ⚠reject` |  | Handler registered as 'GetNextVideoLibraryFileName' in the VideoCapture client Lua module (0x01829af8). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

