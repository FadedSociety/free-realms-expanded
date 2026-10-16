# Class: ShipCombatProxiedFort

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a50450` | `ShipCombatProxiedFort::DtorAdapter` | 🟢 | dtor | EH-framed adapter calls the ShipCombatProxiedFort destructor body at 00cadb20 with the incoming object in ECX, then returns. The Ghidra DAT_01b839d8 XOR expression is stack-cookie bookkeeping, not a call argument. |
| `00a53de0` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cadb20 (the ShipCombatProxiedFort destructor body, whose code references ShipCombatProxiedFort::vftable and ShipCombatProxie… |
| `00cadb20` | `ShipCombatProxiedFort::Dtor` | ✅ | dtor | Installs the fort vtable, frees an owned cannon-barrage list if present, and calls virtual cleanup on its stored base object. |
| `00cadc60` | `ShipCombatProxiedFort::Ctor` | ✅ | ctor | Initializes proxied structure base fields, stores fort arguments, and installs the fort vtable. |
| `00cadd20` | `ShipCombatProxiedFort::ScalarDeletingDtor` | ✅ | dtor | Calls fort destructor and frees the object when the delete flag's low bit is set. |
