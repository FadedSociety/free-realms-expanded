# Class: PacketMountRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b3730` | `PacketMountRequest::PacketMountRequest` | ✅ | ctor | Calls the mount family constructor 0089f9c0(1) (which stores BasePacket id 0xa8, 168, and writes sub-opcode 1), installs PacketMountRequest::vftable, stores param_2, param_3 and param_4 into +0x10, +0x14 and +0x18 and z… |
| `008b37b0` | `— ⚠reject` | ⚪ | dtor | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
