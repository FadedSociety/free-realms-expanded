# Class: PacketClientPendingKickFromServer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cefe0` | `PacketClientPendingKickFromServer::PacketClientPendingKickFromServer` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores the opcode 99 in the id slot +4, installs PacketClientPendingKickFromServer::vftable, initializes one embedded SoeUtil::IString (+8) and zeroes one further dword … |
| `008dc4f0` | `PacketClientPendingKickFromServer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf060 (a packet destructor handling an IString member at +8) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
