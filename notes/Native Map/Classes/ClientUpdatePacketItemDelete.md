# Class: ClientUpdatePacketItemDelete

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d6c0` | `ClientUpdatePacketItemDeleteCtor` | ✅ | ctor | Constructor of ClientUpdatePacketItemDelete: runs the base constructor FUN_0089d180 with sub-opcode 4, installs the derived vtable, calls FUN_0101cd40, and when param_2 is non-null copies its first dword into +0xc. |
| `008b2760` | `ClientUpdatePacketItemDelete::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
