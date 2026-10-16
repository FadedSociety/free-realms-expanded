# Class: FreeformActorManipulationModeRotation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9990` | `— ⚠reject` | ⚪ | virtual | For event 0x1000 while active, chooses the smaller absolute coordinate delta, scales it, changes one vector component per axis flags, calls 007776b0, then handles common state. |
| `009c9e90` | `— ⚠reject` | ⚪ | virtual | Calls child at +8, vtable slot +4, with vector pointers at object offsets +0x190 and +0x2f0. |
| `009caad0` | `FreeformActorManipulationModeRotation::Ctor` | ✅ | ctor | Constructs InternalBase state through 009ca8b0, then installs the Rotation mode vtable. |
| `009caba0` | `FreeformActorManipulationModeRotation::ScalarDeletingDtor` | ✅ | dtor | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |
