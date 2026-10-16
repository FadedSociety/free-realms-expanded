# Class: PlayerUpdatePacketUpdatePosition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089fe30` | `PlayerUpdatePacketUpdatePosition::~PlayerUpdatePacketUpdatePosition` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketUpdatePosition::vftable then BasePacket::vftable; no fields touched, no deallocation. |
| `0089ff00` | `PlayerUpdatePacketUpdatePosition::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores PlayerUpdatePacketUpdatePosition and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3910` | `— ⚠reject` | ⚪ | ctor | Stores BasePacket::vftable, writes packet id 0x7d (125) into +4, installs PlayerUpdatePacketUpdatePosition::vftable, copies the global pair DAT_01be4850/DAT_01be4854 into +8/+0xc, copies the 8 dwords of the global block… |
