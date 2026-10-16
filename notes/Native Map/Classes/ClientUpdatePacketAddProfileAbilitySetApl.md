# Class: ClientUpdatePacketAddProfileAbilitySetApl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089dfc0` | `ClientUpdatePacketAddProfileAbilitySetApl::ClientUpdatePacketAddProfileAbilityS…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0xf) (family 0x26, sub-opcode 15), installs ClientUpdatePacketAddProfileAbilitySetApl::vftable and clears the two dwords at param_1[3] and param_… |
| `0089e020` | `ClientUpdatePacketAddProfileAbilitySetApl::~ClientUpdatePacketAddProfileAbility…` | ✅ | dtor | Destructor body: writes ClientUpdatePacketAddProfileAbilitySetApl::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns; the two payload dwords at param_1[3]/[4] need no re… |
| `0089e080` | `ClientUpdatePacketAddProfileAbilitySetApl::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-15 packet: restores ClientUpdatePacketAddProfileAbilitySetApl::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0 of… |
