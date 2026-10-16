# Class: SoccerPacketTeamColours

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b76ed0` | `SoccerPacketTeamColours::~SoccerPacketTeamColours` | 🟢 | dtor | Non-deleting destructor of the sub-8 TeamColours packet: runs the two member cleanup helpers FUN_004022b0 (the second one triggered by the EH state), then resets the vtable to BaseSoccerPacket::vftable and BasePacket::v… |
| `00b77ce0` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 8 packet SoccerPacketTeamColours: calls the shared base ctor FUN_00b730a0(8), installs SoccerPacketTeamColours::vftable and runs two member initialisation helpers (FUN_00402a10, the seco… |
| `00b77d50` | `SoccerPacketTeamColours::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the class destructor body FUN_00b76ed0 (two member cleanup calls plus vtable resets) and, when the flag byte's low bit is set, frees the object with _free. |
