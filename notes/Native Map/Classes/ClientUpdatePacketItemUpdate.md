# Class: ClientUpdatePacketItemUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b25e0` | `ClientUpdatePacketItemUpdate::ClientUpdatePacketItemUpdate` | ✅ | ctor | Calls the opcode-38 family constructor 0089d180(3) (which stores BasePacket id 0x26 and sub-opcode 3), installs ClientUpdatePacketItemUpdate::vftable, constructs the packet member through 0101cd40 and clears fields +0x1… |
| `008b2680` | `ClientUpdatePacketItemUpdate::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot (no direct callers). |
