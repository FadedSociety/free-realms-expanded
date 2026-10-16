# Class: ClientUpdatePacketEquipItem

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc810` | `ClientUpdatePacketEquipItem::ClientUpdatePacketEquipItem` | ✅ | ctor | Packet constructor: calls family initializer 0089d180 with sub-opcode 5 (family 38), installs ClientUpdatePacketEquipItem::vftable, calls 0101cd40 with ECX=this+0xc and 0070b770 with ECX=this+0x10, then zeroes +0x1d8, +… |
| `008dc890` | `ClientUpdatePacketEquipItem::~ClientUpdatePacketEquipItem` | ✅ | dtor | Destructor body: rewrites ClientUpdatePacketEquipItem::vftable, runs the member destructor 0070a900, then rewrites BaseClientUpdatePacket::vftable and BasePacket::vftable in turn; referenced from an EH unwind funclet (0… |
| `008dc900` | `ClientUpdatePacketEquipItem::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008dc890 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
