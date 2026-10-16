# Class: PacketSetClientArea

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b6e0` | `PacketSetClientAreaCtor` | ✅ | ctor | Constructor of PacketSetClientArea: installs BasePacket::vftable, stores opcode 0x59 (89) at +4, installs the derived vtable, sets the area id at +8 to -1, the byte flag at +0xc to 1 and clears +0x10. |
| `008b12f0` | `PacketSetClientArea::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketSetClientArea::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
