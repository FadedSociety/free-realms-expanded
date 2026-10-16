# Class: HousePlayerEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5ec0` | `HousePlayerEventHandler::MatchesValue8` | ✅ | virtual | Returns 1 exactly when the input value is 8, otherwise 0. |
| `00aaac60` | `HousePlayerEventHandler::ctor` | ✅ | ctor | Constructor: runs the base initialiser FUN_008b3470, stamps HousePlayerEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aaad10` | `HousePlayerEventHandler::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees the object through _free. |
| `00ab7380` | `HousePlayerEventHandler::OnSelectedFixture` | ✅ | virtual | Checks a guarded housing pointer and its active byte, calls the base event callback, then forwards the event arguments to fixture-selection handling. |
