# Class: PacketZoneTeleportRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b97b80` | `PacketZoneTeleportRequest::PacketZoneTeleportRequest` | ✅ | ctor | Constructor: stores BasePacket::vftable, writes the opcode 0x5a (90) into the packet id slot at +4, stores PacketZoneTeleportRequest::vftable and stores the single dword argument param_2 at +8. |
| `00b97fb0` | `PacketZoneTeleportRequest::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set. |
| `00b98ad0` | `PacketZoneTeleportRequest_Serialize` | 🟢 | method | Serializes the op-90 teleport request into a writer: writes the low 16 bits of the packet id at this+4 as a u16 and the dword at this+8 as four bytes, each write clamped to the writer remaining space. |
