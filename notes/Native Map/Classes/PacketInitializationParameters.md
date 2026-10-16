# Class: PacketInitializationParameters

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc6f0` | `PacketInitializationParameters::PacketInitializationParameters` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0xa6 (166) at +4 and PacketInitializationParameters::vftable; constructs a StringFixed<64> subobject at +8 through 00402a10. |
| `008dc760` | `PacketInitializationParameters::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf3b0 (packet destructor with one StringFixed64 member) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
