# Class: ShipCombatPacketAnnouncerMessage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a4f460` | `ShipCombatPacketAnnouncerMessage::~ShipCombatPacketAnnouncerMessage` | 🟢 | dtor | Destructor body: resets the packet vtable to BaseShipCombatPacket then BasePacket; paired with the constructor 00a526f0 (sub-opcode 0x22) through caller 00a62280, the announcer handler whose strings mention 'speaker'/'l… |
| `00a526f0` | `ShipCombatPacketAnnouncerMessage::ShipCombatPacketAnnouncerMessage` | ✅ | ctor | Constructor: base-initialises with sub-opcode 0x22 (34) and stores ShipCombatPacketAnnouncerMessage::vftable. |
| `00a52770` | `ShipCombatPacketAnnouncerMessage::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (vtable slot 0): resets the vtable to BaseShipCombatPacket then BasePacket and conditionally frees the object. |
