# Class: AnnouncementAdminSendAll

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e100` | `AnnouncementAdminSendAll::AnnouncementAdminSendAll` | ✅ | ctor | Constructor of the sub-opcode 3 announcement packet: calls the family constructor 00a3df90 with argument 3 (packet id 0xc1, sub-opcode 3) and installs AnnouncementAdminSendAll::vftable. |
| `00a3e610` | `AnnouncementAdminSendAll::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of the sub-opcode 3 announcement packet: installs BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set. |
