# Class: PlayerUpdatePacketUpdateLivesRemaining

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0620` | `PlayerUpdatePacketUpdateLivesRemaining::~PlayerUpdatePacketUpdateLivesRemaining` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateLivesRemaining::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0680` | `PlayerUpdatePacketUpdateLivesRemaining::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the lives-remaining, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3e20` | `PlayerUpdatePacketUpdateLivesRemaining::PlayerUpdatePacketUpdateLivesRemaining` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x12) and installs PlayerUpdatePacketUpdateLivesRemaining::vftable; nothing else is written. |
