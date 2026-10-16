# Class: ClientUpdatePacketCoinCount

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e670` | `ClientUpdatePacketCoinCount::ClientUpdatePacketCoinCount` | ✅ | ctor | Packet constructor taking one value: calls the client-update base constructor FUN_0089d180(0x13) (family 0x26, sub-opcode 19), installs ClientUpdatePacketCoinCount::vftable and stores param_2 into the payload dword para… |
| `0089e6e0` | `ClientUpdatePacketCoinCount::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketCoinCount vtable is not restored because the class holds only ints. |
| `008b2840` | `ClientUpdatePacketCoinCount::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
