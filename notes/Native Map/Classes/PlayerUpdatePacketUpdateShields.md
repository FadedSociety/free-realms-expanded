# Class: PlayerUpdatePacketUpdateShields

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0550` | `PlayerUpdatePacketUpdateShields::~PlayerUpdatePacketUpdateShields` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateShields::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a05b0` | `PlayerUpdatePacketUpdateShields::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the shields, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3dc0` | `PlayerUpdatePacketUpdateShields::PlayerUpdatePacketUpdateShields` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x39) and installs PlayerUpdatePacketUpdateShields::vftable; nothing else is written. |
