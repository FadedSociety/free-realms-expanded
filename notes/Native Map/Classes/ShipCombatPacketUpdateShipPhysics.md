# Class: ShipCombatPacketUpdateShipPhysics

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3980` | `ShipCombatPacketUpdateShipPhysics::Ctor` | ✅ | ctor | Calls the base packet constructor with sub-opcode 0x10, installs the UpdateShipPhysics vtable, clears one byte, initializes a nested object and copies sixteen default words. |
| `00ca3a40` | `ShipCombatPacketUpdateShipPhysics::ScalarDeletingDtor` | ✅ | dtor | Restores the base packet vtables, frees the allocation when the low deleting flag is set, and otherwise returns the packet pointer. |
