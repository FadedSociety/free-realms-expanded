# Class: PlayerUpdatePacketItemDefinitions

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1210` | `PlayerUpdatePacketItemDefinitions::PlayerUpdatePacketItemDefinitions` | ✅ | ctor | Sub-opcode-37 constructor: calls FUN_0089fbb0(0x25) (opcode 35, sub-opcode 37), installs PlayerUpdatePacketItemDefinitions::vftable and zeroes param_1[3] and param_1[4]. |
| `008a1270` | `PlayerUpdatePacketItemDefinitions::~PlayerUpdatePacketItemDefinitions` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x25 and also runs from EH funclet 0159170f. |
| `008b5030` | `PlayerUpdatePacketItemDefinitions::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
