# Class: PacketPOIChangeMessage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b9f0` | `PacketPOIChangeMessageCtor` | ✅ | ctor | Constructor of PacketPOIChangeMessage: installs BasePacket::vftable, stores opcode 0x68 (104) at +4, installs the derived vtable and clears the three payload words at +8/+0xc/+0x10. |
| `008b1550` | `PacketPOIChangeMessage::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketPOIChangeMessage::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
