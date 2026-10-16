# Class: PlayerUpdatePacketSeekTargetUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1800` | `PlayerUpdatePacketSeekTargetUpdate::~PlayerUpdatePacketSeekTargetUpdate` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x3c and also runs from EH funclet 015917e0. |
| `008b5890` | `PlayerUpdatePacketSeekTargetUpdate::PlayerUpdatePacketSeekTargetUpdate ? ⚠downg…` | 🟡 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x3c), installs PlayerUpdatePacketSeekTargetUpdate::vftable and stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and again into +0x18/+0x1c. |
| `008b5910` | `PlayerUpdatePacketSeekTargetUpdate::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
