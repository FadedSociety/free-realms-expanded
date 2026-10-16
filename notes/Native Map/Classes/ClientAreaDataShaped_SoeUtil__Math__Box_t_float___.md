# Class: ClientAreaDataShaped<SoeUtil::Math::Box_t<float>_>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b09dd0` | `GetAreaShapeData` | 🟢 | virtual | Returns this plus offset 0x1e0 for the box-shaped client-area data field. |
| `00b09e30` | `GetAreaShapeName` | 🟢 | virtual | Returns the address of static data at 0181d270. |
| `00b09e70` | `ForwardAreaDataBoolResult ?` | 🟡 | virtual | Calls the shared area-data validation helper and returns whether validation succeeded. |
| `00b0a1a0` | `Method_00b0a1a0 → DrawBoxAreaDebugShape` | 🟢 | virtual | For box-shaped data, copies stored bounds and static parameters, selects debug color, then draws the box. |
| `00b0a410` | `ClientAreaBox::ReadShape` | ✅ | virtual | Reads six coordinate keys into offsets 0x1e0 through 0x1f8 and initializes the intervening values from a shared default. |
| `00b0a560` | `ClientAreaBox::WriteShape` | ✅ | virtual | Writes shape and six box-bound coordinate fields using values stored at offsets 0x1e0 through 0x1f8. |
| `00b0b110` | `ClientAreaBox::Ctor` | ✅ | ctor | Calls the base area constructor and installs the box-shaped ClientAreaDataShaped vtable. |
| `00b0b1c0` | `ClientAreaBox::ContainsPoint` | ✅ | virtual | Compares four point components against stored lower and upper bounds and returns true only when all four are within range. |
| `00b0ba80` | `ClientAreaBox::ScalarDeletingDtor` | ✅ | dtor | Calls ClientAreaData cleanup and conditionally frees the object when the low flag bit is set. |
| `00b0c2d0` | `ClientAreaBox::FindIntersectingBounds` | ✅ | virtual | Clears a result link, calls the spatial query with the box's stored bounds, and returns a matched node pointer adjusted by 0x34 or null. |
