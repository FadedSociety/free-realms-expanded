# Class: ShipCombatPacketSetPlayerCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f070` | `ShipCombatPacketSetPlayerCount::ShipCombatPacketSetPlayerCount` | ✅ | ctor | Constructor: base-initialises with sub-opcode 7, stores ShipCombatPacketSetPlayerCount::vftable and zeroes the fields +0xc and +0x10 (the player-count message payload). |
| `00a4f0e0` | `ShipCombatPacketSetPlayerCount::~ShipCombatPacketSetPlayerCount` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4f070 through caller 00a5f030. |
| `00a52010` | `ShipCombatPacketSetPlayerCount::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
