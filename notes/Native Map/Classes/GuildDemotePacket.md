# Class: GuildDemotePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0e3a0` | `GuildDemotePacket::~GuildDemotePacket` | 🟢 | dtor | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c0e660` | `GuildDemotePacket::GuildDemotePacket` | ✅ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 5, stores GuildDemotePacket::vftable at *this, clears the four scalar fields at +0x10..+0x1c and returns the object. |
| `00c0e6d0` | `GuildDemotePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildDemotePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object through _free when bit 0 of param_2 is set. |
