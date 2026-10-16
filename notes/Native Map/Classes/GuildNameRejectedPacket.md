# Class: GuildNameRejectedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b18100` | `GuildNameRejectedPacket::GuildNameRejectedPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x14 (20), installs GuildNameRejectedPacket::vftable and zeroes the two member dwords at +0x10 and +0x14. |
| `00b18160` | `GuildNameRejectedPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildNameRejectedPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the flag … |
