# Class: PacketUpdateClientSessionData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc510` | `PacketUpdateClientSessionData::PacketUpdateClientSessionData` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x52 (82) at +4 and PacketUpdateClientSessionData::vftable; constructs four StringFixed<256> members at +8, +0x11c, +0x230 and +0x344 through 004029b0. |
| `008dc5b0` | `PacketUpdateClientSessionData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf0e0 (destroys the four StringFixed256 members) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
