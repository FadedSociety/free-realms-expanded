# Class: PlayerUpdatePacketQueueAnimation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0860` | `PlayerUpdatePacketQueueAnimation::~PlayerUpdatePacketQueueAnimation` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x16 and EH funclet 01591585. |
| `008b4090` | `PlayerUpdatePacketQueueAnimation::PlayerUpdatePacketQueueAnimation ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x16), installs PlayerUpdatePacketQueueAnimation::vftable, zeroes +0x10..+0x1c, +0x24, +0x28 and the byte at +0x2c, copies the global float DAT_0175b420 into +0x20 and set… |
| `008b4120` | `PlayerUpdatePacketQueueAnimation::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
