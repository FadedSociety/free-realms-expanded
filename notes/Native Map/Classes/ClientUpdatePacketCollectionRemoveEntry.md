# Class: ClientUpdatePacketCollectionRemoveEntry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089dc30` | `ClientUpdatePacketCollectionRemoveEntry::ClientUpdatePacketCollectionRemoveEntry` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0xb) (family 0x26, sub-opcode 11) and then installs ClientUpdatePacketCollectionRemoveEntry::vftable. Nothing else is written. |
| `0089dc90` | `ClientUpdatePacketCollectionRemoveEntry::~ClientUpdatePacketCollectionRemoveEnt…` | ✅ | dtor | Destructor body: writes ClientUpdatePacketCollectionRemoveEntry::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089dcf0` | `ClientUpdatePacketCollectionRemoveEntry::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-11 packet: restores ClientUpdatePacketCollectionRemoveEntry::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of p… |
