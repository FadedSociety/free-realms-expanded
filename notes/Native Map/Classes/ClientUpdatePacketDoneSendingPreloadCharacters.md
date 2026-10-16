# Class: ClientUpdatePacketDoneSendingPreloadCharacters

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089e8c0` | `ClientUpdatePacketDoneSendingPreloadCharacters::ClientUpdatePacketDoneSendingPr…` | ✅ | ctor | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1a) (family 0x26, sub-opcode 26), installs ClientUpdatePacketDoneSendingPreloadCharacters::vftable and clears the byte at +0xc. |
| `0089e920` | `ClientUpdatePacketDoneSendingPreloadCharacters::Dtor` | 🟢 | dtor | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived preload-marker vtable is not restored because the class holds only a byte. |
| `008b2940` | `ClientUpdatePacketDoneSendingPreloadCharacters::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
