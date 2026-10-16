# Class: PlayerUpdatePacketItemDefinitionRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1100` | `PlayerUpdatePacketItemDefinitionRequest::PlayerUpdatePacketItemDefinitionRequest` | ✅ | ctor | Sub-opcode-34 constructor taking an id: calls FUN_0089fbb0(0x22) (opcode 35, sub-opcode 34), zeroes param_1[3] and param_1[4], installs PlayerUpdatePacketItemDefinitionRequest::vftable and stores the id argument in para… |
| `008a1170` | `PlayerUpdatePacketItemDefinitionRequest::~PlayerUpdatePacketItemDefinitionReque…` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x22 and is also called by the request senders 00a3ddd0 and 00a799d0. |
| `008b4ee0` | `PlayerUpdatePacketItemDefinitionRequest::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
