# Class: HouseActorEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5ed0` | `HouseActorEventHandler::MatchesValue8Or1` | ✅ | virtual | Returns 1 when the input value is 8 or 1, otherwise 0. |
| `00aae3e0` | `HouseActorEventHandler::ctor` | ✅ | ctor | Constructor: runs the base initialiser FUN_008c49a0, stamps HouseActorEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aae490` | `HouseActorEventHandler::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees the object through _free. |
| `00ac9b30` | `HouseActorEventHandler::OnEvent` | ✅ | virtual | Virtual slot 3 of the HouseActorEventHandler vtable: obtains the housing controller through FUN_008a67a0(this+0xc), returns immediately when the controller's byte +0x16 is clear, otherwise forwards its event arguments t… |
