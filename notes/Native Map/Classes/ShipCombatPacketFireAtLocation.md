# Class: ShipCombatPacketFireAtLocation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca2dc0` | `ShipCombatPacketFireAtLocation::Ctor` | ✅ | ctor | Calls the common ship-combat packet base constructor with sub-opcode 0x14, installs the FireAtLocation vtable, initializes four default words, and clears one field. |
| `00ca3d10` | `ShipCombatPacketFireAtLocation::ScalarDeletingDtor` | ✅ | dtor | Restores the base packet vtables, frees the allocation if the low deleting flag is set, and returns the pointer otherwise. |
