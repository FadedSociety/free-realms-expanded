# Class: FreeformActorManipulationActorClickHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9b20` | `FreeformActorManipulationActorClickHandler::DispatchTargetClick` | ✅ | virtual | If the target pointer reached through +0x0c exists, calls its vtable slot +0x10 with this call's arguments 3 and 5 through 8. |
| `009c9b50` | `FreeformActorManipulationActorClickHandler::AcceptInputKind` | ✅ | virtual | Returns true for nonzero secondary flag or kinds 1, 0x1000, 8 and 0x40; returns false for other kinds. |
| `009c9ed0` | `FreeformActorManipulationActorClickHandler::Ctor` | ✅ | ctor | Constructs the TargetProcessor event-handler base, installs the actor-click vtable, and stores the target processor at +0x0c. |
| `009c9f80` | `FreeformActorManipulationActorClickHandler::ScalarDeletingDtor` | ✅ | dtor | Restores the TargetProcessor base vtable and frees the allocation only when flag bit 0 is set. |
