# Class: GroupPacketGroupLeave

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1670` | `GroupPacketGroupLeave::~GroupPacketGroupLeave` | 🟢 | dtor | Destructor body of the group leave packet: destroys the member at +0x18 through FUN_0070a790, then rewrites the object vtable to BaseGroupPacket::vftable and BasePacket::vftable inside an SEH cookie frame; the destroyed… |
| `008dcfe0` | `GroupPacketGroupLeave::GroupPacketGroupLeave` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 3, installs GroupPacketGroupLeave::vftable and calls embedded member initializer 0070bac0 with ECX=this+0x18. |
| `008dd040` | `GroupPacketGroupLeave::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d1670 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008eb890` | `GroupPacketGroupLeave::CopyPayloadAtOffsets94` | 🟢 | method | Copies two source words to object offsets +0x10 and +0x14, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing words to object offsets +0x94, +0x98, and +0x9c. |
