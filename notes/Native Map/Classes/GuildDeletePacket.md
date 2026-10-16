# Class: GuildDeletePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0e300` | `GuildDeletePacket::~GuildDeletePacket` | 🟢 | dtor | Destructor body of a guild packet with no owned members: writes BaseGuildPacket::vftable and then BasePacket::vftable at *this and returns (single vtable reset, no member releases). |
| `00c0e4b0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 2 (which stamps BaseGuildPacket::vftable and the guild family), stores GuildDeletePacket::vftable at *this, then clears the two scalar fi… |
| `00c0e510` | `GuildDeletePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildDeletePacket): resets the object's vtables to BaseGuildPacket::vftable and then BasePacket::vftable, and when bit 0 of the flag byte param_2 is set frees the object thro… |
