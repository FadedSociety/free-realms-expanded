# Class: PacketPointOfInterestDefinitionRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b97a20` | `PacketPointOfInterestDefinitionRequest::PacketPointOfInterestDefinitionRequest` | ✅ | ctor | Constructor: stores BasePacket::vftable, writes the opcode 0x38 (56) into the packet id slot at +4 and then stores PacketPointOfInterestDefinitionRequest::vftable; no payload fields. |
| `00b97ef0` | `PacketPointOfInterestDefinitionRequest::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: stores BasePacket::vftable at this+0 inside an SEH frame and calls _free(this) when bit 0 of param_2 is set; returns this. |
