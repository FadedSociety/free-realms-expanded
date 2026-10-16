# Class: FreeformActorManipulationModeManipulateActor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c97f0` | `FreeformActorManipulationModeManipulateActor::StoreVectorAndNotifyChild` | ✅ | virtual | Copies four dwords into +0x50..+0x5c and calls child vtable slot +0x18 when the child at +0x14 exists. |
| `009c9830` | `FreeformActorManipulationModeManipulateActor::ForwardChildSlot4` | ✅ | virtual | When child at +0x14 exists, tail-dispatches through its vtable slot +0x10; absent child returns after cleaning 0x14 stack bytes. |
| `009c9850` | `FreeformActorManipulationModeManipulateActor::ForwardChildSlot1` | ✅ | virtual | When child at +0x14 exists, tail-dispatches through its vtable slot +4; otherwise returns. |
| `009c9870` | `FreeformActorManipulationModeManipulateActor::ForwardAdjustedInput ⚠downgrade` | 🟢 | virtual | Calls a virtual method at this-0x10, passes both input values to 00777930, then calls slot +4 on the handler at this-8. |
| `009c98b0` | `FreeformActorManipulationModeManipulateActor::ForwardAdjustedPointerInput ⚠down…` | 🟢 | virtual | Passes the pointed value to the mode at this-0x10 and helper 00779ea0, then forwards the original pointer to handler at this-8 slot +8. |
| `009ca450` | `FreeformActorManipulationModeManipulateActor::Dtor` | ✅ | dtor | Restores derived mode and handler vtables, releases child event handler and three IString members, then restores the base-mode vtable. |
| `009ca590` | `AdjustorThunk_009ca5a0` | ✅ | virtual | Subtracts 0x10 from ECX and jumps to 009ca5a0; this is an adjustor thunk, not the deleting-destructor body. |
| `009ca5a0` | `FreeformActorManipulationModeManipulateActor::ScalarDeletingDtor` | ✅ | dtor | Calls the full ManipulateActor destructor, frees when flag bit 0 is set, and otherwise returns the object pointer. |
| `009cb0b0` | `FreeformActorManipulationModeManipulateActor::Ctor` | ✅ | ctor | Initializes base and handler vtables, stores context values, initializes three IString members, builds the transform action, sets four vector fields, and selects the initial mode. |
