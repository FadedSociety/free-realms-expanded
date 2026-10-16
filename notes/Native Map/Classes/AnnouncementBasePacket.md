# Class: AnnouncementBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3df30` | `AnnouncementBasePacket::AnnouncementBasePacket` | ✅ | ctor | Base constructor of the announcement packet family: installs BasePacket::vftable, stores the packet id 0xc1 (193) in the id slot +4, installs AnnouncementBasePacket::vftable and zeroes the sub-opcode slot +8. |
| `00a3df90` | `AnnouncementBasePacket::AnnouncementBasePacketSub` | ✅ | ctor | Family constructor taking the sub-opcode: installs BasePacket::vftable, stores packet id 0xc1 (193) at +4, installs AnnouncementBasePacket::vftable and stores the argument in the sub-opcode slot +8. |
| `00a3e550` | `AnnouncementBasePacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of the announcement family base: under an SEH frame installs BasePacket::vftable at +0 and calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
