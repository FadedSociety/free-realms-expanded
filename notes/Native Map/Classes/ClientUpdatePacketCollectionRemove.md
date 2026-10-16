# Class: ClientUpdatePacketCollectionRemove

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089da00` | `ClientUpdatePacketCollectionRemove::ClientUpdatePacketCollectionRemove` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(9), which installs BasePacket::vftable, writes family opcode 0x26 into param_1[1] and the sub-opcode 9 into param_1[2], then installs ClientUpdat… |
| `0089da60` | `ClientUpdatePacketCollectionRemove::~ClientUpdatePacketCollectionRemove` | ✅ | dtor | Destructor body: writes ClientUpdatePacketCollectionRemove::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. No field is freed and no other call is made. |
| `0089dac0` | `ClientUpdatePacketCollectionRemove::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-9 collection-remove packet: writes ClientUpdatePacketCollectionRemove::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable in turn, then calls _free(thi… |
