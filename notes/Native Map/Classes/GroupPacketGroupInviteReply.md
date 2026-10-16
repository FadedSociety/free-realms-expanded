# Class: GroupPacketGroupInviteReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3700` | `GroupPacketGroupInviteReply::GroupPacketGroupInviteReply` | ✅ | ctor | Constructor: calls FUN_008a35c0(2) so the packet is group opcode 40 with sub-opcode 2, then installs GroupPacketGroupInviteReply::vftable. |
| `008a3760` | `GroupPacketGroupInviteReply::~GroupPacketGroupInviteReply` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; dispatcher 0093daf0 destroys the invite reply with it after it is serialized. |
| `008b7130` | `GroupPacketGroupInviteReply::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseGroupPacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
