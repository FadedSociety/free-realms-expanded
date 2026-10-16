# Class: GroupPacketGroupAcceptReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a37b0` | `GroupPacketGroupAcceptReply::GroupPacketGroupAcceptReply` | ✅ | ctor | Constructor: calls FUN_008a35c0(5) so the packet is group opcode 40 with sub-opcode 5, then installs GroupPacketGroupAcceptReply::vftable. |
| `008a3810` | `GroupPacketGroupAcceptReply::~GroupPacketGroupAcceptReply` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the accept reply with it. |
| `008b71a0` | `GroupPacketGroupAcceptReply::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
