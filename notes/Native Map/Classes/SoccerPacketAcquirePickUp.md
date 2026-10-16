# Class: SoccerPacketAcquirePickUp

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74ee0` | `SoccerPacketAcquirePickUp::Ctor` | ✅ | ctor | Constructor of the family-76 sub 0xa (10) packet SoccerPacketAcquirePickUp: calls the shared base ctor FUN_00b730a0(10) and installs SoccerPacketAcquirePickUp::vftable. |
| `00b74f40` | `SoccerPacketAcquirePickUp::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
