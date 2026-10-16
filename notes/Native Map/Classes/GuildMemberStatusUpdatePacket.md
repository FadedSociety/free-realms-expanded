# Class: GuildMemberStatusUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1a390` | `GuildMemberStatusUpdatePacket::Destruct` | 🟢 | dtor | Destructor body of the guild member status update: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vftable and BasePacket::vftable; callers are the deleting destructor … |
| `00b1b280` | `GuildMemberStatusUpdatePacket::GuildMemberStatusUpdatePacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x11 (17), installs GuildMemberStatusUpdatePacket::vftable, zeroes the four dwords at +0x10 through +0x1c, builds a name record with 0070bac0 and ze… |
| `00b1b320` | `GuildMemberStatusUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildMemberStatusUpdatePacket::vftable: calls the destructor body 00b1a390 and frees the object with _free when bit 0 of the flag argument is set. |
