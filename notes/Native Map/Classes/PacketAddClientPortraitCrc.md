# Class: PacketAddClientPortraitCrc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b4a9c0` | `PacketAddClientPortraitCrc::Ctor ⚠downgrade` | 🟢 | ctor | Constructs a PacketAddClientPortraitCrc object: stores BasePacket::vftable, writes opcode 0x73 (115), stores PacketAddClientPortraitCrc::vftable, and places its argument at +8. It does not send the packet. |
| `00b4aa30` | `PacketAddClientPortraitCrc::Dtor` | ✅ | dtor | Destructor body: rewrites the object's vtable to BasePacket::vftable (018011ec); no members are released. |
| `00b4ae50` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: rewrites the vtable to BasePacket::vftable (018011ec) and calls _free(this) when the low bit of the deleting flag is set (the free does not return), otherwise returns this. |
