# Class: GroupPacketGroupInvite

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d15b0` | `GroupPacketGroupInvite::~GroupPacketGroupInvite` | 🟢 | dtor | Destructor body of the group invite packet: runs the shared payload teardown FUN_008d1550 (members at +0x98 and +8), then rewrites the object vtable to BaseGroupPacket::vftable and BasePacket::vftable inside an SEH cook… |
| `008dce80` | `GroupPacketGroupInvite::GroupPacketGroupInvite` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 1 (family 40), installs GroupPacketGroupInvite::vftable, calls 008dce10 with ECX=this+0x10, then zeroes five words at +0x138..+0x148. |
| `008dcf10` | `GroupPacketGroupInvite::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d15b0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008eb6f0` | `GroupPacketGroupInvite::CopyInviteMemberPayload ? ⚠rename` | 🟡 | method | Copies two values, three strings and three trailing values into packet object. |
| `008eb750` | `GroupPacketGroupInvite::CopyOtherMemberPayload ? ⚠rename` | 🟡 | method | Copies two values, three strings and trailing values into offsets +0x124 through +0x12c. |
