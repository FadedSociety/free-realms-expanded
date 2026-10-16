# Class: SoccerPacketSpecialAnimTime

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74a60` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x16 (22) packet SoccerPacketSpecialAnimTime: calls the shared base ctor FUN_00b730a0(0x16) and installs SoccerPacketSpecialAnimTime::vftable. |
| `00b74ac0` | `SoccerPacketSpecialAnimTime::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
