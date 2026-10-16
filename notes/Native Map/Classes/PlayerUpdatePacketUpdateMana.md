# Class: PlayerUpdatePacketUpdateMana

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0480` | `PlayerUpdatePacketUpdateMana::~PlayerUpdatePacketUpdateMana` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateMana::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a04e0` | `PlayerUpdatePacketUpdateMana::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the mana, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3d60` | `PlayerUpdatePacketUpdateMana::PlayerUpdatePacketUpdateMana` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(9) and installs PlayerUpdatePacketUpdateMana::vftable; nothing else is written. |
