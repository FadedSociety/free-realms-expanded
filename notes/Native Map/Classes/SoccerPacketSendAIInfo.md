# Class: SoccerPacketSendAIInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b76d50` | `SoccerPacketSendAIInfo::~SoccerPacketSendAIInfo` | 🟢 | dtor | Non-deleting destructor of the sub-0x18 SendAIInfo packet: calls the member cleanup helper FUN_00401fd0, then resets the vtable to BaseSoccerPacket::vftable and BasePacket::vftable. |
| `00b77c40` | `— ⚠reject` | ⚪ | ctor | Constructor of the family-76 sub 0x18 (24) packet SoccerPacketSendAIInfo: calls the shared base ctor FUN_00b730a0(0x18), installs SoccerPacketSendAIInfo::vftable and runs a member initialisation helper FUN_00402950. |
| `00b77ca0` | `SoccerPacketSendAIInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the class destructor body FUN_00b76d50 (member cleanup plus vtable resets) and, when the flag byte's low bit is set, frees the object with _free. |
