# Class: PlayerUpdatePacketUpdateScale

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0900` | `PlayerUpdatePacketUpdateScale::~PlayerUpdatePacketUpdateScale` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdateScale::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a0960` | `PlayerUpdatePacketUpdateScale::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the scale, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b42b0` | `PlayerUpdatePacketUpdateScale::PlayerUpdatePacketUpdateScale` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0xd) and installs PlayerUpdatePacketUpdateScale::vftable; no field is initialised. |
