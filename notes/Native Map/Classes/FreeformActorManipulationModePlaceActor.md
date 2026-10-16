# Class: FreeformActorManipulationModePlaceActor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9790` | `FreeformActorManipulationModePlaceActor::SetPlacementVector ⚠downgrade` | 🟢 | virtual | Builds a four-float vector, sets its first component from nested field +0x390 plus a constant, passes it to own slot +0x14, then calls 007776b0. |
| `009c9c30` | `FreeformActorManipulationModePlaceActor::Dtor ⚠downgrade` | 🟢 | dtor | Restores PlaceActor vtable, clears its current action through a nested virtual call when set, then restores the base-mode vtable. |
| `009c9cc0` | `FreeformActorManipulationModePlaceActor::HandlePlacementEvent` | ✅ | virtual | Reads vector through virtual slot +0x14; kind 1 passes vector offsets +400/+0x2f0 to child callback, kind 0x1000 calls 00777610. |
| `009c9fe0` | `FreeformActorManipulationModePlaceActor::ScalarDeletingDtor` | ✅ | dtor | Calls PlaceActor destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009ca7d0` | `FreeformActorManipulationModePlaceActor::Ctor` | ✅ | ctor | Stores context and actor pointers, installs base then PlaceActor vtables, builds the initial vector and transform action, stores its pointer, and toggles a global flag. |
