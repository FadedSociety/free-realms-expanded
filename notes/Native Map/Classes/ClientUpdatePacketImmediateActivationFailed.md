# Class: ClientUpdatePacketImmediateActivationFailed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089eae0` | `ClientUpdatePacketImmediateActivationFailed::ClientUpdatePacketImmediateActivat…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1d) (family 0x26, sub-opcode 29), zeroes the dword at +0xc through FUN_0101cd40 and installs ClientUpdatePacketImmediateActivationFailed::vftab… |
| `0089eb40` | `ClientUpdatePacketImmediateActivationFailed::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the class holds only plain dwords. |
| `008b2a90` | `ClientUpdatePacketImmediateActivationFailed::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
