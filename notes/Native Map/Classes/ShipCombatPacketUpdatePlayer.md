# Class: ShipCombatPacketUpdatePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f130` | `ShipCombatPacketUpdatePlayer::~ShipCombatPacketUpdatePlayer` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52080 (sub-opcode 8) through caller 00a67110. |
| `00a52080` | `ShipCombatPacketUpdatePlayer::ShipCombatPacketUpdatePlayer` | ✅ | ctor | Constructor: base-initialises with sub-opcode 8, stores ShipCombatPacketUpdatePlayer::vftable and initialises the two fields +0x10/+0x14 from the static data at 01be6e28/01be6e2c. |
| `00a52110` | `ShipCombatPacketUpdatePlayer::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
