# Class: PlayerUpdatePacketSetAnimation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a07c0` | `PlayerUpdatePacketSetAnimation::~PlayerUpdatePacketSetAnimation` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; no fields, no deallocation. Called from the opcode-35 dispatcher case 8 and from EH funclet 01591559. |
| `008b3ee0` | `PlayerUpdatePacketSetAnimation::PlayerUpdatePacketSetAnimation` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(8) and installs PlayerUpdatePacketSetAnimation::vftable; nothing else is written. |
| `008b3f40` | `PlayerUpdatePacketSetAnimation::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
