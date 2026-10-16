# Class: ShipCombatPacketUpdatePlayerGameStates

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4ee00` | `ShipCombatPacketUpdatePlayerGameStates::ShipCombatPacketUpdatePlayerGameStates` | ✅ | ctor | Constructor: base-initialises with sub-opcode 3 and stores ShipCombatPacketUpdatePlayerGameStates::vftable. |
| `00a4ee60` | `ShipCombatPacketUpdatePlayerGameStates::~ShipCombatPacketUpdatePlayerGameStates` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4ee00 through caller 00a69480. |
| `00a51dc0` | `ShipCombatPacketUpdatePlayerGameStates::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
