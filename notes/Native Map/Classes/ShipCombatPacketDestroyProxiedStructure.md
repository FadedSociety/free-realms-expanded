# Class: ShipCombatPacketDestroyProxiedStructure

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f3c0` | `ShipCombatPacketDestroyProxiedStructure::~ShipCombatPacketDestroyProxiedStructu…` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52520 (sub-opcode 0x20) through caller 00a5fe30. |
| `00a52520` | `ShipCombatPacketDestroyProxiedStructure::ShipCombatPacketDestroyProxiedStructure` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x20 (32), stores ShipCombatPacketDestroyProxiedStructure::vftable and seeds the field +0xc from the static default at 01be6e18. |
| `00a52590` | `ShipCombatPacketDestroyProxiedStructure::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
