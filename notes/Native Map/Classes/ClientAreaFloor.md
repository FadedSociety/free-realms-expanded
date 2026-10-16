# Class: ClientAreaFloor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b09e90` | `WriteFloorShapeTag` | 🟢 | virtual | Passes the literal key/value pair shape=floor to the shared data writer. |
| `00b0ae40` | `ClientAreaFloor::Ctor` | ✅ | ctor | Calls the base area constructor, zeros three fields, installs the ClientAreaFloor vtable, and sets a state byte at offset 0x1ab. |
| `00b0af00` | `ClientAreaFloor::CopyBounds` | ✅ | virtual | Copies four 32-bit values from offsets 0x1e0, 0x1e4, 0x1e8, and 0x1ec into the caller output array. |
| `00b0af30` | `ClientAreaFloor::ShapeName` | ✅ | virtual | Returns a pointer to the literal string floor. |
| `00b0af40` | `ClientAreaFloor::Identity` | ✅ | virtual | Returns its first argument unchanged. |
| `00b0b9c0` | `ClientAreaFloor::ScalarDeletingDtor` | ✅ | dtor | Calls ClientAreaData cleanup and frees the object when the low flag bit is set; otherwise returns the pointer. |
