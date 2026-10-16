# System: ui.view

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c13a80` | `ZoomCamera` |  | Script entry to zoom the game camera by a delta; the View module's other entries (SetCameraDistance/Pitch/Heading/Fov, GetCamera*) live outside this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c13ad0` | `RegisterViewLuaModule` |  | Publishes the camera-control table to the client's Lua UI; the module-name pointer resolves to the string View and ZoomCamera (00c13a80) is the only entry of the table that lies in this shard. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

