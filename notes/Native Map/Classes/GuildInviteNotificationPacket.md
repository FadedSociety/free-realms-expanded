# Class: GuildInviteNotificationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1a210` | `— ⚠reject` | ⚪ | dtor | Destructor body of the guild invite notification: rewrites the IString vtable at +0x148, releases that string buffer through IString vtable slot 2 when the reference count is positive, then calls 00b1a1b0 to release the… |
| `00b1b0d0` | `GuildInviteNotificationPacket::GuildInviteNotificationPacket` | ✅ | ctor | Constructor: calls the guild base constructor 00a09b70 with sub-opcode 0x18 (24), installs GuildInviteNotificationPacket::vftable, builds the two embedded name records with 00b1b000 and initializes the IString at +0x148… |
| `00b1b160` | `GuildInviteNotificationPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildInviteNotificationPacket::vftable: calls the destructor body 00b1a210 and frees the object with _free when bit 0 of the flag argument is set. |
