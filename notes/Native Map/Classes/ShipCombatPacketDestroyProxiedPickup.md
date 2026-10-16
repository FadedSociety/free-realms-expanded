# Class: ShipCombatPacketDestroyProxiedPickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f370` | `ShipCombatPacketDestroyProxiedPickup::~ShipCombatPacketDestroyProxiedPickup` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a52400 (sub-opcode 0x1d) through caller 00a5faa0. |
| `00a52400` | `ShipCombatPacketDestroyProxiedPickup::ShipCombatPacketDestroyProxiedPickup` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x1d (29), stores ShipCombatPacketDestroyProxiedPickup::vftable, seeds +0xc from the static default at 01be6e80, zeroes the byte at +0x14 and the dwords +0x10/+0x18, and ini… |
| `00a52490` | `ShipCombatPacketDestroyProxiedPickup::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
