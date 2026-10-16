# Class: SoccerPacketUpdatePlayerControls

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b74920` | `SoccerPacketUpdatePlayerControls::Ctor` | ✅ | ctor | Constructor of the family-76 sub 0x10 (16) packet SoccerPacketUpdatePlayerControls: calls the shared base ctor FUN_00b730a0(0x10) and installs SoccerPacketUpdatePlayerControls::vftable. |
| `00b74980` | `SoccerPacketUpdatePlayerControls::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, frees the object with _free. |
