# Class: SoeUtil::Array<GameRuntimeTerrain::Tile::TileEcoData,5,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d49fb0` | `ArrayDtor` | ✅ | dtor | Destroys the fixed-capacity array state, frees heap storage only when it is not the inline buffer, then runs the dynamic-array base cleanup. |
| `00d4a080` | `InlineCapacityOrAllocate` | ✅ | method | For requests up to 5, writes capacity 5 and returns the aligned inline buffer; larger requests jump to 00d45520. |
| `00d4a220` | `InlineArrayCtor` | ✅ | ctor | Sets the dynamic-array vtable, clears data, size, and capacity, then installs the five-element inline-array vtable. |
| `00d4a280` | `ArrayScalarDeletingDtor` | ✅ | dtor | Runs the fixed-capacity array destructor, frees the object when flag bit 0 is set, and returns the object pointer otherwise. |
| `00d4a5b0` | `ArrayAssign ? ⚠rename` | 🟡 | method | Copies source elements into the destination, clears prior element state, and releases temporary storage used by the assignment path. |
