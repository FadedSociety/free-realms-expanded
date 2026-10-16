# Class: ClientUpdatePacketUpdateProfileAbilitySetApl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e0f0` | `ClientUpdatePacketUpdateProfileAbilitySetApl::ClientUpdatePacketUpdateProfileAb…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x18) (family 0x26, sub-opcode 24), installs ClientUpdatePacketUpdateProfileAbilitySetApl::vftable and clears the two dwords at param_1[3] and pa… |
| `0089e150` | `ClientUpdatePacketUpdateProfileAbilitySetApl::~ClientUpdatePacketUpdateProfileA…` | ✅ | dtor | Destructor body: writes ClientUpdatePacketUpdateProfileAbilitySetApl::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e1b0` | `ClientUpdatePacketUpdateProfileAbilitySetApl::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor for the family-38 sub-24 packet: restores ClientUpdatePacketUpdateProfileAbilitySetApl::vftable, BaseClientUpdatePacket::vftable and BasePacket::vftable, then calls _free(this) when bit 0… |
