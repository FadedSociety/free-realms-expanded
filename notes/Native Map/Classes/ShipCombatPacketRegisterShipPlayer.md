# Class: ShipCombatPacketRegisterShipPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4ef70` | `ShipCombatPacketRegisterShipPlayer::ShipCombatPacketRegisterShipPlayer` | ✅ | ctor | Constructor: base-initialises with sub-opcode 5 and stores ShipCombatPacketRegisterShipPlayer::vftable. |
| `00a4efd0` | `ShipCombatPacketRegisterShipPlayer::~ShipCombatPacketRegisterShipPlayer` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4ef70 through caller 00a69380. |
| `00a51ea0` | `ShipCombatPacketRegisterShipPlayer::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
