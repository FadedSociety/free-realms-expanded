# Class: GuildNameUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19b70` | `GuildNameUpdatePacket::GuildNameUpdatePacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x12 (18), installs GuildNameUpdatePacket::vftable, zeroes the dwords at +0x10 and +0x14 and initializes the embedded IString member at +0x18. |
| `00b19be0` | `— ⚠reject` | ⚪ | dtor | Destructor body of the guild rename announcement: rewrites the IString vtable at +0x18, releases the string buffer through IString vtable slot 2 when the reference count is positive, then restores BaseGuildPacket::vftab… |
| `00b1a4b0` | `GuildNameUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildNameUpdatePacket::vftable: calls the destructor body 00b19be0 and frees the object with _free when bit 0 of the flag argument is set. |
