# Class: GuildQuitPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0e440` | `GuildQuitPacket::~GuildQuitPacket` | 🟢 | dtor | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c0e820` | `GuildQuitPacket::GuildQuitPacket` | ✅ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 7, stores GuildQuitPacket::vftable at *this, clears the two scalar fields at +0x10 and +0x14 and returns the object. |
| `00c0e880` | `GuildQuitPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildQuitPacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object through _free when bit 0 of param_2 is set. |
