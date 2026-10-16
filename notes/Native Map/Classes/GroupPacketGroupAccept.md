# Class: GroupPacketGroupAccept

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1610` | `GroupPacketGroupAccept::~GroupPacketGroupAccept` | 🟢 | dtor | Destructor body of the group accept packet: runs the shared payload teardown FUN_008d1550 (members at +0x98 and +8), then rewrites the object vtable to BaseGroupPacket::vftable and BasePacket::vftable inside an SEH cook… |
| `008dcf30` | `GroupPacketGroupAccept::GroupPacketGroupAccept` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 4, installs GroupPacketGroupAccept::vftable, calls 008dce10 with ECX=this+0x10, sets byte +0x138 to 1 and zeroes words +0x13c..+0x14c. |
| `008dcfc0` | `GroupPacketGroupAccept::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d1610 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
| `008eb7c0` | `GroupPacketGroupAccept::CopyPayloadAtOffsets124` | 🟢 | method | Copies two source words to object offsets +0xa0 and +0xa4, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing words to object offsets +0x124, +0x128, and +0x12c. |
| `008eb830` | `GroupPacketGroupAccept::CopyPayloadAtOffsets94` | 🟢 | method | Copies two source words to object offsets +0x10 and +0x14, invokes the IString copy helper for source offsets 0, +0x24, and +0x48, and copies three trailing words to object offsets +0x94, +0x98, and +0x9c. |
