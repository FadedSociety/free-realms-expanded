# Class: ClientUpdatePacketUpdateProfileExperience

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089de90` | `ClientUpdatePacketUpdateProfileExperience::ClientUpdatePacketUpdateProfileExper…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0xe) (family 0x26, sub-opcode 14) and then installs ClientUpdatePacketUpdateProfileExperience::vftable; no payload field is initialised. |
| `0089def0` | `ClientUpdatePacketUpdateProfileExperience::~ClientUpdatePacketUpdateProfileExpe…` | ✅ | dtor | Destructor body: writes ClientUpdatePacketUpdateProfileExperience::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089df50` | `ClientUpdatePacketUpdateProfileExperience::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-14 packet: restores ClientUpdatePacketUpdateProfileExperience::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of… |
