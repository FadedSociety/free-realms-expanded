# Class: ShipCombatPacketPlaySound

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3f10` | `ShipCombatPacketPlaySound::Ctor` | ✅ | ctor | Calls the base packet constructor with sub-opcode 0x19, installs the PlaySound vtable, clears two fields, and initializes two sound words. |
| `00ca3f90` | `ShipCombatPacketPlaySound::ScalarDeletingDtor` | ✅ | dtor | Restores the packet base vtables, frees the packet allocation when the low deleting flag is set, and otherwise returns it. |
