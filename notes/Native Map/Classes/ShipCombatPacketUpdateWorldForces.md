# Class: ShipCombatPacketUpdateWorldForces

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3bc0` | `ShipCombatPacketUpdateWorldForces::Ctor` | ✅ | ctor | Calls the shared base packet constructor with sub-opcode 0x12, installs the UpdateWorldForces vtable, initializes default vector words and clears trailing fields. |
| `00ca3ca0` | `ShipCombatPacketUpdateWorldForces::ScalarDeletingDtor` | ✅ | dtor | Restores the base packet vtables, conditionally frees the packet when the low deleting flag is set, and returns it otherwise. |
