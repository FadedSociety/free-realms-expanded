# Class: FreeformActorManipulationModeScale

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9a90` | `FreeformActorManipulationModeScale::PassScaleVectorToChild` | ✅ | virtual | Loads child at +8 and calls its vtable slot +8 with a pointer to this object's four floats at +0x20. |
| `009ca100` | `— ⚠reject` | ⚪ | virtual | For type 0x1000 while active, scales the smaller absolute coordinate delta and adds it to all or one stored float, enforcing a lower bound, then calls common input handling. |
| `009cac00` | `FreeformActorManipulationModeScale::Ctor` | ✅ | ctor | Constructs InternalBase state, initializes three floats from a shared constant, installs Scale vtable and clears field +0x2c. |
| `009cacc0` | `FreeformActorManipulationModeScale::StoreScaleVector` | ✅ | virtual | Copies four dwords from the supplied vector into object fields +0x20 through +0x2c. |
| `009cad30` | `FreeformActorManipulationModeScale::ScalarDeletingDtor` | ✅ | dtor | Runs InternalBase destructor, frees when flag bit 0 is set, otherwise returns the object pointer. |
