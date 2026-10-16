# Class: GuildCanCreateGuildPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b17280` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x20 (32), installs GuildCanCreateGuildPacket::vftable and zeroes the single byte member at +0xc. |
| `00b17ec0` | `GuildCanCreateGuildPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildCanCreateGuildPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the fla… |
