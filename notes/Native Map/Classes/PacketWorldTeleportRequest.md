# Class: PacketWorldTeleportRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c1a5e0` | `PacketWorldTeleportRequest::Ctor` | ✅ | ctor | Installs the BasePacket and PacketWorldTeleportRequest vtables, stores opcode 0x3a, and copies two 32-bit arguments into packet fields. |
| `00c1a650` | `PacketWorldTeleportRequest::ScalarDeletingDtor` | ✅ | dtor | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |
