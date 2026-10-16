# Class: PacketClientExitLaunchUrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008dc5d0` | `PacketClientExitLaunchUrl::PacketClientExitLaunchUrl` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores opcode 0x61 (97) at +4 and PacketClientExitLaunchUrl::vftable; constructs a StringFixed<256> subobject at +8 through 004029b0. |
| `008dc640` | `PacketClientExitLaunchUrl::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf270 (packet destructor with one StringFixed256 member) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
