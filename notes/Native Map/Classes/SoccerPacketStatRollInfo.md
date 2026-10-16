# Class: SoccerPacketStatRollInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b745e0` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x11 (17) packet SoccerPacketStatRollInfo: calls the shared base ctor FUN_00b730a0(0x11) and installs SoccerPacketStatRollInfo::vftable. |
| `00b74640` | `SoccerPacketStatRollInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
