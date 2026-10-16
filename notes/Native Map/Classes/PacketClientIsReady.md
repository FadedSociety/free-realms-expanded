# Class: PacketClientIsReady

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00acc6f0` | `PacketClientIsReady::PacketClientIsReady` | ✅ | ctor | Constructor: installs BasePacket::vftable, stamps the packet id 13 into the word at +4 and installs PacketClientIsReady::vftable. |
| `00acc750` | `PacketClientIsReady::Destructor` | ✅ | dtor | 73-byte destructor body: rewrites the vtable to PacketClientIsReady::vftable and then to BasePacket::vftable; reached only from an unwind funclet. |
| `00acc7a0` | `PacketClientIsReady::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to PacketClientIsReady::vftable then BasePacket::vftable, and frees the object with _free when bit 0 of the flag argument is set. |
