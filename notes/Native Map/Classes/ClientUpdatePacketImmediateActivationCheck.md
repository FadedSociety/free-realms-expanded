# Class: ClientUpdatePacketImmediateActivationCheck

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089ea30` | `ClientUpdatePacketImmediateActivationCheck::ClientUpdatePacketImmediateActivati…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1e) (family 0x26, sub-opcode 30), zeroes the dword at +0xc through the 9-byte helper FUN_0101cd40 (its body is a single store of 0) and install… |
| `0089ea90` | `ClientUpdatePacketImmediateActivationCheck::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the class holds only plain dwords. |
| `008b2a20` | `ClientUpdatePacketImmediateActivationCheck::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
