# Class: ClientUpdatePacketMana

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d360` | `ClientUpdatePacketManaCtor` | ✅ | ctor | Constructor of ClientUpdatePacketMana: runs the base constructor FUN_0089d180 with sub-opcode 0xd (13), installs the derived vtable and clears the payload fields at +0xc/+0x10 and the byte at +0x14. |
| `008b2570` | `ClientUpdatePacketMana::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers, so it is a vtable-slot entry only. |
