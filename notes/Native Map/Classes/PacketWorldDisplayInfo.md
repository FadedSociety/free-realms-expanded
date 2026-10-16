# Class: PacketWorldDisplayInfo

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b630` | `PacketWorldDisplayInfoCtor` | 🟢 | ctor | Constructor of PacketWorldDisplayInfo: installs BasePacket::vftable, stores opcode 0x56 (86) at +4, installs the derived vtable and clears the payload word at +8. |
| `008b1290` | `PacketWorldDisplayInfo::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketWorldDisplayInfo::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
