# Class: ShipCombatPacketRefreshUI

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4eeb0` | `ShipCombatPacketRefreshUI::ShipCombatPacketRefreshUI` | ✅ | ctor | Constructor: base-initialises with sub-opcode 4 and stores ShipCombatPacketRefreshUI::vftable. |
| `00a4ef20` | `ShipCombatPacketRefreshUI::~ShipCombatPacketRefreshUI` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a4eeb0 through caller 00a655a0. |
| `00a51e30` | `ShipCombatPacketRefreshUI::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
