# Class: SoccerPacketUpdatePlayerPos

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74850` | `SoccerPacketUpdatePlayerPos::Ctor` | ✅ | ctor | Constructor of the family-76 sub 1 packet SoccerPacketUpdatePlayerPos: calls the shared base ctor FUN_00b730a0(1) and installs SoccerPacketUpdatePlayerPos::vftable. |
| `00b748b0` | `SoccerPacketUpdatePlayerPos::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
