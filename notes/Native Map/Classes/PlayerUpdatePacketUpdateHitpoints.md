# Class: PlayerUpdatePacketUpdateHitpoints

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a03b0` | `PlayerUpdatePacketUpdateHitpoints::~PlayerUpdatePacketUpdateHitpoints` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateHitpoints::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0410` | `PlayerUpdatePacketUpdateHitpoints::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the hit-point, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3d00` | `PlayerUpdatePacketUpdateHitpoints::PlayerUpdatePacketUpdateHitpoints` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(5) and installs PlayerUpdatePacketUpdateHitpoints::vftable; nothing else is written. |
