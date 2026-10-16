# Class: PlayerUpdatePacketUpdateWalkAnim

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0c70` | `PlayerUpdatePacketUpdateWalkAnim::~PlayerUpdatePacketUpdateWalkAnim` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x15 and EH funclet 01591693. |
| `008b4570` | `PlayerUpdatePacketUpdateWalkAnim::PlayerUpdatePacketUpdateWalkAnim` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x15), installs PlayerUpdatePacketUpdateWalkAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +0x14 and +0x18. |
| `008b4600` | `PlayerUpdatePacketUpdateWalkAnim::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
