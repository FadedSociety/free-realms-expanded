# Class: PacketWorldShutdownNotice

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc660` | `PacketWorldShutdownNotice::PacketWorldShutdownNotice` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x5c (92) at +4 and PacketWorldShutdownNotice::vftable; constructs a StringFixed<512> subobject at +8 through 007001c0. |
| `008dc6d0` | `PacketWorldShutdownNotice::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf2d0 (packet destructor for the member handled by 006fff70) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
