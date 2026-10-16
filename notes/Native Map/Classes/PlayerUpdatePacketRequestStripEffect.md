# Class: PlayerUpdatePacketRequestStripEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0fb0` | `PlayerUpdatePacketRequestStripEffect::PlayerUpdatePacketRequestStripEffect` | ✅ | ctor | Sub-opcode-33 constructor: calls FUN_0089fbb0(0x21) so the packet header carries opcode 35 and sub-opcode 33, then installs PlayerUpdatePacketRequestStripEffect::vftable. |
| `008a1010` | `PlayerUpdatePacketRequestStripEffect::~PlayerUpdatePacketRequestStripEffect` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; its only caller is EH unwind funclet 01591880. |
| `008b4cd0` | `PlayerUpdatePacketRequestStripEffect::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
