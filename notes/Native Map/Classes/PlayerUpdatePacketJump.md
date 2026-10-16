# Class: PlayerUpdatePacketJump

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0090` | `PlayerUpdatePacketJump::~PlayerUpdatePacketJump` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketJump::vftable then BasePacket::vftable. |
| `008a00f0` | `PlayerUpdatePacketJump::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores PlayerUpdatePacketJump and BasePacket vtables in an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3b70` | `— ⚠reject` | ⚪ | ctor | Stores BasePacket::vftable, writes packet id 0xa4 (164) into +4, installs PlayerUpdatePacketJump::vftable, copies the global pair DAT_01be4850/DAT_01be4854 into +8/+0xc, the two global blocks at 01cb54f0 and 01cb53d0 in… |
