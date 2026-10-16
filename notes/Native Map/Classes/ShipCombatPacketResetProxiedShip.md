# Class: ShipCombatPacketResetProxiedShip

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca3870` | `ShipCombatPacketResetProxiedShip::Ctor` | ✅ | ctor | Calls the shared packet base constructor with sub-opcode 0xb, installs the ResetProxiedShip vtable, initializes two identifier words, and clears the final field. |
| `00ca3910` | `ShipCombatPacketResetProxiedShip::ScalarDeletingDtor` | ✅ | dtor | Restores the BaseShipCombatPacket and BasePacket vtables, frees the packet allocation when the low deleting flag is set, and returns the object otherwise. |
