# Class: PlayerUpdatePacketUpdateActiveWieldType

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a06f0` | `PlayerUpdatePacketUpdateActiveWieldType::~PlayerUpdatePacketUpdateActiveWieldTy…` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateActiveWieldType::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0750` | `PlayerUpdatePacketUpdateActiveWieldType::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the wield-type, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3e80` | `PlayerUpdatePacketUpdateActiveWieldType::PlayerUpdatePacketUpdateActiveWieldType` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x3d) and installs PlayerUpdatePacketUpdateActiveWieldType::vftable; nothing else is written. |
