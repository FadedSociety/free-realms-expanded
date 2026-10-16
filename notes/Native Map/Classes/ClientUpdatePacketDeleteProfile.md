# Class: ClientUpdatePacketDeleteProfile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e220` | `ClientUpdatePacketDeleteProfile::ClientUpdatePacketDeleteProfile` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x14) (family 0x26, sub-opcode 20), installs ClientUpdatePacketDeleteProfile::vftable and clears the dword at param_1[3]. |
| `0089e280` | `ClientUpdatePacketDeleteProfile::~ClientUpdatePacketDeleteProfile` | ✅ | dtor | Destructor body: writes ClientUpdatePacketDeleteProfile::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e2e0` | `ClientUpdatePacketDeleteProfile::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-20 packet: restores ClientUpdatePacketDeleteProfile::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of param_2 i… |
