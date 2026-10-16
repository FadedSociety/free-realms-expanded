# Class: GuildInvitePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0ebd0` | `GuildInvitePacket::GuildInvitePacket` | ✅ | ctor | Constructor: calls the guild-packet base constructor FUN_00a09b70 with sub-opcode 3, stores GuildInvitePacket::vftable at *this, clears the four scalar fields at +0x10..+0x1c, and initialises one embedded SoeUtil::IStri… |
| `00c0ec50` | `GuildInvitePacket::~GuildInvitePacket` | ✅ | dtor | Destructor body: releases the embedded SoeUtil::IString member at +0x20 (interlocked reference decrement on the buffer header at ptr-4, virtual slot +8 when it reaches zero) and resets the vtables to BaseGuildPacket::vf… |
| `00c0ee50` | `GuildInvitePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of GuildInvitePacket): calls the destructor body FUN_00c0ec50 (releases the embedded string, resets the vtables) and frees the object through _free when bit 0 of param_2 is set. |
