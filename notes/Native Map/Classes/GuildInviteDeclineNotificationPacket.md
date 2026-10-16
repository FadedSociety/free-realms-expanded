# Class: GuildInviteDeclineNotificationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1a330` | `GuildInviteDeclineNotificationPacket::Destruct` | 🟢 | dtor | Destructor body of the guild invite-decline notification: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vftable and BasePacket::vftable; callers are the deleting dest… |
| `00b1b200` | `GuildInviteDeclineNotificationPacket::GuildInviteDeclineNotificationPacket` | ✅ | ctor | Constructor: calls the base guild constructor 00b17220 (opcode 129, sub-opcode word zeroed), installs GuildInviteDeclineNotificationPacket::vftable and builds one name record with 0070bac0. |
| `00b1b260` | `GuildInviteDeclineNotificationPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildInviteDeclineNotificationPacket::vftable: calls the destructor body 00b1a330 and frees the object with _free when bit 0 of the flag argument is set. |
