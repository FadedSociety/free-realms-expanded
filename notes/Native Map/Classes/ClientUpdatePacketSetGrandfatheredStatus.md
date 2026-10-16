# Class: ClientUpdatePacketSetGrandfatheredStatus

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e970` | `ClientUpdatePacketSetGrandfatheredStatus::ClientUpdatePacketSetGrandfatheredSta…` | ✅ | ctor | Packet constructor taking one byte: calls the client-update base constructor FUN_0089d180(0x1b) (family 0x26, sub-opcode 27), installs ClientUpdatePacketSetGrandfatheredStatus::vftable and stores param_2 into the byte a… |
| `0089e9e0` | `ClientUpdatePacketSetGrandfatheredStatus::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the class holds only a byte. |
| `008b29b0` | `ClientUpdatePacketSetGrandfatheredStatus::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
