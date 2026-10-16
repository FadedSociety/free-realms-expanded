# Class: PacketPointOfInterestDefinitionReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b97ad0` | `PacketPointOfInterestDefinitionReply::PacketPointOfInterestDefinitionReply` | ✅ | ctor | Constructor: stores BasePacket::vftable, writes the opcode 0x39 (57) into the packet id slot at +4, stores PacketPointOfInterestDefinitionReply::vftable and zeroes the two list words at +8 and +0xc. |
| `00b97f50` | `PacketPointOfInterestDefinitionReply::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set. |
