# Class: GroupPacketGroupKickReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3860` | `GroupPacketGroupKickReply::GroupPacketGroupKickReply` | 🟢 | ctor | Constructor: calls FUN_008a35c0(7) so the packet is group opcode 40 with sub-opcode 7, then installs GroupPacketGroupKickReply::vftable. |
| `008a38c0` | `GroupPacketGroupKickReply::~GroupPacketGroupKickReply` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the kick reply with it. |
| `008b7210` | `GroupPacketGroupKickReply::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
