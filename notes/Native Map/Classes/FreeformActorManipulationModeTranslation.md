# Class: FreeformActorManipulationModeTranslation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9da0` | `FreeformActorManipulationModeTranslation::DrawAxisHandles ⚠downgrade` | 🟢 | virtual | When active, reads two object vectors, adds a fixed fourth-component offset, builds handle geometry through 0075a000, and submits it through 007590e0 when the render field is set. |
| `009c9e70` | `— ⚠reject` | ⚪ | virtual | Calls child at +8, vtable slot +4, with vector pointers at object offsets +0x190 and +0x2f0. |
| `009ca020` | `— ⚠reject` | ⚪ | virtual | For type 0x1000 while active, clamps a scaled event-coordinate delta and applies it to one or four vector components by flags, then invokes common input handling. |
| `009ca9a0` | `FreeformActorManipulationModeTranslation::Ctor` | ✅ | ctor | Constructs InternalBase state through 009ca8b0, then installs the Translation mode vtable. |
| `009caa70` | `FreeformActorManipulationModeTranslation::ScalarDeletingDtor` | ✅ | dtor | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |
