# Class: AdventurersJournalQuestUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a42850` | `AdventurersJournalQuestUpdatePacket::AdventurersJournalQuestUpdatePacket` | ✅ | ctor | Constructor of the journal quest-update message: runs the family base constructor FUN_00a426c0(2) (opcode 0xd1, sub-opcode 2), installs AdventurersJournalQuestUpdatePacket::vftable and stores param_2 in the payload dwor… |
| `00a428c0` | `AdventurersJournalQuestUpdatePacket::~AdventurersJournalQuestUpdatePacket` | 🟢 | dtor | Destructor of the journal quest-update packet: reinstalls BasePacket::vftable at +0. |
| `00a431d0` | `AdventurersJournalQuestUpdatePacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the journal quest-update packet: installs BasePacket::vftable and frees the packet with _free when bit 0 of the delete flag is set. |
