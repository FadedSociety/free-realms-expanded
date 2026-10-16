# Class: ClientUpdatePacketAddAbility

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e730` | `ClientUpdatePacketAddAbility::ClientUpdatePacketAddAbility` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x16) (family 0x26, sub-opcode 22), installs ClientUpdatePacketAddAbility::vftable and initialises four payload dwords: param_1[3] = 0xffffffff, … |
| `0089e7a0` | `ClientUpdatePacketAddAbility::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketAddAbility vtable is not restored because the class holds only plain dwords. |
| `008b28b0` | `ClientUpdatePacketAddAbility::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
