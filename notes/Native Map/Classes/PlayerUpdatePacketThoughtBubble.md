# Class: PlayerUpdatePacketThoughtBubble

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0db0` | `PlayerUpdatePacketThoughtBubble::~PlayerUpdatePacketThoughtBubble` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1b and EH funclet 015916bf. |
| `008b4870` | `PlayerUpdatePacketThoughtBubble::PlayerUpdatePacketThoughtBubble ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x1b), installs PlayerUpdatePacketThoughtBubble::vftable, stores param_2..param_5 into +0x10..+0x1c and the byte param_6 at +0x20. |
| `008b4910` | `PlayerUpdatePacketThoughtBubble::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
