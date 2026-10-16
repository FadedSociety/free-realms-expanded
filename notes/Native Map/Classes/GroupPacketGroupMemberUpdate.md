# Class: GroupPacketGroupMemberUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1790` | `GroupPacketGroupMemberUpdate::~GroupPacketGroupMemberUpdate` | 🟢 | dtor | Destructor body of the group member-update packet: destroys the member at +0x18 (reached as +8 of the sub-object at +0x10, with a zero byte stored into the SEH state) through FUN_0070a790, then rewrites the object vtabl… |
| `008dd130` | `GroupPacketGroupMemberUpdate::GroupPacketGroupMemberUpdate` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 9, installs GroupPacketGroupMemberUpdate::vftable and calls Ctor_GroupMemberRecord 008dcd60 with ECX=this+0x10. |
| `008dd190` | `GroupPacketGroupMemberUpdate::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d1790 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
