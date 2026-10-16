# Class: SoeUtil::HashListMap<int,RecipeData,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c8a6c0` | `HashListMapRecipeDataCtor` | ✅ | ctor | Installs the map vtable, clears scalar fields, zeroes 64 bucket dwords, and returns the map address. |
| `00c8bf50` | `RecipeHashListMapDtor` | ✅ | dtor | Restores the hash-map vtable and repeatedly removes the current head node until the map is empty. |
| `00c8bfd0` | `RecipeHashListMapScalarDeletingDtor` | ✅ | dtor | Restores its map vtable, removes nodes until empty, and frees the map when deleting flag bit 0 is set. |
