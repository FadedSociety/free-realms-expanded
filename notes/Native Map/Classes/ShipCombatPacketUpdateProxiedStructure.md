# Class: ShipCombatPacketUpdateProxiedStructure

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f410` | `ShipCombatPacketUpdateProxiedStructure::~ShipCombatPacketUpdateProxiedStructure` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52600 (sub-opcode 0x21) through caller 00a5ff90. |
| `00a52600` | `ShipCombatPacketUpdateProxiedStructure::ShipCombatPacketUpdateProxiedStructure` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x21 (33), stores ShipCombatPacketUpdateProxiedStructure::vftable, seeds +0xc from the static default at 01be6e18 and zeroes +0x10. |
| `00a52680` | `ShipCombatPacketUpdateProxiedStructure::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
