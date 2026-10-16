# Class: GuildPromotePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0e350` | `GuildPromotePacket::~GuildPromotePacket` | 🟢 | dtor | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c0e580` | `GuildPromotePacket::GuildPromotePacket` | ✅ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 4, stores GuildPromotePacket::vftable at *this, clears the four scalar fields at +0x10, +0x14, +0x18 and +0x1c and returns the object. |
| `00c0e5f0` | `GuildPromotePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildPromotePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object through _free when bit 0 of param_2 is set. |
