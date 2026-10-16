# Class: PlayerUpdatePacketSeekTarget

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1750` | `PlayerUpdatePacketSeekTarget::~PlayerUpdatePacketSeekTarget` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3b and also runs from EH funclet 015917d5. |
| `008b5730` | `PlayerUpdatePacketSeekTarget::PlayerUpdatePacketSeekTarget ? ⚠downgrade` | 🟡 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x3b), installs PlayerUpdatePacketSeekTarget::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and again into +0x18/+0x1c, and zeroes +0x20, +0x2… |
| `008b57e0` | `PlayerUpdatePacketSeekTarget::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
