# Class: PlayerUpdatePacketUpdateIdleAnim

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0d30` | `PlayerUpdatePacketUpdateIdleAnim::~PlayerUpdatePacketUpdateIdleAnim` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1a and EH funclet 015916a9. |
| `008b4770` | `PlayerUpdatePacketUpdateIdleAnim::PlayerUpdatePacketUpdateIdleAnim` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x1a), installs PlayerUpdatePacketUpdateIdleAnim::vftable and stores param_2, param_3 and param_4 into +0x10, +0x14 and +0x18. |
| `008b4800` | `PlayerUpdatePacketUpdateIdleAnim::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
