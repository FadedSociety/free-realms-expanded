# Class: FreeformActorManipulationModeManipulateActorInternalBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c98f0` | `FreeformActorManipulationModeManipulateActorInternalBase::HandleActiveInputTran…` | ✅ | virtual | Event 8 activates the mode and saves coordinates; event 0x40 deactivates it and clears two flags; bit 1 of final flags sets another byte. |
| `009c9980` | `FreeformActorManipulationModeManipulateActorInternalBase::GetActiveOrPrimaryFie…` | 🟡 | virtual | Returns field +0x1c when nonzero, otherwise returns field +0x0c. |
| `009c9d10` | `FreeformActorManipulationModeManipulateActorInternalBase::Dtor ⚠downgrade` | 🟢 | dtor | Restores InternalBase vtable, clears action pointer through a nested virtual call when set, then restores the manipulation base vtable. |
| `009ca000` | `FreeformActorManipulationModeManipulateActorInternalBase::ScalarDeletingDtor` | ✅ | dtor | Calls InternalBase destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009ca8b0` | `FreeformActorManipulationModeManipulateActorInternalBase::Ctor` | ✅ | ctor | Stores context, actor and event values, clears two state bytes, creates the transform action, adjusts its option bit when set, toggles a global flag and initializes a scalar. |
