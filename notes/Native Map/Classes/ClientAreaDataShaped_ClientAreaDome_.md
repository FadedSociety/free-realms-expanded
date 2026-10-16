# Class: ClientAreaDataShaped<ClientAreaDome>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b09db0` | `GetAreaShapeData` | 🟢 | virtual | Returns this plus offset 0x1e0 for the dome-shaped client-area data field. |
| `00b09e10` | `GetAreaShapeData` | 🟢 | virtual | Returns this plus offset 0x1e0 for the second dome data field. |
| `00b09e40` | `ReturnStaticShapeNameData ? → GetAreaShapeName` | 🟡 | virtual | Returns the address of static data at 0181d274; its contents are not proven by this function body. |
| `00b0a100` | `TestDomeBounds ? ⚠rename` | 🟡 | virtual | Validates an area object, copies eight float values from a virtual getter, applies a dome bound test, and returns its result. |
| `00b0a250` | `Method_00b0a250 → DrawDomeAreaDebugShape` | 🟢 | virtual | For dome-shaped data, chooses a debug color from the flag and draws the stored bounds unless disabled. |
| `00b0a650` | `ClientAreaDome::ReadShape` | ✅ | virtual | Reads three center coordinates and radius into offsets 0x1e0, 0x1e4, 0x1e8, and 0x1f0, setting the fourth coordinate from a default. |
| `00b0a730` | `ClientAreaDome::WriteShape` | ✅ | virtual | Writes shape, center coordinates, and radius from the dome object's geometry fields. |
| `00b0b500` | `ClientAreaDome::Ctor` | ✅ | ctor | Calls the base area constructor and installs the ClientAreaDataShaped<ClientAreaDome> vtable. |
| `00b0b570` | `ClientAreaDome::ContainsPoint` | ✅ | virtual | Computes squared distance between four point components and stored center, then compares against squared radius. |
| `00b0bae0` | `ClientAreaDome::ScalarDeletingDtor` | ✅ | dtor | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c3d0` | `ClientAreaDome::CallGeometryHelper` | ✅ | virtual | Adds 0x1e0 to this, passes that adjusted pointer to FUN_00b0bbc0 together with the second argument, then returns. |
