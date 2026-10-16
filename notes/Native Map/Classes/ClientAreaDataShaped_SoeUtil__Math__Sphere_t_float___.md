# Class: ClientAreaDataShaped<SoeUtil::Math::Sphere_t<float>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b09d90` | `GetAreaShapeData` | 🟢 | virtual | Returns this plus offset 0x1e0 for the shaped client-area data field. |
| `00b09e20` | `GetAreaShapeName` | 🟢 | virtual | Returns the literal string sphere. |
| `00b09e50` | `ForwardAreaDataBoolResult ?` | 🟡 | virtual | Calls the shared area-data validation helper and returns whether validation succeeded. |
| `00b0a160` | `Method_00b0a160 → DrawSphereAreaDebugShape` | 🟢 | virtual | For sphere-shaped data, chooses a debug color from the flag and draws the stored bounds unless disabled. |
| `00b0a290` | `ReadSphereCenterAndRadius` | ✅ | virtual | Reads three keyed coordinate values and radius, converts them to floats, and stores center/radius at offsets 0x1e0?0x1f0. |
| `00b0a370` | `ClientAreaSphere::WriteShape` | ✅ | virtual | Writes shape=sphere and stores center coordinates and radius from offsets 0x1e0, 0x1e4, 0x1e8, and 0x1f0. |
| `00b0aff0` | `ClientAreaSphere::Ctor` | ✅ | ctor | Calls the base area constructor and installs the spherical ClientAreaDataShaped vtable. |
| `00b0b0b0` | `ClientAreaSphere::ContainsPoint` | ✅ | virtual | Subtracts four point components from the stored center and returns whether squared distance is less than the stored radius squared. |
| `00b0ba20` | `ClientAreaSphere::ScalarDeletingDtor` | ✅ | dtor | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c3b0` | `ClientAreaSphere::CallGeometryHelper` | ✅ | virtual | Adds 0x1e0 to this, passes that adjusted pointer as an argument to FUN_008e4330, then returns. |
