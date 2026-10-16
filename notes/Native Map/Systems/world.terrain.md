# System: world.terrain

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d4a350` | `TileCtor` | GameRuntimeTerrain::Tile | Constructs a runtime terrain tile and its owned containers. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4a6f0` | `TileDtor` | GameRuntimeTerrain::Tile | Destroys runtime terrain tile resources and owned collections. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4ba30` | `TileScalarDeletingDtor` | GameRuntimeTerrain::Tile | Scalar deleting destructor for GameRuntimeTerrain::Tile. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4ba50` | `TileCreateAll` |  | The function's own log strings name Tile::CreateAll and identify missing texture and fallback cases. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4dc20` | `ChunkDtor` | GameRuntimeTerrain::Chunk | Destructor body for the runtime terrain Chunk. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4dea0` | `AdjustorThunk_00d4e210` | GameRuntimeTerrain::Chunk | Adjusts the Chunk subobject pointer before entering the shared deleting-destructor implementation. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e0a0` | `ChunkCtor` | GameRuntimeTerrain::Chunk | Constructs a runtime terrain Chunk and its owned arrays. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |
| `00d4e210` | `ChunkScalarDeletingDtor` | GameRuntimeTerrain::Chunk | Scalar deleting destructor for the runtime terrain Chunk. | [[Native Map/Shards/S0443 00d49d90-00d4f150|S0443]] |

