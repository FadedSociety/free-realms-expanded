# Class: PlayerUpdatePacketFreeze

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0f00` | `PlayerUpdatePacketFreeze::PlayerUpdatePacketFreeze` | ✅ | ctor | Sub-opcode-32 constructor: calls the family constructor FUN_0089fbb0(0x20), which writes opcode 35 and sub-opcode 32 into the header slots, then installs PlayerUpdatePacketFreeze::vftable inside an SEH frame. |
| `008a0f60` | `PlayerUpdatePacketFreeze::~PlayerUpdatePacketFreeze` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x20 and also runs from EH funclet 015916eb. |
| `008b4c60` | `PlayerUpdatePacketFreeze::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
