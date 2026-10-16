# Class: PacketClientLog

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc430` | `PacketClientLog::PacketClientLog` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x6d (109) at +4 and PacketClientLog::vftable; constructs StringFixed<64> at +8 through 00402a10 and StringFixed<2048> at +0x5c through 00402a70. |
| `008dc4b0` | `PacketClientLog::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cee70 (releases the StringFixed64/StringFixed2048 members) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
