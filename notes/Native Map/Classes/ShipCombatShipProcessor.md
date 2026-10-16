# Class: ShipCombatShipProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f780` | `ShipCombatShipProcessor::ScalarDeletingDtor` | 🟢 | dtor | MSVC scalar deleting destructor: calls the class destructor body at 00ca4d40 (whose own code references ShipCombatShipProcessor::vftable and SoeUtil::List<ShipCombatProxiedShip>) with this unchanged in ecx, then frees t… |
| `00ca3270` | `— ⚠reject` | ⚪ | virtual | When both pointer arguments and the nested +0x14 object exist, calls 00fefa30 with three values and clears receiver field +0x2d0. |
| `00ca4ca0` | `ShipCombatShipProcessor::Ctor` | ✅ | ctor | Installs processor and embedded list vtables, zeros state and pointer fields, stores one constructor argument, then binds six pirate battle audio events. |
| `00ca4d40` | `ShipCombatShipProcessor::Dtor` | ✅ | dtor | Restores the processor vtable, releases six shared references, restores the embedded list vtable, then drains and destroys the proxied ship nodes. |
