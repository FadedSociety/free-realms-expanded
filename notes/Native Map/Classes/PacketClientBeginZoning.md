# Class: PacketClientBeginZoning

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc360` | `PacketClientBeginZoning::PacketClientBeginZoning` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x1f (31) at +4, installs PacketClientBeginZoning::vftable, constructs a StringFixed<32> subobject at +8 through 00703540, then sets sentinel +0x3c, initia… |
| `008dc410` | `PacketClientBeginZoning::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (raw bytes 56 8b f1 e8 ... f6 44 24 08 01 74 09 56 e8 ...): runs the destructor body 008cedd0 and frees the object with _free (00d82463) when the low bit of the flag argument is set, returning… |
