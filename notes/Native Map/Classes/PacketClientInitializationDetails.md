# Class: PacketClientInitializationDetails

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089bfa0` | `PacketClientInitializationDetailsCtor` | ✅ | ctor | Constructor of PacketClientInitializationDetails: installs BasePacket::vftable, stores opcode 0xa9 (169) at +4, installs the derived vtable and clears the payload word at +8. |
| `008b1850` | `PacketClientInitializationDetails::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientInitializationDetails::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
