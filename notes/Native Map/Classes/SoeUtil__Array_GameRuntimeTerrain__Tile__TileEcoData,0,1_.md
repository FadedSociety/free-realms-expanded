# Class: SoeUtil::Array<GameRuntimeTerrain::Tile::TileEcoData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d49f80` | `ArrayDtor` | ✅ | dtor | Destroys live array elements, frees the backing block with alignment 4, and clears the stored data pointer. |
| `00d4a0b0` | `ArrayScalarDeletingDtor` | ✅ | dtor | Runs dynamic-array cleanup, then frees the array object when flag bit 0 is set; otherwise returns the object pointer. |
