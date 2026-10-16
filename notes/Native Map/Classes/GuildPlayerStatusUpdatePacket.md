# Class: GuildPlayerStatusUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b182b0` | `GuildPlayerStatusUpdatePacket::GuildPlayerStatusUpdatePacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x1f (31) and installs GuildPlayerStatusUpdatePacket::vftable; no further members are initialized here. |
| `00b18310` | `GuildPlayerStatusUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildPlayerStatusUpdatePacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the… |
| `00b1add0` | `GuildPlayerStatusUpdatePacket::ReadBody` | 🟢 | method | Reads the body of a player status update: two 16-bit header values through 00b1a810, an 8-byte pair into +0x10/+0x14, a second 8-byte pair into +0x18/+0x1c and one boolean byte at +0x20, flagging the reader when a field… |
