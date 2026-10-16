# Class: GuildRenameGuildPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0ece0` | `GuildRenameGuildPacket::GuildRenameGuildPacket ⚠downgrade` | 🟢 | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 0xd (13), stores GuildRenameGuildPacket::vftable at *this, clears the two scalar fields at +0x10/+0x14 and initialises two embedded SoeUt… |
| `00c0ed70` | `— ⚠reject` | ⚪ | dtor | Destructor body: releases the embedded SoeUtil::IString members at +0x28 and +0x18 in that order (interlocked reference decrement at buffer header ptr-4 with IString virtual slot +8 on zero) and then resets the vtables … |
| `00c0ee70` | `GuildRenameGuildPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildRenameGuildPacket): calls the destructor body FUN_00c0ed70 (releases the two embedded strings, resets the vtables) and frees the object through _free when bit 0 of param… |
