# Class: GroupPacketGroupKick

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d16d0` | `GroupPacketGroupKick::~GroupPacketGroupKick` | 🟢 | dtor | Destructor body of the group kick packet: destroys two members through FUN_0070a790, the sub-object at +0xa8 first (SEH state stepped from 1 to 0) and then the member at +0x18, then rewrites the object vtable to BaseGro… |
| `008dd060` | `GroupPacketGroupKick::GroupPacketGroupKick` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 6, installs GroupPacketGroupKick::vftable, stores DAT_01be4850/DAT_01be4854 at +0x10/+0x14 and +0xa0/+0xa4, calls 0070bac0 with ECX=this+0x… |
| `008dd110` | `GroupPacketGroupKick::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d16d0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008eb8f0` | `GroupPacketGroupKick::CopyMemberPayload ? ⚠rename` | 🟡 | method | Copies two values, three strings and trailing values into offsets +0x94 through +0x9c. |
| `008eb950` | `GroupPacketGroupKick::CopyOtherMemberPayload ? ⚠rename` | 🟡 | method | Copies two values, three strings and trailing values into offsets +0x124 through +0x12c. |
