# Class: PlayerUpdatePacketHeadInflationScale

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0e60` | `PlayerUpdatePacketHeadInflationScale::~PlayerUpdatePacketHeadInflationScale` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x1e and EH funclet 01591464. |
| `008b4a80` | `PlayerUpdatePacketHeadInflationScale::PlayerUpdatePacketHeadInflationScale ⚠dow…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x1e), installs PlayerUpdatePacketHeadInflationScale::vftable and stores param_2, param_3 and param_4 into +0x10, +0x14 and +0x18. |
| `008b4b00` | `PlayerUpdatePacketHeadInflationScale::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
