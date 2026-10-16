# Class: AnnouncementDataSendPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a40460` | `AnnouncementDataSendPacket::AnnouncementDataSendPacket` | ✅ | ctor | Constructor: runs the announcement base constructor FUN_00a3df90(2) (which installs BasePacket and AnnouncementBasePacket vtables, sets the base opcode 0xc1 at +4 and the sub-opcode 2 at +8), then installs AnnouncementD… |
| `00a404d0` | `AnnouncementDataSendPacket::~AnnouncementDataSendPacket` | ✅ | dtor | Installs AnnouncementDataSendPacket::vftable, removes list-head nodes through FUN_00a3f6d0 until the +0x10 head is empty, reinstalls the embedded list vtable at +0x0c, then reaches a second loop whose saved head is alre… |
| `00a40570` | `AnnouncementDataSendPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of AnnouncementDataSendPacket: calls the destructor body 00a404d0 and frees the packet with _free when bit 0 of the delete flag is set. |
| `00a40870` | `— ⚠reject` | ⚪ | method | Reads a packet payload into a node list: with param_1 non-null and a non-null buffer/len it builds a local raw reader over (param_2, param_3) with the length/begin/end/cursor/error fields, initialises it with FUN_00a3ef… |
