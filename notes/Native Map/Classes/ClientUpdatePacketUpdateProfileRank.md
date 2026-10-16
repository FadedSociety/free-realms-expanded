# Class: ClientUpdatePacketUpdateProfileRank

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e350` | `ClientUpdatePacketUpdateProfileRank::ClientUpdatePacketUpdateProfileRank` | ✅ | ctor | Packet constructor taking two values: calls the client-update base constructor FUN_0089d180(0x12) (family 0x26, sub-opcode 18), installs ClientUpdatePacketUpdateProfileRank::vftable, then stores param_2 into param_1[3] … |
| `0089e3c0` | `ClientUpdatePacketUpdateProfileRank::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketUpdateProfileRank vtable is not restored, so this is the compiler-emitted base d… |
| `008b27d0` | `ClientUpdatePacketUpdateProfileRank::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
