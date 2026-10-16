# Class: PetTileTargetHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b52070` | `MatchKind1Or0x1000` | 🟢 | virtual | Two-integer predicate that computes nothing else: returns 1 exactly when param_1 equals 1 and param_2 equals 0, or when param_1 equals 0x1000 regardless of param_2; otherwise returns 0. |
| `00b54780` | `PetTileTargetHandler::PetTileTargetHandler` | ✅ | ctor | Constructor: calls the base constructor 008b3590 (TargetProcessorTileEventHandler, which installs the TargetProcessorEventHandler vtable), then stores PetTileTargetHandler::vftable at *this and saves the argument at +0x… |
| `00b54830` | `PetTileTargetHandler::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to TargetProcessorEventHandler::vftable (the inlined base destructor) and calls _free(param_1) when bit 0 of the flag argument is set; returns the object po… |
| `00b60770` | `PetTileTargetHandler::OnMessage` | 🟢 | virtual | Dispatches on the fifth argument: when param_5 == 1 it forwards all four other arguments to FUN_00b5fbc0, when param_5 == 0x1000 it forwards them to FUN_00b5fcf0, and otherwise does nothing. |
