# Class: PlayerUpdatePacketClearMovementRail

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a15f0` | `PlayerUpdatePacketClearMovementRail::~PlayerUpdatePacketClearMovementRail` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x36 and also runs from EH funclet 015917a9. |
| `008b54f0` | `PlayerUpdatePacketClearMovementRail::PlayerUpdatePacketClearMovementRail ⚠downg…` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x36), installs PlayerUpdatePacketClearMovementRail::vftable and stores param_2 and param_3 into +0x10 and +0x14. |
| `008b5560` | `PlayerUpdatePacketClearMovementRail::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
