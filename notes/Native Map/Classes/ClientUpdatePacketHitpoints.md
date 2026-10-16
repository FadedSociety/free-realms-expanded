# Class: ClientUpdatePacketHitpoints

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d2b0` | `ClientUpdatePacketHitpointsCtor` | ✅ | ctor | Constructor of ClientUpdatePacketHitpoints: runs the base constructor FUN_0089d180 with sub-opcode 1, then installs ClientUpdatePacketHitpoints::vftable. |
| `008b2500` | `ClientUpdatePacketHitpoints::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set. It has no direct callers, so it is reached only through a vtable slot. |
