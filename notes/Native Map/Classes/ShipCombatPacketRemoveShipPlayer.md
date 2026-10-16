# Class: ShipCombatPacketRemoveShipPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f020` | `ShipCombatPacketRemoveShipPlayer::~ShipCombatPacketRemoveShipPlayer` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired through shared caller 00a6cbd0 with the constructor 00a51f10 (sub-opcode 6) that lives later in this shard. |
| `00a51f10` | `ShipCombatPacketRemoveShipPlayer::ShipCombatPacketRemoveShipPlayer` | ✅ | ctor | Constructor: base-initialises with sub-opcode 6, stores ShipCombatPacketRemoveShipPlayer::vftable and stores the two caller arguments into the fields +0x10 and +0x14 (the removed player's identifiers). |
| `00a51fa0` | `ShipCombatPacketRemoveShipPlayer::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
