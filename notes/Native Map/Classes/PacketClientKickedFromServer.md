# Class: PacketClientKickedFromServer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ceef0` | `PacketClientKickedFromServer::PacketClientKickedFromServer` | ✅ | ctor | Packet constructor: installs BasePacket::vftable, stores the opcode 0x51 (81) in the id slot +4, installs PacketClientKickedFromServer::vftable and initializes one embedded SoeUtil::IString (+8) with the shared empty bu… |
| `008cef60` | `PacketClientKickedFromServer::~PacketClientKickedFromServer` | 🟢 | dtor | Destructor body: releases the embedded SoeUtil::IString at +8 (capacity +0x14) through the locked refcount and virtual slot +8, then resets the vtable to BasePacket::vftable. |
| `008dc4d0` | `PacketClientKickedFromServer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cef60 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
