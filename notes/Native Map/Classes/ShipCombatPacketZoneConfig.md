# Class: ShipCombatPacketZoneConfig

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f1d0` | `ShipCombatPacketZoneConfig::ShipCombatPacketZoneConfig` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0xc (12), stores ShipCombatPacketZoneConfig::vftable, then runs the zone-configuration helper FUN_01067ac0 on the packet (config member construction). |
| `00a4f240` | `ShipCombatPacketZoneConfig::~ShipCombatPacketZoneConfig` | 🟢 | dtor | Destructor body: runs the zone-config member teardown FUN_01067f20 and then resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4f1d0 through caller 00a6cdf0. |
| `00a522a0` | `ShipCombatPacketZoneConfig::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): runs the ZoneConfig destructor body 00a4f240 (which tears down the config member and resets the base vtables) and frees the packet through the _free thunk when the deletion fl… |
