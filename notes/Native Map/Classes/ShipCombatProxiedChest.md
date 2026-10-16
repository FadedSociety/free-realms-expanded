# Class: ShipCombatProxiedChest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53e40` | `— ⚠reject` | ⚪ | dtor | MSVC scalar deleting destructor shape: under an SEH frame with the stack cookie it calls FUN_00cad030 (the ShipCombatProxiedChest destructor body, whose code references ShipCombatProxiedChest::vftable and ShipCombatProx… |
| `00cacfa0` | `ShipCombatProxiedChest::Ctor` | ✅ | ctor | Initializes proxied structure fields, stores chest parameters, sets subtype 2, installs chest vtable, and clears remaining state. |
| `00cad030` | `ShipCombatProxiedChest::Dtor ⚠downgrade` | 🟢 | dtor | Installs the chest vtable, runs virtual resource cleanup through its owner, and releases remaining chest state. |
| `00cad590` | `ShipCombatProxiedChest::ScalarDeletingDtor` | ✅ | dtor | Calls chest destructor and frees the object when the delete flag's low bit is set. |
| `00cad5b0` | `ShipCombatProxiedChest::Update` | ✅ | virtual | Compares chest state flags, creates or clears effect state on transitions, and dispatches state-specific effect helpers. |
