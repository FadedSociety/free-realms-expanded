# Class: SoccerPacketSpawnPickUp

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b75f80` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 9 packet SoccerPacketSpawnPickUp: calls the shared base ctor FUN_00b730a0(9) and installs SoccerPacketSpawnPickUp::vftable. |
| `00b75fe0` | `SoccerPacketSpawnPickUp::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
