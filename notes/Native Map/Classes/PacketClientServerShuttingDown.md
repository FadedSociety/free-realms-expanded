# Class: PacketClientServerShuttingDown

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b460` | `PacketClientServerShuttingDownCtor` | ✅ | ctor | Constructor of PacketClientServerShuttingDown: installs BasePacket::vftable, stores opcode 0x49 (73) at +4, then installs the derived vtable. |
| `008b1230` | `PacketClientServerShuttingDown::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientServerShuttingDown::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
