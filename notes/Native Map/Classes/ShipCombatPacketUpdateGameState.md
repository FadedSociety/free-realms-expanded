# Class: ShipCombatPacketUpdateGameState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4ed30` | `ShipCombatPacketUpdateGameState::ShipCombatPacketUpdateGameState` | ✅ | ctor | Constructor: base-initialises with sub-opcode 2, stores ShipCombatPacketUpdateGameState::vftable and zeroes the trailing field +0xc. |
| `00a4edb0` | `ShipCombatPacketUpdateGameState::~ShipCombatPacketUpdateGameState` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor at 00a4ed30 through the shared caller 00a5ed80 and called from its EH unwind funclet. |
| `00a51d50` | `ShipCombatPacketUpdateGameState::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and frees the packet through the _free thunk when the deletion flag is set. |
