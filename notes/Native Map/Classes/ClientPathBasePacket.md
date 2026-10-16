# Class: ClientPathBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4800` | `ClientPathBasePacket::ClientPathBasePacket` | ✅ | ctor | Client-path family constructor with a zero sub-opcode: installs BasePacket::vftable, stores opcode 98 (0x62) at +4, installs ClientPathBasePacket::vftable and zeroes +8; no calls. |
| `008a4860` | `ClientPathBasePacket::ClientPathBasePacketSub` | ✅ | ctor | Client-path family constructor taking the sub-opcode: installs BasePacket::vftable, stores opcode 98 (0x62) at +4, installs ClientPathBasePacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a48d0` | `ClientPathBasePacket::~ClientPathBasePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; the 009471a0 router destroys the packet with it at the end of case 0x62, and EH funclets around 0157de00 also call it. |
| `008b7d90` | `ClientPathBasePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
