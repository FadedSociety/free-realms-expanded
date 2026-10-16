# Class: GuildDataFullPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b17f30` | `GuildDataFullPacket::GuildDataFullPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x16 (22), installs GuildDataFullPacket::vftable and zeroes the member at +0x18 (param_1[6]). |
| `00b17f90` | `GuildDataFullPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildDataFullPacket::vftable: under an exception frame it restores BaseGuildPacket::vftable and then BasePacket::vftable and frees the object with _free when bit 0 of the flag argu… |
