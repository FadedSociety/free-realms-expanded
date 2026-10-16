# Class: GameRuntimeTerrain::Tile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4a350` | `TileCtor` | ✅ | ctor | Installs the Tile vtable, clears tile fields and flags, initializes its arrays and lists, and sets default state values. |
| `00d4a6f0` | `TileDtor` | ✅ | dtor | Clears tile state, releases owned resources and arrays, removes linked RawGroup and RawArea entries, and resets the Tile vtable. |
| `00d4ba30` | `TileScalarDeletingDtor` | ✅ | dtor | Runs Tile cleanup, conditionally frees the Tile object when flag bit 0 is set, and otherwise returns the same pointer. |
