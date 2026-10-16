# Class: PacketClientSettings

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc780` | `PacketClientSettings::PacketClientSettings` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0xc3 (195) at +4 and PacketClientSettings::vftable; calls the three-IString initializer 008dc260 with ECX=this+8. |
| `008dc7f0` | `PacketClientSettings::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf410 (which delegates to 008ceaf0, the three-IString payload destructor) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
