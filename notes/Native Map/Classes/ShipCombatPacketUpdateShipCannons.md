# Class: ShipCombatPacketUpdateShipCannons

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f2a0` | `ShipCombatPacketUpdateShipCannons::~ShipCombatPacketUpdateShipCannons` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a522c0 (sub-opcode 0x13) through caller 00a5ee90. |
| `00a522c0` | `ShipCombatPacketUpdateShipCannons::ShipCombatPacketUpdateShipCannons` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x13 (19), stores ShipCombatPacketUpdateShipCannons::vftable, initialises +0x10/+0x14 from static defaults and zeroes the eight further fields +0x18..+0x34 (the cannon-updat… |
| `00a52360` | `ShipCombatPacketUpdateShipCannons::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
