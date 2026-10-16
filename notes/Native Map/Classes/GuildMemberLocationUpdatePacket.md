# Class: GuildMemberLocationUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b19860` | `GuildMemberLocationUpdatePacket::GuildMemberLocationUpdatePacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x19 (25), installs GuildMemberLocationUpdatePacket::vftable, stores the two header arguments at +0x10 and +0x14, and initializes the embedded SoeUt… |
| `00b198e0` | `GuildMemberLocationUpdatePacket::Destruct` | 🟢 | dtor | Destructor body of the member location update: rewrites the entry-array vtable at +0x18, zeroes the array count, frees the entry storage at +0x1c with alignment 8 through 00401c00 and nulls it, then restores BaseGuildPa… |
| `00b1a450` | `GuildMemberLocationUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildMemberLocationUpdatePacket::vftable: calls the destructor body 00b198e0 and frees the object with _free when bit 0 of the flag argument is set. |
