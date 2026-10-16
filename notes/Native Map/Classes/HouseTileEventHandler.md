# Class: HouseTileEventHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5ef0` | `HouseTileEventHandler::MatchesValue8Or1` | ✅ | virtual | Returns 1 when the input value is 8 or 1, otherwise 0. |
| `00aaad70` | `HouseTileEventHandler::ctor` | ✅ | ctor | Constructor: runs the base initialiser FUN_008b3590, stamps HouseTileEventHandler::vftable at this+0 and stores param_2 at this+0xc. |
| `00aaae20` | `HouseTileEventHandler::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): resets the vtable to the base TargetProcessorEventHandler::vftable and, when the low bit of param_2 is set, frees the object through _free. |
| `00ac9b80` | `HouseTileEventHandler::OnEvent` | ✅ | virtual | Virtual slot 3 of the HouseTileEventHandler vtable: gets the housing controller through FUN_008a67a0(this+0xc), returns when the controller's byte +0x16 is clear, otherwise calls the housing selection handler FUN_00ac91… |
