# Class: ShipCombatPacketDestroyProxiedShip

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f180` | `ShipCombatPacketDestroyProxiedShip::~ShipCombatPacketDestroyProxiedShip` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a521b0 (sub-opcode 10) through caller 00a5f120 (the 00a5f212 exit path is the same handler). |
| `00a521b0` | `ShipCombatPacketDestroyProxiedShip::ShipCombatPacketDestroyProxiedShip` | ✅ | ctor | Constructor: base-initialises with sub-opcode 10 and stores ShipCombatPacketDestroyProxiedShip::vftable. |
| `00a52230` | `ShipCombatPacketDestroyProxiedShip::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
