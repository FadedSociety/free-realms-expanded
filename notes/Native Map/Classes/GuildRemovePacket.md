# Class: GuildRemovePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0e3f0` | `GuildRemovePacket::~GuildRemovePacket` | 🟢 | dtor | Destructor body resetting the object's vtable: stores BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c0e740` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 6, stores GuildRemovePacket::vftable at *this, clears the four scalar fields at +0x10..+0x1c and returns the object. |
| `00c0e7b0` | `GuildRemovePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildRemovePacket): resets the vtables to BaseGuildPacket::vftable then BasePacket::vftable and frees the object through _free when bit 0 of param_2 is set. |
