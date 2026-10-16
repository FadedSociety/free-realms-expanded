# Class: AnnouncementDataRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e050` | `AnnouncementDataRequestPacket::AnnouncementDataRequestPacket` | ✅ | ctor | Constructor of the sub-opcode 1 announcement packet: calls the family constructor 00a3df90 with argument 1 (so packet id 0xc1 and sub-opcode 1 are stored) and installs AnnouncementDataRequestPacket::vftable. |
| `00a3e5b0` | `AnnouncementDataRequestPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of the sub-opcode 1 announcement packet: installs BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set. |
