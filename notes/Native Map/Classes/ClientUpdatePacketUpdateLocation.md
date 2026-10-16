# Class: ClientUpdatePacketUpdateLocation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089dd60` | `ClientUpdatePacketUpdateLocation::ClientUpdatePacketUpdateLocation` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0xc) (family 0x26, sub-opcode 12), installs ClientUpdatePacketUpdateLocation::vftable and clears the byte at +0x31. The position/rotation block i… |
| `0089ddc0` | `ClientUpdatePacketUpdateLocation::~ClientUpdatePacketUpdateLocation` | ✅ | dtor | Destructor body: writes ClientUpdatePacketUpdateLocation::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns; no field is released. |
| `0089de20` | `ClientUpdatePacketUpdateLocation::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-12 location packet: restores ClientUpdatePacketUpdateLocation::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of… |
