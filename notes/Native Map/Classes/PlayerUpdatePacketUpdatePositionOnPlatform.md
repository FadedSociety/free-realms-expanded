# Class: PlayerUpdatePacketUpdatePositionOnPlatform

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089ff70` | `PlayerUpdatePacketUpdatePositionOnPlatform::~PlayerUpdatePacketUpdatePositionOn…` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdatePositionOnPlatform::vftable, then PlayerUpdatePacketUpdatePosition::vftable, then BasePacket::vftable. |
| `0089ffd0` | `PlayerUpdatePacketUpdatePositionOnPlatform::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the OnPlatform, UpdatePosition and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `008b3a30` | `PlayerUpdatePacketUpdatePositionOnPlatform::PlayerUpdatePacketUpdatePositionOnP…` | ✅ | ctor | Runs the position-packet constructor 008b3910, installs PlayerUpdatePacketUpdatePositionOnPlatform::vftable over it, stores the global pair DAT_01be4850/DAT_01be4854 into +0x40/+0x44 and overwrites the packet id at +4 w… |
