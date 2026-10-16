# Class: PacketEnterSocialZoneRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c142c0` | `PacketEnterSocialZoneRequest::PacketEnterSocialZoneRequest` | ✅ | ctor | Constructor: stores BasePacket::vftable, writes the packet opcode 0xcc (204) at +4, then stores PacketEnterSocialZoneRequest::vftable at *this; no payload fields are initialised. |
| `00c1a6b0` | `PacketEnterSocialZoneRequest::ScalarDeletingDtor` | ✅ | dtor | Replaces the object vptr with BasePacket::vftable and frees the allocation when the deleting flag bit is set. |
