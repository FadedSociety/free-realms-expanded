# Class: GuildInviteAcceptNotificationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1a2d0` | `GuildInviteAcceptNotificationPacket::Destruct` | 🟢 | dtor | Destructor body of the guild invite-accept notification: releases its name record through 0070a790 under an exception frame, then restores BaseGuildPacket::vftable and BasePacket::vftable; callers are the deleting destr… |
| `00b1b180` | `GuildInviteAcceptNotificationPacket::GuildInviteAcceptNotificationPacket` | ✅ | ctor | Constructor: calls the base guild constructor 00b17220 (opcode 129, sub-opcode word zeroed), installs GuildInviteAcceptNotificationPacket::vftable and builds one name record with 0070bac0. |
| `00b1b1e0` | `GuildInviteAcceptNotificationPacket::ScalarDeletingDtor` | ✅ | dtor | Slot [0] scalar deleting destructor of GuildInviteAcceptNotificationPacket::vftable: calls the destructor body 00b1a2d0 and frees the object with _free when bit 0 of the flag argument is set. |
