# Class: ShipCombatPacketShipHit

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3df0` | `ShipCombatPacketShipHit::Ctor` | ✅ | ctor | Calls the shared base constructor with sub-opcode 0x17, installs the ShipHit vtable, and clears three payload fields. |
| `00ca3ea0` | `ShipCombatPacketShipHit::ScalarDeletingDtor` | ✅ | dtor | Restores the base packet vtables, frees the packet allocation when the low deleting flag is set, and returns it otherwise. |
