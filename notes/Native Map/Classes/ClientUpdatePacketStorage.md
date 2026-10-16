# Class: ClientUpdatePacketStorage

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d5e0` | `— ⚠reject` | ⚪ | ctor | Constructor of ClientUpdatePacketStorage: runs the base constructor FUN_0089d180 with sub-opcode 0x1f (31), installs the derived vtable and stores the two payload arguments at +0xc and +0x10. |
| `008b26f0` | `ClientUpdatePacketStorage::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseClientUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; reached only through a vtable slot. |
