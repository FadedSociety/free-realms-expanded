# Class: SoccerPacketAcquireBall

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74fb0` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0xb (11) packet SoccerPacketAcquireBall: calls the shared base ctor FUN_00b730a0(0xb) and installs SoccerPacketAcquireBall::vftable. |
| `00b75010` | `SoccerPacketAcquireBall::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
