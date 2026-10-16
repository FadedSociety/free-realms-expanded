# Class: PacketClientMetrics

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1420` | `PacketClientMetrics::PacketClientMetrics` | ✅ | ctor | Constructor: writes BasePacket::vftable, stores id 0x69 at this+4, then installs PacketClientMetrics::vftable. Calls FUN_0089b800 with the payload address this+8 in ECX; that helper clears the 0x98-byte payload and writ… |
| `008b1490` | `PacketClientMetrics::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketClientMetrics::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
