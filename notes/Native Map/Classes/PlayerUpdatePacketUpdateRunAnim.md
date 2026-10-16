# Class: PlayerUpdatePacketUpdateRunAnim

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0cd0` | `PlayerUpdatePacketUpdateRunAnim::~PlayerUpdatePacketUpdateRunAnim` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x19 and EH funclet 0159169e. |
| `008b4670` | `PlayerUpdatePacketUpdateRunAnim::PlayerUpdatePacketUpdateRunAnim` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x19), installs PlayerUpdatePacketUpdateRunAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +0x14 and +0x18. |
| `008b4700` | `PlayerUpdatePacketUpdateRunAnim::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
