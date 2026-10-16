# Class: ClientUpdatePacketRemoveEffectTag

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e540` | `ClientUpdatePacketRemoveEffectTag::ClientUpdatePacketRemoveEffectTag` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x11) (family 0x26, sub-opcode 17), installs ClientUpdatePacketRemoveEffectTag::vftable and clears the payload dword at param_1[3]. |
| `0089e5a0` | `ClientUpdatePacketRemoveEffectTag::~ClientUpdatePacketRemoveEffectTag` | ✅ | dtor | Destructor body: writes ClientUpdatePacketRemoveEffectTag::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e600` | `ClientUpdatePacketRemoveEffectTag::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-17 packet: restores ClientUpdatePacketRemoveEffectTag::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of param_2… |
