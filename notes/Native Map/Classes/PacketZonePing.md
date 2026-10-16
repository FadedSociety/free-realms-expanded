# Class: PacketZonePing

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1350` | `PacketZonePing::PacketZonePing` | ✅ | ctor | Packet constructor: writes BasePacket::vftable, stores packet id 0x60 (96) at this+4 and installs PacketZonePing::vftable at offset 0. No payload fields are initialised. |
| `008b13b0` | `PacketZonePing::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor in slot 0 of PacketZonePing::vftable: writes BasePacket::vftable at offset 0 inside an SEH frame and frees this when flag bit 0 is set. |
