# Class: FreeformActorManipulationTerrainClickHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9ab0` | `FreeformActorManipulationTerrainClickHandler::DispatchTargetClick` | ✅ | virtual | If the target pointer reached through +0x0c exists, calls its vtable slot +0x10 with this call's arguments 2 and 4 through 7. |
| `009c9ae0` | `FreeformActorManipulationTerrainClickHandler::AcceptInputKind` | ✅ | virtual | Returns true for nonzero secondary flag or kinds 1, 0x1000, 8 and 0x40; returns false for other kinds. |
| `009ca240` | `FreeformActorManipulationTerrainClickHandler::Ctor` | ✅ | ctor | Constructs the TargetProcessor tile-event base, installs the terrain-click vtable, and stores the target processor at +0x0c. |
| `009ca2f0` | `FreeformActorManipulationTerrainClickHandler::ScalarDeletingDtor` | ✅ | dtor | Restores the TargetProcessor base vtable and frees the allocation only when flag bit 0 is set. |
