# Class: SoccerPacketPlayerVariableStats

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74c00` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x13 (19) packet SoccerPacketPlayerVariableStats: calls the shared base ctor FUN_00b730a0(0x13) and installs SoccerPacketPlayerVariableStats::vftable. |
| `00b74c60` | `SoccerPacketPlayerVariableStats::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
