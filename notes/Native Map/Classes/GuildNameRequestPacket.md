# Class: GuildNameRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b18000` | `GuildNameRequestPacket::GuildNameRequestPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0xe (14), installs GuildNameRequestPacket::vftable and zeroes the two member dwords at +0x10 and +0x14. |
| `00b18060` | `GuildNameRequestPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildNameRequestPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the flag a… |
