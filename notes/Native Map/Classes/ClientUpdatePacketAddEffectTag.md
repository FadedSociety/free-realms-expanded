# Class: ClientUpdatePacketAddEffectTag

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e410` | `ClientUpdatePacketAddEffectTag::ClientUpdatePacketAddEffectTag` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x10) (family 0x26, sub-opcode 16), installs ClientUpdatePacketAddEffectTag::vftable and clears three payload dwords at param_1[3], param_1[4] an… |
| `0089e470` | `ClientUpdatePacketAddEffectTag::~ClientUpdatePacketAddEffectTag` | ✅ | dtor | Destructor body: writes ClientUpdatePacketAddEffectTag::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e4d0` | `ClientUpdatePacketAddEffectTag::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-16 packet: restores ClientUpdatePacketAddEffectTag::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of param_2 is… |
