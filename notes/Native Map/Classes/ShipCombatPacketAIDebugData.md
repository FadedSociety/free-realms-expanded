# Class: ShipCombatPacketAIDebugData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f4c0` | `ShipCombatPacketAIDebugData::~ShipCombatPacketAIDebugData` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a527e0 (sub-opcode 0x24) through caller 00a65980. |
| `00a527e0` | `ShipCombatPacketAIDebugData::ShipCombatPacketAIDebugData` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x24 (36) and stores ShipCombatPacketAIDebugData::vftable. |
| `00a52860` | `ShipCombatPacketAIDebugData::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
