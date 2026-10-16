# Class: FreeformActorManipulationModeBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c9750` | `FreeformActorManipulationModeBase::GetField0C` | ✅ | virtual | Returns the dword stored at object offset +0x0c. |
| `009c9770` | `FreeformActorManipulationModeBase::Dtor` | ✅ | dtor | Restores the FreeformActorManipulationModeBase vtable at the object base. |
| `009c9780` | `FreeformActorManipulationModeBase::ForwardNestedEvent ? ⚠rename` | 🟡 | virtual | Loads the nested handler at +8, then tail-dispatches through its vtable slot +0x0c. |
| `009c9c10` | `FreeformActorManipulationModeBase::ScalarDeletingDtor` | ✅ | dtor | Restores the base-mode vtable, frees when flag bit 0 is set, otherwise returns the object pointer. |
