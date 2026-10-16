# Class: AdventurersJournalInfoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a42790` | `AdventurersJournalInfoPacket::AdventurersJournalInfoPacket` | ✅ | ctor | Constructor of the journal info message: runs the family base constructor FUN_00a426c0(1) (opcode 0xd1, sub-opcode 1 at +8), installs AdventurersJournalInfoPacket::vftable and stores param_2 in the payload dword at +0xc. |
| `00a42800` | `AdventurersJournalInfoPacket::~AdventurersJournalInfoPacket` | 🟢 | dtor | Destructor of the journal info packet: reinstalls BasePacket::vftable at +0, i.e. the packet owns nothing beyond its header and one payload dword. |
| `00a43170` | `AdventurersJournalInfoPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the journal info packet: installs BasePacket::vftable and frees the packet with _free when bit 0 of the delete flag is set. |
